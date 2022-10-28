#[cfg(test)]
mod tests;

use std::{fmt::Display, io::Write, path::PathBuf, str, str::from_utf8};

use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::{
	BuiltinHintProcessor, HintFunc,
};
use clap::{Args, ValueHint};
use colored::Colorize;
use log::error;
use serde::Serialize;
use uuid::Uuid;

use super::CommandExecution;

use crate::{
	cairo_run::cairo_run,
	compile::compile,
	hints::{clear_buffer, get_buffer, greater_than, init_buffer},
};

#[derive(Args, Debug)]
pub struct ExecuteArgs {
	/// Path to a cairo program
	#[clap(short, long, value_hint=ValueHint::FilePath, value_parser=is_cairo)]
	pub program: PathBuf,
}

fn is_cairo(path: &str) -> Result<PathBuf, String> {
	let path = PathBuf::from(path);
	if path.exists() && path.is_file() {
		match path.extension() {
			Some(ext) if ext == "cairo" => Ok(path),
			_ => Err(format!("\"{}\" is not a cairo file", path.display())),
		}
	} else {
		Err(format!("\"{}\" is not a valid file", path.display()))
	}
}

/// Execute command output
#[derive(Debug, Serialize)]
pub struct ExecuteOutput(Vec<u8>);

impl Write for ExecuteOutput {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.0.write(buf)
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.0.flush()
	}
}

impl Display for ExecuteOutput {
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

impl CommandExecution<ExecuteOutput> for ExecuteArgs {
	fn exec(&self) -> Result<ExecuteOutput, String> {
		let hint = HintFunc(Box::new(greater_than));
		let mut hint_processor = BuiltinHintProcessor::new_empty();
		hint_processor.add_hint(String::from("print(ids.a > ids.b)"), hint);

		// Call the compile function
		let compiled_program_path = compile(&self.program).map_err(|e| e.to_string())?;

		let execution_uuid = Uuid::new_v4();
		init_buffer(execution_uuid);
		// Run the main function of cairo contract
		let mut cairo_runner = cairo_run(
			&compiled_program_path,
			"main",
			false,
			&hint_processor,
			Default::default(),
		)
		.map_err(|e| {
			format!(
				"failed to run the program \"{}\": {}",
				self.program.display(),
				e,
			)
		})?;

		let mut output = ExecuteOutput(vec![]);

		let buffer = get_buffer(&execution_uuid).unwrap();
		if !buffer.is_empty() {
			output
				.write_all(format!("[{}]:\n{}", "captured stdout".blue(), buffer).as_bytes())
				.map_err(|e| {
					format!(
						"failed to print the program hints output \"{}\": {}",
						compiled_program_path.display(),
						e,
					)
				})?;
		}
		clear_buffer(&execution_uuid);

		cairo_runner.write_output(&mut output).map_err(|e| {
			format!(
				"failed to print the program output \"{}\": {}",
				compiled_program_path.display(),
				e,
			)
		})?;

		Ok(output)
	}
}
