pub mod cache;
#[cfg(test)]
pub mod tests;

use crate::hints::{self, EXPECT_REVERT_FLAG};
use regex::Regex;

use cairo_rs::{
	hint_processor::builtin_hint_processor::builtin_hint_processor_definition::{
		BuiltinHintProcessor, HintFunc,
	},
	serde::deserialize_program::{deserialize_program_json, ProgramJson},
	types::{errors::program_errors, program::Program},
	vm::{
		errors::{cairo_run_errors::CairoRunError, vm_errors::VirtualMachineError},
		hook::Hooks,
	},
};
use clap::{Args, ValueHint};
use colored::Colorize;
// 2023-01-06: wwe can't execute parallel since HintFunc are reference counted
// use rayon::prelude::*;
use serde::Serialize;
use serde_json::Value;
use std::{fmt::Display, fs, io, path::PathBuf, rc::Rc, sync::Arc, time::Instant};
use thiserror::Error;
use uuid::Uuid;

use super::{
	list::{path_is_valid_directory, ListArgs, ListCommandError, ListOutput},
	CommandExecution,
};

use crate::{
	cairo_run::cairo_run,
	compile::{self, compile},
	hints::{clear_buffer, expect_revert, get_buffer, init_buffer},
	hooks,
};

/// Enum containing the possible errors that you may encounter in the ``Test`` module
#[derive(Error, Debug)]
pub enum TestCommandError {
	#[error("Failed to list test entrypoints for file {0}: {1}")]
	ListEntrypointsError(PathBuf, String),
	#[error("Failed to compile file {0}: {1}")]
	RunTestError(String, PathBuf, String),
	#[error(transparent)]
	IOError(#[from] io::Error),
	#[error(transparent)]
	JsonError(#[from] serde_json::Error),
	#[error(transparent)]
	CompileError(#[from] compile::Error),
	#[error(transparent)]
	ProgramError(#[from] program_errors::ProgramError),
	#[error(transparent)]
	CairoRunError(#[from] CairoRunError),
	#[error(transparent)]
	ListCommandError(#[from] ListCommandError),
}

/// Structure containing the path to a cairo directory.
/// Used to execute all the tests files contained in this directory
#[derive(Args, Debug)]
pub struct TestArgs {
	/// Path to a cairo directory
	#[clap(short, long, value_hint=ValueHint::DirPath, value_parser=path_is_valid_directory, default_value="./")]
	pub root: PathBuf,
}

#[derive(PartialEq)]
pub enum TestStatus {
	SUCCESS,
	FAILURE,
}

impl Into<bool> for TestStatus {
	fn into(self) -> bool {
		match self {
			TestStatus::SUCCESS => true,
			TestStatus::FAILURE => false,
		}
	}
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

/// Get the list of test entrypoint from a compiled cairo file.
/// test entrypoint are function starting with "test_".
/// The function will return a list of test entrypoint as `String` (ie: "test_function");
///
/// return a vector of entrypoints
///
/// # Examples
///
/// Basic usage:
///
/// ```ignore
/// //assuming your program have a "test_function1" and "test_function2" functions.
///
/// let plain_path = PathBuf::from("path_to_your_program");
/// let compiled_path = compile(&plain_path)?;
/// let expected_entrypoints = vec![
/// 	"test_function1",
/// 	"test_function2"
/// ]
///
/// assert_eq!(list_test_entrypoints(compiled_path), expected_entrypoints);
/// ```
fn list_test_entrypoints(compiled_path: &PathBuf) -> Result<Vec<String>, TestCommandError> {
	let re = Regex::new(r"__main__.(test_\w+)$").expect("Should be a valid regex");
	let data = fs::read_to_string(compiled_path)?;
	let json = serde_json::from_str::<Value>(&data)?;
	let mut test_entrypoints = Vec::new();

	let identifiers = json["identifiers"].as_object();
	match identifiers {
		Some(identifiers) => {
			for (key, value) in identifiers {
				if re.is_match(key) && value["type"] == "function" {
					// capture 0 refers to the whole match
					// capture n-1 refers to the next to last match
					// captures are denoted with () in regex
					for capture in re.captures_iter(key) {
						// regex __main__.(test_\w+)$ has 2 captures
						// capture 0 is the whole match
						// capture 1 is the first (and last) capture in this regex
						test_entrypoints.push(capture[1].to_string());
					}
				}
			}
		},
		None => eprintln!("Compilation output does not contain identifiers"),
	}

	Ok(test_entrypoints)
}

/// Execute command output
#[derive(Debug, Serialize, Default)]
pub struct TestOutput(String);

impl Display for TestOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", &self.0)
	}
}

fn setup_hint_processor() -> BuiltinHintProcessor {
	let skip_hint = Rc::new(HintFunc(Box::new(hints::skip)));
	let mock_call_hint = Rc::new(HintFunc(Box::new(hints::mock_call)));
	let expect_revert_hint = Rc::new(HintFunc(Box::new(expect_revert)));
	let mut hint_processor = BuiltinHintProcessor::new_empty();
	hint_processor.add_hint(String::from("skip()"), skip_hint);
	hint_processor.add_hint(String::from("expect_revert()"), expect_revert_hint);
	hint_processor.add_hint(
		String::from("mock_call(func_to_mock, mock_ret_value)"),
		mock_call_hint,
	);
	hint_processor
}

///create a new ``Hooks`` object, with the followings hooks:
/// - pre_step_instruction
///
/// see [src/hooks.rs]
fn setup_hooks() -> Hooks {
	Hooks::new(Arc::new(hooks::pre_step_instruction))
}

/// List the cairo files contained in a directory.
/// Takes a path to a directory, and return a list of exact path to all cairo files contained into
/// this directory.
fn list_cairo_files(root: &PathBuf) -> Result<Vec<PathBuf>, ListCommandError> {
	ListArgs {
		root: PathBuf::from(root),
	}
	.exec()
	.map(|cmd_output: ListOutput| cmd_output.files)
}

/// compile a cairo file, returning a truple
/// (path_to_original_code, path_to_compiled_code, entrypoints)
/// # Examples
///
/// Basic usage:
///
/// ```ignore
/// let plain_path = PathBuf::from("path_to_your_program");
/// let compiled_path = compile(&plain_path);
/// let entrypoints = list_test_entrypoints(compiled_path);
///
/// assert_eq!(
/// 	compile_and_list_entrypoints(plain_path),
/// 	(plain_path, compiled_path, entrypoints)
/// )
/// ```
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
/// this function will take a program and an entrypoint name, will search for this entrypoint and
/// execute the selected test.
/// It will then return a TestResult, representing the output of the test.
///
/// # Examples
///
/// Basic usage:
///
/// ```ignore
/// //assuming your program have a "test_function1" and "test_function2" functions.
///
/// let mut hint_processor = setup_hint_processor();
/// let plain_path = PathBuf::from("path_to_your_program");
/// let compiled_path = compile(&plain_path)?;
/// let program = deserialize_program_json(&compiled_path)?;
///
/// test_single_entrypoint(program, "test_function1", hint_processor, None);
/// test_single_entrypoint(program, "test_function2", hint_processor, None);
/// ```
pub(crate) fn test_single_entrypoint(
	program: ProgramJson,
	test_entrypoint: String,
	hint_processor: &mut BuiltinHintProcessor,
	hooks: Option<Hooks>,
) -> Result<TestResult, TestCommandError> {
	let start = Instant::now();
	let mut output = String::new();
	let execution_uuid = Uuid::new_v4();
	init_buffer(execution_uuid);

	let program = Program::from_json(program, Some(&test_entrypoint))?;

	let res_cairo_run = cairo_run(program, hint_processor, execution_uuid, hooks);
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
		Err(e) => Err(TestCommandError::CairoRunError(e))?,
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
///
/// # Examples
///
/// Basic usage:
///
/// ```ignore
/// //assuming your program have a "test_function1" and "test_function2" functions.
///
/// let mut hint_processor = setup_hint_processor();
/// let plain_path = PathBuf::from("path_to_your_program");
/// let compiled_path = compile(&plain_path)?;
/// let hooks = setup_hooks();
///
/// run_test_for_one_file(hint_processor, plain_path, compiled_path, vec!("test_function1",
/// "test_fuction2"), hooks);
/// ```
fn run_tests_for_one_file(
	hint_processor: &mut BuiltinHintProcessor,
	path_to_original: PathBuf,
	path_to_compiled: PathBuf,
	test_entrypoints: Vec<String>,
	hooks: Hooks,
) -> Result<TestResult, TestCommandError> {
	let file = fs::File::open(&path_to_compiled).unwrap();
	let reader = io::BufReader::new(file);
	let program_json = deserialize_program_json(reader)?;

	let output = format!("Running tests in file {}\n", path_to_original.display());
	let res = test_entrypoints
		.into_iter()
		.map(|test_entrypoint| {
			test_single_entrypoint(
				program_json.clone(),
				test_entrypoint,
				hint_processor,
				Some(hooks.clone()),
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

		list_cairo_files(&self.root)?
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
