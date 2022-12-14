pub mod cache {
	use std::{
		fmt::Debug,
		fs::read_to_string,
		path::PathBuf,
	};

	use thiserror::Error;

	use serde::{Deserialize, Serialize};
	use serde_json;

	#[derive(Error, Debug)]
	pub enum CacheError {
		#[error("file `{0}` not found ")]
		FileNotFound(PathBuf),
		#[error("failed to deserialize file '{0}': {1}")]
		DeserializeError(String, serde_json::Error),
	}

	#[derive(Serialize, Deserialize, Debug, PartialEq)]
	pub struct CacheCairoFoundry {
		pub contract_path: PathBuf,
		pub compiled_contract_path: PathBuf,
		pub hash: String,
	}

	pub fn read_cache_file(path: &PathBuf) -> Result<CacheCairoFoundry, CacheError> {
		let file =
			read_to_string(path).map_err(|_op_| CacheError::FileNotFound(path.to_owned()))?;
		let data = serde_json::from_str::<CacheCairoFoundry>(file.as_str())
			.map_err(|op| CacheError::DeserializeError(file, op))?;
		return Ok(data)
	}
}
