pub mod cache {
	use dirs;
	use std::{fmt::Debug, fs::read_to_string, io, path::PathBuf};

	use thiserror::Error;

	use serde::{Deserialize, Serialize};
	use serde_json;

	const CAIRO_FOUNDRY_CACHE_DIR: &str = "cairo-foundry-cache";
	const CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR: &str = "compiled-cairo-files";

	#[derive(Error, Debug)]
	pub enum CacheError {
		#[error(transparent)]
		FileNotFoundError(#[from] io::Error),
		#[error(transparent)]
		DeserializeError(#[from] serde_json::Error),
		#[error("cache directory does not exist on this platform")]
		CacheDirNotSupportedError,
		#[error("filename does not exist")]
		InvalidContractExtension(String),
		#[error(transparent)]
		StripPrefixError(#[from] std::path::StripPrefixError),
	}

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	pub struct Cache {
		pub contract_path: PathBuf,
		pub compiled_contract_path: PathBuf,
		pub hash: String,
	}

	fn is_valid_cairo_contract(contract_path: &PathBuf) -> Result<(), CacheError> {
		// check if contract_path have .cairo extension
		let extension = contract_path
			.extension()
			.ok_or(CacheError::InvalidContractExtension(format!(" ")))?;

		// assert extension to be cairo
		if extension != "cairo" {
			// convert osStr to string
			match extension.to_str() {
				Some(extension) =>
					return Err(CacheError::InvalidContractExtension(extension.to_string())),
				None => return Err(CacheError::InvalidContractExtension(" ".to_string())),
			}
		}
		Ok(())
	}

	pub fn get_cache_path(
		contract_path: &PathBuf,
		root_dir: &PathBuf,
	) -> Result<PathBuf, CacheError> {
		// check if contract_path have .cairo extension
		is_valid_cairo_contract(contract_path)?;
		let cache_dir = dirs::cache_dir().ok_or(CacheError::CacheDirNotSupportedError)?;
		// get relative dir path from root_dir
		let contract_relative_path = contract_path.strip_prefix(root_dir)?;

		let mut cache_path =
			PathBuf::from(cache_dir.join(CAIRO_FOUNDRY_CACHE_DIR).join(contract_relative_path));
		cache_path.set_extension("json");
		return Ok(cache_path)
	}

	fn get_compiled_contract_path(
		contract_path: &PathBuf,
		root_dir: &PathBuf,
	) -> Result<PathBuf, CacheError> {
		// check if contract_path have .cairo extension
		is_valid_cairo_contract(contract_path)?;
		let cache_dir = dirs::cache_dir().ok_or(CacheError::CacheDirNotSupportedError)?;
		let contract_relative_path = contract_path.strip_prefix(root_dir)?;
		let mut compiled_contract_path = PathBuf::from(
			cache_dir.join(CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR).join(contract_relative_path),
		);
		compiled_contract_path.set_extension("json");
		return Ok(compiled_contract_path)
	}

	pub fn read_cache_file(path: &PathBuf) -> Result<Cache, CacheError> {
		let file = read_to_string(path)?;
		let data = serde_json::from_str::<Cache>(file.as_str())?;
		return Ok(data)
	}
}
