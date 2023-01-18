use std::path::PathBuf;

use assert_matches::assert_matches;

use super::{get_cache_path, get_compiled_contract_path, read_cache_file, Cache, CacheError};

const CAIRO_FOUNDRY_CACHE_DIR: &str = "cairo-foundry-cache";
const CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR: &str = "compiled-cairo-files";

#[test]
fn read_cache_with_valid_input() {
	let current_dir = std::env::current_dir().unwrap();
	let cache_path = current_dir.join("test_cache_files").join("test_valid_program.json");
	let cache = read_cache_file(&cache_path).unwrap();

	let expected = Cache {
		contract_path: PathBuf::from("test_cairo_contracts/test_valid_program.cairo"),
		compiled_contract_path: PathBuf::from("test_compiled_contracts/test_valid_program.json"),
		hash: "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
	};

	assert_eq!(cache, expected);
}

#[test]
fn read_non_existing_cache_file() {
	let current_dir = std::env::current_dir().unwrap();
	let cache_path = current_dir.join("test_cache_files").join("non_existing_cache.json");
	let result = read_cache_file(&cache_path);
	assert_matches!(result, Err(CacheError::FileNotFoundError(_)));
}

#[test]
fn read_existing_cache_with_incorrect_field() {
	let current_dir = std::env::current_dir().unwrap();
	let cache_path = current_dir.join("test_cache_files").join("test_invalid_structure.json");
	let result = read_cache_file(&cache_path);
	assert_matches!(result, Err(CacheError::DeserializeError(_)));
}

#[test]
fn get_cache_path_for_valid_contract_path() {
	let current_dir = std::env::current_dir().unwrap();
	let root_dir = current_dir.join("test_cairo_contracts");

	// in test_cairo_contracts dir
	let contract_path = root_dir.join("test_valid_program_in_cairo_contracts_dir.cairo");
	let cache_path = get_cache_path(&contract_path, &root_dir).unwrap();

	let cache_dir = dirs::cache_dir().unwrap();
	let expected = cache_dir
		.join(CAIRO_FOUNDRY_CACHE_DIR)
		.join("test_valid_program_in_cairo_contracts_dir.json");

	assert_eq!(cache_path, expected);

	// in project root dir
	let contract_path = current_dir.join("test_valid_program_in_project_root_dir.cairo");
	let cache_path = get_cache_path(&contract_path, &current_dir).unwrap();

	let cache_dir = dirs::cache_dir().unwrap();
	let expected = cache_dir
		.join(CAIRO_FOUNDRY_CACHE_DIR)
		.join("test_valid_program_in_project_root_dir.json");
	assert_eq!(cache_path, expected);

	// in arbitrary path
	let arbitrary_dir = PathBuf::from("arbitrary_dir");
	let contract_path = arbitrary_dir.join("test_valid_program_in_arbitrary_path.cairo");
	let cache_path = get_cache_path(&contract_path, &arbitrary_dir).unwrap();
	let expected = cache_dir
		.join(CAIRO_FOUNDRY_CACHE_DIR)
		.join("test_valid_program_in_arbitrary_path.json");
	assert_eq!(cache_path, expected);

	// nested dir in test_cairo_contracts dir
	let root_dir = current_dir.join("test_cairo_contracts");
	let contract_path = root_dir
		.join("test_nested_dir")
		.join("test_valid_program_in_cairo_contracts_dir.cairo");
	let cache_path = get_cache_path(&contract_path, &root_dir).unwrap();

	let cache_dir = dirs::cache_dir().unwrap();
	let expected = cache_dir
		.join(CAIRO_FOUNDRY_CACHE_DIR)
		.join("test_nested_dir")
		.join("test_valid_program_in_cairo_contracts_dir.json");
	assert_eq!(cache_path, expected);
}

#[test]
fn get_cache_path_for_invalid_contract_extension() {
	let current_dir = std::env::current_dir().unwrap();
	// incorrect extension sol
	let contract_path = current_dir.join("test_invalid_extension.sol");
	let cache_path = get_cache_path(&contract_path, &current_dir);
	assert_matches!(cache_path, Err(CacheError::InvalidContractExtension(_)));

	// incorrect extension rs
	let contract_path = current_dir.join("test_invalid_extension.rs");
	let cache_path = get_cache_path(&contract_path, &current_dir);
	assert_matches!(cache_path, Err(CacheError::InvalidContractExtension(_)));

	// no extension
	let contract_path = current_dir.join("test_no_extension");
	let cache_path = get_cache_path(&contract_path, &current_dir);
	assert_matches!(cache_path, Err(CacheError::InvalidContractExtension(_)));
}

#[test]
fn get_cache_path_for_invalid_root_dir() {
	let current_dir = std::env::current_dir().unwrap();
	// incorrect rootdir
	let contract_path = PathBuf::from("test_invalid_root_dir.cairo");
	let cache_path = get_cache_path(&contract_path, &current_dir);
	assert_matches!(cache_path, Err(CacheError::StripPrefixError(_)));
}

#[test]
fn get_compiled_contract_path_for_valid_contract_path() {
	let current_dir = std::env::current_dir().unwrap();
	let root_dir = current_dir.join("test_cairo_contracts");

	let contract_path = root_dir.join("test_valid_program_in_test_cairo_contracts_dir.cairo");
	let compiled_contract_path = get_compiled_contract_path(&contract_path, &root_dir).unwrap();

	let cache_dir = dirs::cache_dir().unwrap();
	assert_eq!(
		compiled_contract_path,
		cache_dir
			.join(CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR)
			.join("test_valid_program_in_test_cairo_contracts_dir.json")
	);

	let contract_path =
		root_dir.join("test_nested_dir").join("test_valid_program_in_nested_dir.cairo");
	let compiled_contract_path = get_compiled_contract_path(&contract_path, &root_dir).unwrap();
	assert_eq!(
		compiled_contract_path,
		cache_dir
			.join(CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR)
			.join("test_nested_dir")
			.join("test_valid_program_in_nested_dir.json")
	);
}

#[test]
fn get_compiled_contract_path_for_invalid_contract_extension() {
	let current_dir = std::env::current_dir().unwrap();
	// incorrect extension sol
	let contract_path = current_dir.join("test_invalid_extension.sol");
	let cache_path = get_compiled_contract_path(&contract_path, &current_dir);
	assert_matches!(cache_path, Err(CacheError::InvalidContractExtension(_)));

	// incorrect extension rs
	let contract_path = current_dir.join("test_invalid_extension.rs");
	let cache_path = get_compiled_contract_path(&contract_path, &current_dir);
	assert_matches!(cache_path, Err(CacheError::InvalidContractExtension(_)));

	// no extension
	let contract_path = current_dir.join("test_no_extension");
	let cache_path = get_compiled_contract_path(&contract_path, &current_dir);
	assert_matches!(cache_path, Err(CacheError::InvalidContractExtension(_)));
}

#[test]
fn get_compiled_contract_path_for_invalid_root_dir() {
	let current_dir = std::env::current_dir().unwrap();
	// incorrect rootdir
	let contract_path = PathBuf::from("test_invalid_root_dir.cairo");
	let cache_path = get_compiled_contract_path(&contract_path, &current_dir);
	assert_matches!(cache_path, Err(CacheError::StripPrefixError(_)));
}
