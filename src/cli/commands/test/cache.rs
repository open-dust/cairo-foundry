pub mod cache {
	use std::{fmt::Debug, fs::read_to_string, io, path::PathBuf};
	use dirs;

	use thiserror::Error;

	use serde::{Deserialize, Serialize};
	use serde_json;

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
		
	}

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	pub struct Cache {
		pub contract_path: PathBuf,
		pub compiled_contract_path: PathBuf,
		pub hash: String,
	}

	struct CairoContract {
		pub contract_path: PathBuf
	}

	impl CairoContract {
		fn new(contract_path: PathBuf) -> Self {
			// extract contract name from contract_path
			CairoContract { contract_path: contract_path }
		}

		fn get_cache_path(&self) -> Result<PathBuf, CacheError> {
			let cache_dir = dirs::cache_dir().ok_or(CacheError::CacheDirNotSupportedError)?;
			let contract_name = self.contract_path.file_stem().ok_or(CacheError::FileNameDoesNotExistError)?;
			
			let path_to_cache =
			PathBuf::from(cache_dir.join("cairo-foundry-cache").join(contract_name));
			return Ok(path_to_cache);
		}

		
	}




	pub fn read_cache_file(path: &PathBuf) -> Result<Cache, CacheError> {
		let file = read_to_string(path)?;
		let data = serde_json::from_str::<Cache>(file.as_str())?;
		return Ok(data)
	}

	// create class


	
}
