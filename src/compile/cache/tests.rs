use std::fs;

use assert_matches::assert_matches;

use super::{CacheError, CompileCacheItem};

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
