#[cfg(test)]
pub mod tests;

use cairo_rs::{
	hint_processor::builtin_hint_processor::builtin_hint_processor_definition::BuiltinHintProcessor,
	serde::deserialize_program::{deserialize_program_json, ProgramJson},
	types::{errors::program_errors, program::Program},
	vm::{
		errors::{cairo_run_errors::CairoRunError, vm_errors::VirtualMachineError},
		hook::Hooks,
	},
};
use clap::{Args, ValueHint};
use colored::Colorize;
use serde::Serialize;
use std::{fmt::Display, fs, io, path::PathBuf, sync::Arc, time::Instant};
use thiserror::Error;
use uuid::Uuid;

use super::{list::path_is_valid_directory, CommandExecution};

use crate::{
	cairo_run::cairo_run,
	compile::{self, compile},
	hints::{
		output_buffer::{clear_buffer, get_buffer, init_buffer},
		processor::setup_hint_processor,
		EXPECT_REVERT_FLAG,
	},
	hooks,
	io::{
		compiled_programs::{list_test_entrypoints, ListTestEntrypointsError},
		test_files::{list_test_files, ListTestsFilesError},
	},
};

/// Enum containing the possible errors that you may encounter in the ``Test`` module
#[derive(Error, Debug)]
// Todo: Maybe use anyhow at this level
#[allow(clippy::large_enum_variant)]
pub enum TestCommandError {
	#[error("Failed to list test entrypoints for file {0}: {1}")]
	ListEntrypoints(PathBuf, String),
	#[error("Failed to compile file {0}: {1}")]
	RunTest(String, PathBuf, String),
	#[error(transparent)]
	IO(#[from] io::Error),
	#[error(transparent)]
	JsonDeSerialization(#[from] serde_json::Error),
	#[error(transparent)]
	Compile(#[from] compile::Error),
	#[error(transparent)]
	Program(#[from] program_errors::ProgramError),
	#[error(transparent)]
	CairoRun(#[from] CairoRunError),
	#[error(transparent)]
	ListTestsFiles(#[from] ListTestsFilesError),
	#[error(transparent)]
	ListTestEntripoints(#[from] ListTestEntrypointsError),
}

/// Structure containing the path to a cairo directory.
/// Used to execute all the tests files contained in this directory
#[derive(Args, Debug)]
pub struct TestArgs {
	/// Path to a cairo directory
	#[clap(short, long, value_hint=ValueHint::DirPath, value_parser=path_is_valid_directory, default_value="./")]
	pub root: PathBuf,
	/// Max steps cap allowed in a test
	#[clap(short, long, default_value_t = 1000000)]
	pub max_step: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestStatus {
	SUCCESS,
	FAILURE,
}

/// Structure representing the result of one or multiple test.
/// Contains the output of the test, as well as the status.
pub struct TestResult {
	pub output: String,
	pub success: TestStatus,
}

impl From<(String, TestStatus)> for TestResult {
	fn from(from: (String, TestStatus)) -> Self {
		Self {
			output: from.0,
			success: from.1,
		}
	}
}

/// Execute command output
#[derive(Debug, Serialize, Default)]
pub struct TestOutput(String);

impl Display for TestOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", &self.0)
	}
}

/// Create a new ``Hooks`` object, with the followings hooks:
/// - pre_step_instruction
/// - post_step_instruction
///
/// see [src/hooks.rs]
fn setup_hooks() -> Hooks {
	Hooks::new(
		Arc::new(hooks::pre_step_instruction),
		Arc::new(hooks::post_step_instruction),
	)
}

/// Compile a cairo file, returning a truple
/// (path_to_original_code, path_to_compiled_code, entrypoints)
fn compile_and_list_entrypoints(
	path_to_code: PathBuf,
) -> Result<(PathBuf, PathBuf, Vec<String>), TestCommandError> {
	let path_to_compiled = compile(&path_to_code)?;
	let entrypoints = list_test_entrypoints(&path_to_compiled)?;
	Ok((path_to_code, path_to_compiled, entrypoints))
}

fn purge_hint_buffer(execution_uuid: &Uuid, output: &mut String) {
	// Safe to unwrap as long as `init_buffer` has been called before
	let buffer = get_buffer(execution_uuid).unwrap();
	if !buffer.is_empty() {
		output.push_str(&format!("[{}]:\n{}", "captured stdout".blue(), buffer));
	}
	clear_buffer(execution_uuid);
}

/// Execute a single test.
/// Take a program and a test name as input, search for this entrypoint in the compiled file
/// and execute it.
/// It will then return a TestResult, representing the output of the test.
fn test_single_entrypoint(
	program: ProgramJson,
	test_entrypoint: &str,
	hint_processor: &mut BuiltinHintProcessor,
	hooks: Option<Hooks>,
	max_step: u64,
) -> Result<TestResult, TestCommandError> {
	let start = Instant::now();
	let mut output = String::new();
	let execution_uuid = Uuid::new_v4();
	init_buffer(execution_uuid);

	let program = Program::from_json(program, Some(test_entrypoint))?;

	let res_cairo_run = cairo_run(program, hint_processor, execution_uuid, hooks, max_step);
	let duration = start.elapsed();
	let (opt_runner_and_output, test_success) = match res_cairo_run {
		Ok(res) => {
			output.push_str(&format!(
				"[{}] {} ({:?})\n",
				"OK".green(),
				test_entrypoint,
				duration
			));
			(Some(res), TestStatus::SUCCESS)
		},
		Err(CairoRunError::VirtualMachine(VirtualMachineError::CustomHint(
			custom_error_message,
		))) if custom_error_message == "skip" => {
			output.push_str(&format!("[{}] {}\n", "SKIPPED".yellow(), test_entrypoint,));
			(None, TestStatus::SUCCESS)
		},
		Err(CairoRunError::VirtualMachine(VirtualMachineError::CustomHint(
			custom_error_message,
		))) if custom_error_message == EXPECT_REVERT_FLAG => {
			output.push_str(&format!(
				"[{}] {}\nError: execution did not revert while expect_revert() was specified\n\n",
				"FAILED".red(),
				test_entrypoint,
			));
			(None, TestStatus::FAILURE)
		},
		Err(e) => {
			output.push_str(&format!(
				"[{}] {}\nError: {:?}\n\n",
				"FAILED".red(),
				test_entrypoint,
				e
			));
			(None, TestStatus::FAILURE)
		},
	};

	purge_hint_buffer(&execution_uuid, &mut output);
	let (mut runner, mut vm) = match opt_runner_and_output {
		Some(runner_and_vm) => runner_and_vm,
		None => return Ok((output, test_success).into()),
	};

	// Display the execution output if present
	match runner.get_output(&mut vm) {
		Ok(runner_output) => {
			if !runner_output.is_empty() {
				output.push_str(&format!(
					"[{}]:\n{}",
					"execution output".purple(),
					&runner_output
				));
			}
		},
		Err(e) => eprintln!("failed to get output from the cairo runner: {e}"),
	};

	output.push('\n');
	Ok((output, test_success).into())
}

/// Run every test contained in a cairo file.
/// this function will deserialize a compiled cairo file, and call ``test_single_entrypoint`` on
/// each entrypoint provided.
/// It will then return a TestResult corresponding to all the tests (SUCCESS if all the test
/// succeded, FAILURE otherwise).
fn run_tests_for_one_file(
	hint_processor: &mut BuiltinHintProcessor,
	path_to_original: PathBuf,
	path_to_compiled: PathBuf,
	test_entrypoints: Vec<String>,
	hooks: Hooks,
	max_step: u64,
) -> Result<TestResult, TestCommandError> {
	let file = fs::File::open(path_to_compiled).unwrap();
	let reader = io::BufReader::new(file);
	let program_json = deserialize_program_json(reader)?;

	let output = format!("Running tests in file {}\n", path_to_original.display());
	let res = test_entrypoints
		.into_iter()
		.map(|test_entrypoint| {
			test_single_entrypoint(
				program_json.clone(),
				&test_entrypoint,
				hint_processor,
				Some(hooks.clone()),
				max_step,
			)
		})
		.collect::<Result<Vec<_>, TestCommandError>>()?
		.into_iter()
		.fold((output, TestStatus::SUCCESS), |mut a, b| {
			a.0.push_str(&b.output);
			// SUCCESS if both a.1 and b.success are SUCCESS, otherwise, FAILURE
			a.1 = if a.1 == TestStatus::SUCCESS && b.success == TestStatus::SUCCESS {
				TestStatus::SUCCESS
			} else {
				TestStatus::FAILURE
			};
			a
		});
	Ok(res.into())
}

impl CommandExecution<TestOutput, TestCommandError> for TestArgs {
	fn exec(&self) -> Result<TestOutput, TestCommandError> {
		// Declare hints
		let mut hint_processor = setup_hint_processor();
		let hooks = setup_hooks();

		list_test_files(&self.root)?
			//.into_par_iter()
			.into_iter()
			.map(compile_and_list_entrypoints)
			.map(|res| -> Result<TestResult, TestCommandError> {
				match res {
					Ok((path_to_original, path_to_compiled, test_entrypoints)) => {
						run_tests_for_one_file(
							&mut hint_processor,
							path_to_original,
							path_to_compiled,
							test_entrypoints,
							hooks.clone(),
							self.max_step,
						)
					},
					Err(err) => Err(err),
				}
			})
			.for_each(|test_result| match test_result {
				Ok(result) => {
					println!("{}", result.output);
				},
				Err(err) => println!("{}", format!("Error: {}", err).red()),
			});

		Ok(Default::default())
	}
}
