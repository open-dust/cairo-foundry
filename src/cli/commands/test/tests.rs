use crate::cli::commands::{test::TestArgs, CommandExecution};
use cairo_rs::serde::deserialize_program::deserialize_program_json;
use std::path::PathBuf;
use colored::Colorize;

use super::{compile_and_list_entrypoints, setup_hint_processor, test_single_entrypoint};

pub fn run_single_test(test_name: &str, test_path: &PathBuf) -> (String, bool) {
	let (_, path_to_compiled, _) = compile_and_list_entrypoints(test_path.to_owned()).unwrap();

	let program_json = match deserialize_program_json(&path_to_compiled) {
		Ok(program_json) => program_json,
		Err(e) => {
			let output = format!(
				"[{}] {}\nError: failed to deserialize program",
				"FAILED".red(),
				e
			);
			return (output, false);
		},
	};

	test_single_entrypoint(
		program_json,
		test_name.to_string(),
		&setup_hint_processor(),
		None,
	)
}

#[test]
fn test_cairo_contracts() {
	TestArgs {
		root: PathBuf::from("./test_cairo_contracts"),
	}
	.exec()
	.unwrap();
}

#[test]
fn test_cairo_hints() {
	TestArgs {
		root: PathBuf::from("./test_cairo_hints"),
	}
	.exec()
	.unwrap();
}
