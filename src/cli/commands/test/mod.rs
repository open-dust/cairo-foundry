#[cfg(test)]
pub mod tests;
pub mod cache;

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
use rayon::prelude::*;
use serde::Serialize;
use serde_json::Value;
use std::{fmt::Display, fs, io, path::PathBuf, sync::Arc, time::Instant};
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

#[derive(Args, Debug)]
pub struct TestArgs {
	/// Path to a cairo directory
	#[clap(short, long, value_hint=ValueHint::DirPath, value_parser=path_is_valid_directory, default_value="./")]
	pub root: PathBuf,
}

pub struct TestResult {
	pub output: String,
	pub success: bool,
}

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
	let skip_hint = HintFunc(Box::new(hints::skip));
	let mock_call_hint = HintFunc(Box::new(hints::mock_call));
	let expect_revert_hint = HintFunc(Box::new(expect_revert));
	let mut hint_processor = BuiltinHintProcessor::new_empty();
	hint_processor.add_hint(String::from("skip()"), skip_hint);
	hint_processor.add_hint(String::from("expect_revert()"), expect_revert_hint);
	hint_processor.add_hint(
		String::from("mock_call(func_to_mock, mock_ret_value)"),
		mock_call_hint,
	);
	hint_processor
}

fn setup_hooks() -> Hooks {
	Hooks::new(Arc::new(hooks::pre_step_instruction))
}

fn list_cairo_files(root: &PathBuf) -> Result<Vec<PathBuf>, ListCommandError> {
	ListArgs {
		root: PathBuf::from(root),
	}
	.exec()
	.map(|cmd_output: ListOutput| cmd_output.files)
}

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

pub(crate) fn test_single_entrypoint(
	program: ProgramJson,
	test_entrypoint: String,
	hint_processor: &BuiltinHintProcessor,
	hooks: Option<Hooks>,
) -> Result<(String, bool), TestCommandError> {
	let start = Instant::now();
	let mut output = String::new();
	let execution_uuid = Uuid::new_v4();
	init_buffer(execution_uuid);

	let program = Program::from_json(program, &test_entrypoint)?;

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
			(Some(res), true)
		},
		Err(CairoRunError::VirtualMachine(VirtualMachineError::CustomHint(
			custom_error_message,
		))) if custom_error_message == "skip" => {
			output.push_str(&format!("[{}] {}\n", "SKIPPED".yellow(), test_entrypoint,));
			(None, true)
		},
		Err(CairoRunError::VirtualMachine(VirtualMachineError::CustomHint(
			custom_error_message,
		))) if custom_error_message == EXPECT_REVERT_FLAG => {
			output.push_str(&format!(
				"[{}] {}\nError: execution did not revert while expect_revert() was specified\n\n",
				"FAILED".red(),
				test_entrypoint,
			));
			(None, false)
		},
		Err(e) => Err(TestCommandError::CairoRunError(e))?,
	};

	purge_hint_buffer(&execution_uuid, &mut output);
	let (mut runner, mut vm) = match opt_runner_and_output {
		Some(runner_and_vm) => runner_and_vm,
		None => return Ok((output, test_success)),
	};

	// Display the execution output if present
	match runner.get_output(&mut vm) {
		Ok(runner_output) =>
			if !runner_output.is_empty() {
				output.push_str(&format!(
					"[{}]:\n{}",
					"execution output".purple(),
					&runner_output
				));
			},
		Err(e) => eprintln!("failed to get output from the cairo runner: {e}"),
	};

	output.push('\n');
	Ok((output, test_success))
}

fn run_tests_for_one_file(
	hint_processor: &BuiltinHintProcessor,
	path_to_original: PathBuf,
	path_to_compiled: PathBuf,
	test_entrypoints: Vec<String>,
	hooks: Hooks,
) -> Result<TestResult, TestCommandError> {
	let program_json = deserialize_program_json(&path_to_compiled)?;

	let (tests_output, tests_success) = test_entrypoints
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
		.fold((String::new(), true), |mut a, b| {
			a.0.push_str(&b.0);
			a.1 &= b.1;
			a
		});

	Ok(TestResult {
		output: format!(
			"Running tests in file {}\n{}",
			path_to_original.display(),
			tests_output
		),
		success: tests_success,
	})
}

impl CommandExecution<TestOutput, TestCommandError> for TestArgs {
	fn exec(&self) -> Result<TestOutput, TestCommandError> {
		// Declare hints
		let hint_processor = setup_hint_processor();
		let hooks = setup_hooks();

		list_cairo_files(&self.root)?
			.into_par_iter()
			.map(compile_and_list_entrypoints)
			.map(|res| -> Result<TestResult, TestCommandError> {
				match res {
					Ok((path_to_original, path_to_compiled, test_entrypoints)) =>
						run_tests_for_one_file(
							&hint_processor,
							path_to_original,
							path_to_compiled,
							test_entrypoints,
							hooks.clone(),
						),
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
