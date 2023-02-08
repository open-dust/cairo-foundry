#[cfg(test)]
mod tests;

#[cfg(test)]
use std::env;

use std::{
	fmt::Debug,
	fs::{read_to_string, File},
	io,
	path::PathBuf,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use serde_json::{self, Value};
use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
};

// CacheDirNotSupported is a top level struct and not an enum variant because
// it's converted elsewhere to other errors using [#from] and we want to be
// as specific as possible
#[derive(Error, Debug)]
#[error("cache directory does not exist on this platform")]
pub struct CacheDirNotSupported;

#[derive(Error, Debug)]
pub enum CacheError {
	#[error(transparent)]
	CacheDirNotSupported(#[from] CacheDirNotSupported),
	#[error("invalid contract extension {0}")]
	InvalidContractExtension(PathBuf),
	#[error(transparent)]
	StripPrefixError(#[from] std::path::StripPrefixError),
	#[error("file '{0}' has no stem")]
	StemlessFile(String),
	#[error("failed to create file '{0}': {1}")]
	FileCreation(String, io::Error),
	#[error("failed to create directory '{0}': {1}")]
	DirCreation(String, io::Error),
	#[error("failed to write to file '{0}': {1}")]
	WriteToFile(String, io::Error),
	// TODO: the value that caused the error could be long and/or contain sensitive data
	// we probably need to avoid printing it in the error
	#[error("cannot deserialize {0}: {1}")]
	DeserializeError(String, serde_json::Error),
	// TODO: add the value that couldn't be serialized to the error struct
	#[error("cannot serialize: {0}")]
	SerializeError(serde_json::Error),
	#[error("failed to read file '{0}': {1}")]
	ReadFile(String, io::Error),
}

pub const JSON_FILE_EXTENTION: &str = "json";
pub const CAIRO_FOUNDRY_CACHE_DIR: &str = "cairo-foundry-cache";
pub const CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR: &str = "compiled-cairo-files";

#[cfg(not(test))]
pub fn cache_dir() -> Result<PathBuf, CacheDirNotSupported> {
	dirs::cache_dir().ok_or(CacheDirNotSupported)
}

#[cfg(test)]
pub fn cache_dir() -> Result<PathBuf, CacheDirNotSupported> {
	Ok(env::temp_dir().join("cairo-foundry-tests"))
}

// TODO: support cashing compile errors to avoid recompiling files that failed before
#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
	pub hash: String,
	// TODO: make program_json: ProgramJson, we're not using it right now because
	// it doesn't implement Serialize
	pub program_json: Value,
}

pub fn read_cache(path: &PathBuf) -> Result<Cache, CacheError> {
	let file_content = read_to_string(path)
		.map_err(|e| CacheError::ReadFile(path.as_path().display().to_string(), e))?;
	let data = serde_json::from_str::<Cache>(file_content.as_str())
		.map_err(|e| CacheError::DeserializeError(file_content, e))?;
	Ok(data)
}

pub fn store_cache(cache: Cache, cache_path: &PathBuf) -> Result<(), CacheError> {
	// Create a file to store command output inside a json file
	let file = File::create(&cache_path)
		.map_err(|e| CacheError::FileCreation(cache_path.as_path().display().to_string(), e))?;

	serde_json::to_writer(file, &cache).map_err(|e| CacheError::SerializeError(e))
}

pub fn get_cache_path(path_to_cairo_file: &PathBuf) -> Result<PathBuf, CacheError> {
	let path_to_cache_dir = dirs::cache_dir().ok_or(CacheDirNotSupported)?;

	// Retrieve only the file name to create a clean compiled file name.
	let filename = path_to_cairo_file.file_stem().and_then(|f| f.to_str()).ok_or_else(|| {
		CacheError::StemlessFile(path_to_cairo_file.as_path().display().to_string())
	})?;

	let path_hash = hash(path_to_cairo_file);

	// Build path to save the  compiled file
	let mut cache_path = PathBuf::new();
	cache_path.push(&path_to_cache_dir);
	cache_path.push(CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR);

	std::fs::create_dir_all(&cache_path)
		.map_err(|e| CacheError::DirCreation(cache_path.as_path().display().to_string(), e))?;

	cache_path.push(format!("{}_{}", filename, path_hash.to_string()));
	cache_path.set_extension(JSON_FILE_EXTENTION);

	return Ok(cache_path)
}

fn is_valid_cairo_contract(contract_path: &PathBuf) -> Result<(), CacheError> {
	let extension = contract_path
		.extension()
		.ok_or_else(|| CacheError::InvalidContractExtension(contract_path.to_owned()))?;
	if extension != "cairo" {
		return Err(CacheError::InvalidContractExtension(
			contract_path.to_owned(),
		))
	}
	Ok(())
}

// fn get_cache_path(contract_path: &PathBuf, root_dir: &PathBuf) -> Result<PathBuf, CacheError> {
// 	// check if contract_path have .cairo extension
// 	is_valid_cairo_contract(contract_path)?;
// 	let cache_dir = dirs::cache_dir().ok_or(CacheError::CacheDirNotSupportedError)?;
// 	// get relative dir path from root_dir
// 	let contract_relative_path = contract_path.strip_prefix(root_dir)?;

// 	let mut cache_path = cache_dir.join(CAIRO_FOUNDRY_CACHE_DIR).join(contract_relative_path);
// 	cache_path.set_extension("json");
// 	Ok(cache_path)
// }

fn get_compiled_contract_path(
	contract_path: &PathBuf,
	root_dir: &PathBuf,
) -> Result<PathBuf, CacheError> {
	// check if contract_path have .cairo extension
	is_valid_cairo_contract(contract_path)?;
	let contract_relative_path = contract_path.strip_prefix(root_dir)?;
	let mut compiled_contract_path = cache_dir()?
		.join(CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR)
		.join(contract_relative_path);
	compiled_contract_path.set_extension("json");
	Ok(compiled_contract_path)
}

pub fn hash_file(path: &PathBuf) -> Result<u64, CacheError> {
	let data =
		std::fs::read(path).map_err(|e| CacheError::ReadFile(path.display().to_string(), e))?;
	Ok(hash(&data))
}

fn hash<T: Hash>(t: &T) -> u64 {
	let mut s = DefaultHasher::new();
	t.hash(&mut s);
	s.finish()
}
