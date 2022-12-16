pub mod cache {
	use std::{fmt::Debug, fs::read_to_string, io, path::PathBuf};

	use thiserror::Error;

	use serde::{Deserialize, Serialize};
	use serde_json;

	#[derive(Error, Debug)]
	pub enum CacheError {
		#[error(transparent)]
		FileNotFoundError(#[from] io::Error),
		#[error(transparent)]
		DeserializeError(#[from] serde_json::Error),
	}

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	pub struct Cache {
		pub contract_path: PathBuf,
		pub compiled_contract_path: PathBuf,
		pub hash: String,
	}

	pub fn read_cache_file(path: &PathBuf) -> Result<Cache, CacheError> {
		let file = read_to_string(path)?;
		let data = serde_json::from_str::<Cache>(file.as_str())?;
		return Ok(data)
	}
}
