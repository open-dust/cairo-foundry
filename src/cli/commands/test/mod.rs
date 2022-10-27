#[cfg(test)]
mod tests;

use regex::Regex;

use cairo_rs::{
	cairo_run::cairo_run,
	hint_processor::builtin_hint_processor::builtin_hint_processor_definition::{
		BuiltinHintProcessor, HintFunc,
	},
};
use clap::{Args, ValueHint};
use colored::Colorize;
use log::error;
use serde::Serialize;
use serde_json::Value;
use std::{fmt::Display, fs, io::Write, path::PathBuf, str::from_utf8};

use super::{
	list::{path_is_valid_directory, ListArgs},
	CommandExecution,
};

use crate::{
	compile::compile,
	hints::{greater_than, HINT_OUTPUT_BUFFER},
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
pub struct TestOutput(Vec<u8>);

impl Write for TestOutput {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.0.write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.0.flush()
	}
}

impl Display for TestOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			from_utf8(&self.0).map_err(|e| {
				error!("failed to format the execution output due to invalid utf8 encodig: {e}");
				std::fmt::Error
			})?
		)
	}
}

impl CommandExecution<TestOutput> for TestArgs {
	fn exec(&self) -> Result<TestOutput, String> {
		// Declare hints
		let hint = HintFunc(Box::new(greater_than));
		let mut hint_processor = BuiltinHintProcessor::new_empty();
		hint_processor.add_hint(String::from("print(ids.a > ids.b)"), hint);

		// Recursively list cairo files
		let list_of_cairo_files = ListArgs {
			root: PathBuf::from(&self.root),
		}
		.exec();

		// Try to compile those files
		let iterator_over_compiled_cairo_files =
			list_of_cairo_files?.files.into_iter().map(|path| {
				let res_compilation = compile(&path);
				(path, res_compilation)
			});

		// List test entrypoints for each compiled file
		// (path_to_cairo_file, path_to_compiled_file, list of entrypoints)
		let entrypoints_by_file: Vec<(PathBuf, PathBuf, Vec<String>)> =
			iterator_over_compiled_cairo_files
				.into_iter()
				.filter_map(|(path_to_code, res_compilation)| match res_compilation {
					Ok(path_to_compiled) => {
						let entrypoints = list_test_entrypoints(&path_to_compiled);
						match entrypoints {
							Ok(entrypoints) => Some((path_to_code, path_to_compiled, entrypoints)),
							Err(e) => {
								eprintln!(
									"Failed to list test entrypoints for file {}: {}",
									path_to_compiled.display(),
									e
								);
								None
							},
						}
					},
					Err(e) => {
						eprintln!("Compilation output is not a valid JSON: {}", e);
						None
					},
				})
				.collect();

		// Run each test
		for (path_to_original, path_to_compiled, test_entrypoints) in entrypoints_by_file {
			println!("Running tests in file {}", path_to_original.display());
			for test_entrypoint in test_entrypoints {
				let cairo_runner =
					cairo_run(&path_to_compiled, &test_entrypoint, false, &hint_processor);
				let mut result = match cairo_runner {
					Ok(res) => {
						println!("[{}] {}", "OK".green(), test_entrypoint);
						res
					},
					Err(_) => {
						println!("[{}] {}", "FAILED".red(), test_entrypoint);
						continue
					},
				};

				// Purge the hint output buffer
				let mut hint_output_buffer = HINT_OUTPUT_BUFFER.lock().unwrap();
				if !hint_output_buffer.buffer().is_empty() {
					println!("[{}]:", "captured stdout".blue());
					hint_output_buffer.flush().unwrap();
				}
				drop(hint_output_buffer);
				println!();

				// Display the exectution output if present
				if let Some(runner_output) = result
					.get_output()
					.map_err(|e| format!("failed to get output from the cairo runner: {e}"))?
				{
					if !runner_output.is_empty() {
						println!("[{}]:", "execution output".purple());
						println!("{runner_output}",);
					}
				}
			}
		}

		Ok(Default::default())
	}
}
