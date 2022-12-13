use crate::cli::commands::{test::TestArgs, CommandExecution};
use cairo_rs::serde::deserialize_program::deserialize_program_json;
use std::path::PathBuf;

// import io::Error;
use std::{io::{self}};


use super::{
	compile_and_list_entrypoints, setup_hint_processor, test_single_entrypoint, TestCommandError,
};

use core::fmt::Error;

use crate::cli::commands::test::cache::cache::{read_json_file, CacheError, CacheJson};
// use serde_json::Error;



pub fn run_single_test(
	test_name: &str,
	test_path: &PathBuf,
) -> Result<(String, bool), TestCommandError> {
	let (_, path_to_compiled, _) = compile_and_list_entrypoints(test_path.to_owned())?;

	let program_json = deserialize_program_json(&path_to_compiled)?;

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

#[test]
fn test_read_json_positive_0() {
	let current_dir = std::env::current_dir().unwrap();
	let root = PathBuf::from(current_dir.join("test_compiled_contracts"));
	let path_to_compiled_contract_path = PathBuf::from(root.join("test_valid_program.json"));
	let json = read_json_file(&path_to_compiled_contract_path).unwrap();

	let expected_json = CacheJson {
		contract_path: "test_compiled_contracts/test_valid_program.cairo".to_string(),
		hash: "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
	};

	assert_eq!(json, expected_json);
}

#[test]
fn test_read_json_negative_0() {
	let current_dir = std::env::current_dir().unwrap();
	let root = PathBuf::from(current_dir.join("test_compiled_contracts"));
	let path_to_compiled_contract_path = PathBuf::from(root.join("test_inexistent.json"));
	let result = read_json_file(&path_to_compiled_contract_path).is_err();
	assert!(result);
	// TODO: test error type
}

#[test]
fn test_read_json_negative_1() {
	let current_dir = std::env::current_dir().unwrap();
	let root = PathBuf::from(current_dir.join("test_compiled_contracts"));
	let path_to_compiled_contract_path = PathBuf::from(root.join("test_invalid_structure.json"));
	let result = read_json_file(&path_to_compiled_contract_path).is_err();
	assert!(result);
	// TODO: test error type
}