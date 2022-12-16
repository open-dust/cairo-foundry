pub mod cache {
	use dirs;
	use std::{fmt::Debug, fs::read_to_string, io, path::PathBuf};

	use thiserror::Error;

	use serde::{Deserialize, Serialize};
	use serde_json;

	use crate::compile::Error;

	#[derive(Error, Debug)]
	pub enum CacheError {
		#[error(transparent)]
		FileNotFoundError(#[from] io::Error),
		#[error(transparent)]
		DeserializeError(#[from] serde_json::Error),
		#[error("cache directory does not exist on this platform")]
		CacheDirNotSupportedError,
		#[error("filename does not exist")]
		FileNameDoesNotExistError,
		#[error("file extension does not exist")]
		ExtensionDoesNotExistError,
	}

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	pub struct Cache {
		pub contract_path: PathBuf,
		pub compiled_contract_path: PathBuf,
		pub hash: String,
	}

	pub fn get_cache_path(contract_path: &PathBuf) -> Result<PathBuf, CacheError> {
		// check if contract_path have .cairo extension
		let extension = contract_path.extension().ok_or(CacheError::ExtensionDoesNotExistError)?;
		// assert extension to be cairo
		if extension != "cairo" {
			return Err(CacheError::ExtensionDoesNotExistError)
		}
		assert_eq!(extension, "cairo");
		let cache_dir = dirs::cache_dir().ok_or(CacheError::CacheDirNotSupportedError)?;
		let contract_name =
			contract_path.file_stem().ok_or(CacheError::FileNameDoesNotExistError)?;

		let mut cache_path =
			PathBuf::from(cache_dir.join("cairo-foundry-cache").join(contract_name));
		cache_path.set_extension("json");
		return Ok(cache_path)
	}

	fn get_compiled_contract_path(contract_path: &PathBuf) -> Result<PathBuf, CacheError> {
		let cache_dir = dirs::cache_dir().ok_or(CacheError::CacheDirNotSupportedError)?;
		let contract_name =
			contract_path.file_stem().ok_or(CacheError::FileNameDoesNotExistError)?;
		let mut compiled_contract_path =
			PathBuf::from(cache_dir.join("compiled-cairo-files").join(contract_name));
		compiled_contract_path.set_extension("json");
		return Ok(compiled_contract_path)
	}

	pub fn read_cache_file(path: &PathBuf) -> Result<Cache, CacheError> {
		let file = read_to_string(path)?;
		let data = serde_json::from_str::<Cache>(file.as_str())?;
		return Ok(data)
	}
}
