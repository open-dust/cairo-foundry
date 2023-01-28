use crate::cli::commands::{test::TestArgs, CommandExecution};
use cairo_rs::serde::deserialize_program::deserialize_program_json;
use std::{fs::File, io::BufReader, path::PathBuf};

use super::{
	compile_and_list_entrypoints, setup_hint_processor, setup_hooks, test_single_entrypoint,
	TestCommandError, TestResult,
};

pub fn run_single_test(
	test_name: &str,
	test_path: &PathBuf,
	max_steps: u64,
) -> Result<TestResult, TestCommandError> {
	let (_, path_to_compiled, _) = compile_and_list_entrypoints(test_path.to_owned())?;
	let file = File::open(path_to_compiled).unwrap();
	let reader = BufReader::new(file);
	let program_json = deserialize_program_json(reader)?;

	test_single_entrypoint(
		program_json,
		test_name,
		&mut setup_hint_processor(),
		Some(setup_hooks()),
		max_steps,
	)
}

#[test]
fn test_cairo_contracts() {
	TestArgs {
		root: PathBuf::from("./test_cairo_contracts"),
		max_steps: 1000000,
	}
	.exec()
	.unwrap();
}
