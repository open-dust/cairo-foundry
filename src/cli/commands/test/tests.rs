use crate::cli::commands::{test::TestArgs, CommandExecution};
use cairo_rs::serde::deserialize_program::{deserialize_program, deserialize_program_json};
use std::{fs::File, io::BufReader, path::PathBuf};

use super::{
	compile, compile_and_list_entrypoints, retrieve_return_signatures, setup_hint_processor,
	setup_hooks, test_single_entrypoint, TestCommandError, TestResult,
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

#[test]
fn get_function_return_signatures() {
	let current_dir = std::env::current_dir().unwrap();
	let root_dir = current_dir.join("test_cairo_contracts");

	let contract_path = root_dir.join("test_retrieve_function_signatures.cairo");
	let compiled_contract_path = compile(&contract_path).unwrap();
	let file = File::open(&compiled_contract_path).unwrap();
	let reader = BufReader::new(file);
	let program = deserialize_program(reader, None).unwrap();
	let signatures = retrieve_return_signatures(&program);

	assert_eq!(signatures["__main__.array_sum"], ["felt"]);
	assert_eq!(signatures["__main__.test_array_sum_negative"], [""; 0]);
	assert_eq!(
		signatures["__main__.get_account"],
		[
			"__main__.Account",
			"starkware.cairo.common.uint256.Uint256",
			"felt"
		]
	);
}
