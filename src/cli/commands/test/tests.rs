use crate::cli::commands::{test::TestArgs, CommandExecution};
use std::path::PathBuf;

use super::{compile_and_list_entrypoints, setup_hint_processor, test_single_entrypoint};

pub fn run_single_test(test_name: &str, test_path: &PathBuf) -> (String, bool) {
	let (_, path_to_compiled, _) = compile_and_list_entrypoints(test_path.to_owned()).unwrap();
	test_single_entrypoint(
		&path_to_compiled,
		test_name.to_string(),
		&setup_hint_processor(),
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
