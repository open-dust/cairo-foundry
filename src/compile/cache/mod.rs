#[cfg(test)]
mod tests;

#[cfg(test)]
use std::env;

use std::{fmt::Debug, fs::read_to_string, io, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json;
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Cache {
	pub contract_path: PathBuf,
	pub compiled_contract_path: PathBuf,
	pub hash: String,
}

// CacheDirNotSupported is a top level struct and not an enum variant because
// it's converted elsewhere to other errors using [#from] and we want to be
// as specific as possible
#[derive(Error, Debug)]
#[error("cache directory does not exist on this platform")]
pub struct CacheDirNotSupported;

#[derive(Error, Debug)]
pub enum CacheError {
	#[error(transparent)]
	FileNotFoundError(#[from] io::Error),
	#[error(transparent)]
	DeserializeError(#[from] serde_json::Error),
	#[error(transparent)]
	CacheDirNotSupportedError(#[from] CacheDirNotSupported),
	#[error("filename does not exist")]
	InvalidContractExtension(PathBuf),
	#[error(transparent)]
	StripPrefixError(#[from] std::path::StripPrefixError),
}

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

fn read_cache_file(path: &PathBuf) -> Result<Cache, CacheError> {
	let file = read_to_string(path)?;
	let data = serde_json::from_str::<Cache>(file.as_str())?;
	Ok(data)
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

fn get_cache_path(contract_path: &PathBuf, root_dir: &PathBuf) -> Result<PathBuf, CacheError> {
	// check if contract_path have .cairo extension
	is_valid_cairo_contract(contract_path)?;
	// get relative dir path from root_dir
	let contract_relative_path = contract_path.strip_prefix(root_dir)?;

	let mut cache_path = cache_dir()?.join(CAIRO_FOUNDRY_CACHE_DIR).join(contract_relative_path);
	cache_path.set_extension("json");
	Ok(cache_path)
}

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
