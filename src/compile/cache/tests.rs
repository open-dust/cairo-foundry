use std::{fs, path::PathBuf};

use assert_matches::assert_matches;

use super::{
	cache_dir, get_cache_path, get_compiled_contract_path, CacheError, CompileCacheItem,
	CAIRO_FOUNDRY_CACHE_DIR, CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR,
};

#[test]
fn read_cache_with_valid_input() {
	let current_dir = std::env::current_dir().unwrap();
	let cache_path = current_dir.join("test_cache_files").join("test_valid_program.json");
	let found_cache = CompileCacheItem::read(&cache_path).unwrap();

	let expected_cache = CompileCacheItem {
		program_json: "".into(),
		hash: 10,
	};

	assert_eq!(found_cache, expected_cache);
}

#[test]
fn read_non_existing_cache_file() {
	let current_dir = std::env::current_dir().unwrap();
	let cache_path = current_dir.join("test_cache_files").join("non_existing_cache.json");
	let result = CompileCacheItem::read(&cache_path);
	assert_matches!(result, Err(CacheError::ReadFile(_, _)));
}

#[test]
fn read_existing_cache_with_incorrect_field() {
	let current_dir = std::env::current_dir().unwrap();
	let cache_path = current_dir.join("test_cache_files").join("test_invalid_structure.json");
	let result = CompileCacheItem::read(&cache_path);
	assert_matches!(result, Err(CacheError::DeserializeError(_, _)));
}

#[test]
fn write_cache_with_valid_input() -> Result<(), CacheError> {
	let cache_dir = std::env::temp_dir().join("cairo_foundry_test");

	fs::create_dir_all(&cache_dir)
		.map_err(|e| CacheError::DirCreation(cache_dir.as_path().display().to_string(), e))?;

	let cache_path = cache_dir.join("write_cache_with_valid_input.json");

	let expected_cache = CompileCacheItem {
		program_json: "".into(),
		hash: 10,
	};

	CompileCacheItem::write(&expected_cache, &cache_path)?;

	let found_cache = CompileCacheItem::read(&cache_path)?;

	assert_eq!(found_cache, expected_cache);

	Ok(())
}

#[test]
fn update_cache_with_valid_input() -> Result<(), CacheError> {
	let cache_dir = std::env::temp_dir().join("cairo_foundry_test");

	fs::create_dir_all(&cache_dir)
		.map_err(|e| CacheError::DirCreation(cache_dir.as_path().display().to_string(), e))?;

	let cache_path = cache_dir.join("update_cache_with_valid_input.json");

	let expected_cache = CompileCacheItem {
		program_json: "".into(),
		hash: 10,
	};

	CompileCacheItem::write(&expected_cache, &cache_path)?;

	let found_cache = CompileCacheItem::read(&cache_path)?;

	assert_eq!(found_cache, expected_cache);

	let expected_cache = CompileCacheItem {
		program_json: "updated".into(),
		hash: 20,
	};

	CompileCacheItem::write(&expected_cache, &cache_path)?;

	let found_cache = CompileCacheItem::read(&cache_path)?;

	assert_eq!(found_cache, expected_cache);

	Ok(())
}

#[test]
fn get_cache_path_for_valid_contract_path() -> Result<(), CacheError> {
	let current_dir = std::env::current_dir().unwrap();
	let root_dir = current_dir.join("test_cairo_contracts");

	// in test_cairo_contracts dir
	let contract_path = root_dir.join("test_valid_program_in_cairo_contracts_dir.cairo");
	let cache_path = get_cache_path(&contract_path, &root_dir).unwrap();

	let expected_cache_path = cache_dir()
		.unwrap()
		.join(CAIRO_FOUNDRY_CACHE_DIR)
		.join("test_valid_program_in_cairo_contracts_dir.json");

	assert_eq!(cache_path, expected_cache_path);

	// in project root dir
	let contract_path = current_dir.join("test_valid_program_in_project_root_dir.cairo");
	let cache_path = get_cache_path(&contract_path, &current_dir).unwrap();

	let expected_cache_path = cache_dir()
		.unwrap()
		.join(CAIRO_FOUNDRY_CACHE_DIR)
		.join("test_valid_program_in_project_root_dir.json");
	assert_eq!(cache_path, expected_cache_path);

	// in arbitrary path
	let arbitrary_dir = PathBuf::from("arbitrary_dir");
	let contract_path = arbitrary_dir.join("test_valid_program_in_arbitrary_path.cairo");
	let cache_path = get_cache_path(&contract_path, &arbitrary_dir).unwrap();
	let expected = cache_dir()
		.unwrap()
		.join(CAIRO_FOUNDRY_CACHE_DIR)
		.join("test_valid_program_in_arbitrary_path.json");
	assert_eq!(cache_path, expected);

	// nested dir in test_cairo_contracts dir
	let root_dir = current_dir.join("test_cairo_contracts");
	let contract_path = root_dir
		.join("test_nested_dir")
		.join("test_valid_program_in_cairo_contracts_dir.cairo");
	let cache_path = get_cache_path(&contract_path, &root_dir).unwrap();

	let expected_cache_path = cache_dir()
		.unwrap()
		.join(CAIRO_FOUNDRY_CACHE_DIR)
		.join("test_nested_dir")
		.join("test_valid_program_in_cairo_contracts_dir.json");
	assert_eq!(cache_path, expected_cache_path);

	Ok(())
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

	assert_eq!(
		compiled_contract_path,
		cache_dir()
			.unwrap()
			.join(CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR)
			.join("test_valid_program_in_test_cairo_contracts_dir.json")
	);

	let contract_path =
		root_dir.join("test_nested_dir").join("test_valid_program_in_nested_dir.cairo");
	let compiled_contract_path = get_compiled_contract_path(&contract_path, &root_dir).unwrap();
	assert_eq!(
		compiled_contract_path,
		cache_dir()
			.unwrap()
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
