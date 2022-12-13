use crate::cli::commands::{test::{TestArgs, cache::CacheJson}, CommandExecution};
use std::path::PathBuf;

use super::{compile_and_list_entrypoints, setup_hint_processor, test_single_entrypoint};

use crate::cli::commands::test::cache::{create_compiled_contract_path, Error, read_json_file};

pub fn run_single_test(test_name: &str, test_path: &PathBuf) -> (String, bool) {
	let (_, path_to_compiled, _) = compile_and_list_entrypoints(test_path.to_owned()).unwrap();
	test_single_entrypoint(
		&path_to_compiled,
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
fn test_create_compiled_contract_path_positive_0() {
	let current_dir = std::env::current_dir().unwrap();
	let root = PathBuf::from(current_dir.join("test_cairo_contracts"));
	
	let path_to_contract_file = PathBuf::from(root.join("test_valid_program.cairo"));
	let path_to_compiled_contract_path = create_compiled_contract_path(&path_to_contract_file, &root).unwrap();
	let cache_dir = dirs::cache_dir().ok_or(Error::CacheDirSupported).unwrap();
	assert_eq!(path_to_compiled_contract_path, cache_dir.join("compiled-cairo-files/test_cairo_contracts/test_valid_program.json"));
}

#[test]
fn test_create_compiled_contract_path_positive_1() {
	let current_dir = std::env::current_dir().unwrap();
	let root = PathBuf::from(current_dir.join("test_cairo_contracts"));
	let path_to_contract_file = PathBuf::from(root.join("test_valid_program.cairo"));
	let path_to_compiled_contract_path = create_compiled_contract_path(&path_to_contract_file, &root).unwrap();
	let cache_dir = dirs::cache_dir().ok_or(Error::CacheDirSupported).unwrap();
	assert_eq!(path_to_compiled_contract_path, cache_dir.join("compiled-cairo-files/test_cairo_contracts/test_valid_program.json"));
}

#[test]
fn test_read_json_positive_0() {
	let current_dir = std::env::current_dir().unwrap();
	let root = PathBuf::from(current_dir.join("test_compiled_contracts"));
	let path_to_compiled_contract_path = PathBuf::from(root.join("test_valid_program.json"));
	let json = read_json_file(&path_to_compiled_contract_path).unwrap();

	let expected_json = CacheJson {
		contract_path: "test_compiled_contracts/test_valid_program.cairo".to_string(),
		sha256: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
	};

	assert_eq!(json, expected_json);
}