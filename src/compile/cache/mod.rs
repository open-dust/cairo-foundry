#[cfg(test)]
mod tests;

#[cfg(test)]
use std::env;

use std::{
	fmt::Debug,
	fs::{self, read_to_string, File},
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
pub struct CompileCacheItem {
	pub hash: u64,
	// TODO: make program_json: ProgramJson, we're not using it right now because
	// it doesn't implement Serialize
	pub program_json: Value,
}

impl PartialEq for CompileCacheItem {
	fn eq(&self, other: &Self) -> bool {
		self.hash == other.hash
	}
}

impl CompileCacheItem {
	pub fn read(path: &PathBuf) -> Result<CompileCacheItem, CacheError> {
		let file_content = read_to_string(path)
			.map_err(|e| CacheError::ReadFile(path.as_path().display().to_string(), e))?;

		let data = serde_json::from_str::<CompileCacheItem>(file_content.as_str())
			.map_err(|e| CacheError::DeserializeError(file_content, e))?;
		Ok(data)
	}

	pub fn write(&self, cache_path: &PathBuf) -> Result<(), CacheError> {
		let file = File::create(cache_path)
			.map_err(|e| CacheError::FileCreation(cache_path.as_path().display().to_string(), e))?;

		serde_json::to_writer(file, self).map_err(CacheError::SerializeError)
	}
}

pub fn get_compile_cache_path(path_to_cairo_file: &PathBuf) -> Result<PathBuf, CacheError> {
	// Retrieve only the file name to create a clean compiled file name.
	let filename = path_to_cairo_file.file_stem().and_then(|f| f.to_str()).ok_or_else(|| {
		CacheError::StemlessFile(path_to_cairo_file.as_path().display().to_string())
	})?;

	let path_hash = hash(path_to_cairo_file);

	// Build path to save the  compiled file
	let mut cache_path = cache_dir()?;
	cache_path.push(CAIRO_FOUNDRY_CACHE_DIR);

	fs::create_dir_all(&cache_path)
		.map_err(|e| CacheError::DirCreation(cache_path.as_path().display().to_string(), e))?;

	cache_path.push(format!("{filename}_{path_hash}"));
	cache_path.set_extension(JSON_FILE_EXTENTION);

	Ok(cache_path)
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
