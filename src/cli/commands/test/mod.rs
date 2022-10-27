#[cfg(test)]
mod tests;

use regex::Regex;

use cairo_rs::{
	cairo_run::cairo_run,
	hint_processor::{
		builtin_hint_processor::{
			builtin_hint_processor_definition::{BuiltinHintProcessor, HintFunc},
			hint_utils::get_integer_from_var_name,
		},
		hint_processor_definition::HintReference,
		proxies::{exec_scopes_proxy::ExecutionScopesProxy, vm_proxy::VMProxy},
	},
	serde::deserialize_program::ApTracking,
	vm::errors::vm_errors::VirtualMachineError,
};
use clap::{Args, ValueHint};
use colored::Colorize;
use log::error;
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, fmt::Display, fs, io::Write, path::PathBuf, str::from_utf8};

use super::{
	list::{path_is_valid_directory, ListArgs},
	CommandExecution,
};

use crate::compile::compile;

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
		let hint = HintFunc(Box::new(greater_than_hint));
		let mut hint_processor = BuiltinHintProcessor::new_empty();
		hint_processor.add_hint(String::from("print(ids.a > ids.b)"), hint);

		let list_of_cairo_files = ListArgs {
			root: PathBuf::from(&self.root),
		}
		.exec();

		let iterator_over_compiled_cairo_files =
			list_of_cairo_files?.files.into_iter().map(|path| {
				let res_compilation = compile(&path);
				(path, res_compilation)
			});

		let entrypoints_by_file: Vec<(PathBuf, PathBuf, Vec<String>)> =
			iterator_over_compiled_cairo_files
				.into_iter()
				.filter_map(
					|(path_to_original, res_compilation)| match res_compilation {
						Ok(path_to_compiled) => {
							let entrypoints = list_test_entrypoints(&path_to_compiled);
							match entrypoints {
								Ok(entrypoints) =>
									Some((path_to_original, path_to_compiled, entrypoints)),
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
					},
				)
				.collect();

		// run each test entrypoint
		for (path_to_original, path_to_compiled, test_entrypoints) in entrypoints_by_file {
			println!("\nRunning tests in file {}", path_to_original.display());
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

				if let Some(runner_output) = result
					.get_output()
					.map_err(|e| format!("failed to get output from the cairo runner: {e}"))?
				{
					if !runner_output.is_empty() {
						println!("[{}]", "execution output".blue());
						println!("{runner_output}",);
					}
				}
			}
		}
		Ok(Default::default())
	}
}

fn greater_than_hint(
	vm_proxy: &mut VMProxy,
	_exec_scopes_proxy: &mut ExecutionScopesProxy,
	ids_data: &HashMap<String, HintReference>,
	ap_tracking: &ApTracking,
) -> Result<(), VirtualMachineError> {
	let a = get_integer_from_var_name("a", vm_proxy, ids_data, ap_tracking)?;
	let b = get_integer_from_var_name("b", vm_proxy, ids_data, ap_tracking)?;
	println!("{}", a > b);
	Ok(())
}
