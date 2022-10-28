#[cfg(test)]
mod tests;

use regex::Regex;

use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::{
	BuiltinHintProcessor, HintFunc,
};
use clap::{Args, ValueHint};
use colored::Colorize;
use rayon::prelude::*;
use serde::Serialize;
use serde_json::Value;
use std::{fmt::Display, fs, path::PathBuf};
use uuid::Uuid;

use super::{
	list::{path_is_valid_directory, ListArgs, ListOutput},
	CommandExecution,
};

use crate::{
	cairo_run::cairo_run,
	compile::compile,
	hints::{clear_buffer, get_buffer, greater_than, init_buffer},
};

#[derive(Args, Debug)]
pub struct TestArgs {
	/// Path to a cairo directory
	#[clap(short, long, value_hint=ValueHint::DirPath, value_parser=path_is_valid_directory, default_value="./")]
	pub root: PathBuf,
}

fn list_test_entrypoints(compiled_path: &PathBuf) -> Result<Vec<String>, String> {
	let re = Regex::new(r"__main__.(test_\w+)$").expect("Should be a valid regex");
	let data =
		fs::read_to_string(compiled_path).map_err(|err| format!("File does not exist: {}", err))?;
	let json = serde_json::from_str::<Value>(&data)
		.map_err(|err| format!("Compilation output is not a valid JSON: {}", err))?;
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
	let hint = HintFunc(Box::new(greater_than));
	let mut hint_processor = BuiltinHintProcessor::new_empty();
	hint_processor.add_hint(String::from("print(ids.a > ids.b)"), hint);
	hint_processor
}

fn list_cairo_files(root: &PathBuf) -> Result<Vec<PathBuf>, String> {
	ListArgs {
		root: PathBuf::from(root),
	}
	.exec()
	.map(|cmd_output: ListOutput| cmd_output.files)
}

fn compile_and_list_entrypoints(path_to_code: PathBuf) -> Option<(PathBuf, PathBuf, Vec<String>)> {
	match compile(&path_to_code) {
		Ok(path_to_compiled) => match list_test_entrypoints(&path_to_compiled) {
			Ok(entrypoints) => Some((path_to_code, path_to_compiled, entrypoints)),
			Err(e) => {
				eprintln!(
					"Failed to list test entrypoints for file {}: {}",
					path_to_compiled.display(),
					e
				);
				None
			},
		},
		Err(e) => {
			eprintln!("{}", e);
			None
		},
	}
}

fn run_tests_for_one_file(
	hint_processor: &BuiltinHintProcessor,
	path_to_original: PathBuf,
	path_to_compiled: PathBuf,
	test_entrypoints: Vec<String>,
) -> String {
	let tests_output = test_entrypoints
		.into_par_iter()
		.map(|test_entrypoint| {
			let mut output = String::new();
			let execution_uuid = Uuid::new_v4();
			init_buffer(execution_uuid);
			let cairo_runner = cairo_run(
				&path_to_compiled,
				&test_entrypoint,
				false,
				hint_processor,
				execution_uuid,
			);
			let mut result = match cairo_runner {
				Ok(res) => {
					output.push_str(&format!("[{}] {}\n", "OK".green(), test_entrypoint));
					res
				},
				Err(_) => {
					output.push_str(&format!("[{}] {}\n", "FAILED".red(), test_entrypoint));
					return output
				},
			};

			// Purge the hint output buffer
			let ref_buffer = get_buffer(&execution_uuid).unwrap();
			let buffer = ref_buffer.lock().unwrap();
			if !buffer.is_empty() {
				output.push_str(&format!(
					"[{}]:\n{}",
					"captured stdout".blue(),
					String::from_utf8(buffer.to_vec()).unwrap()
				));
			}
			drop(buffer);
			clear_buffer(&execution_uuid);

			// Display the exectution output if present
			match result.get_output() {
				Ok(Some(runner_output)) =>
					if !runner_output.is_empty() {
						output.push_str(&format!(
							"[{}]:\n{}",
							"execution output".purple(),
							&runner_output
						));
					},
				Ok(None) => {}, // Cannot happen due to cairo-rs implem
				Err(e) => eprintln!("failed to get output from the cairo runner: {e}"),
			};

			output.push('\n');
			output
		})
		.reduce(String::new, |mut a, b| {
			a.push_str(&b);
			a
		});

	format!(
		"Running tests in file {}\n{}",
		path_to_original.display(),
		tests_output
	)
}

impl CommandExecution<TestOutput> for TestArgs {
	fn exec(&self) -> Result<TestOutput, String> {
		// Declare hints
		let hint_processor = setup_hint_processor();

		let output = list_cairo_files(&self.root)?
			.into_par_iter()
			.filter_map(compile_and_list_entrypoints)
			.map(|(path_to_original, path_to_compiled, test_entrypoints)| {
				run_tests_for_one_file(
					&hint_processor,
					path_to_original,
					path_to_compiled,
					test_entrypoints,
				)
			})
			.reduce(String::new, |mut a, b| {
				a.push_str(&b);
				a
			});

		Ok(TestOutput(output))
	}
}
