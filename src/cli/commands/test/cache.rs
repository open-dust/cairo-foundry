#[cfg(test)]

use std::{
	fmt::Debug,
	fs::{read_to_string},
	io::{self},
	path::{PathBuf},
};

use thiserror::Error;

use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum CacheError {
	#[error("failed to read file '{0}': {1}")]
	FileNotFound(PathBuf, io::Error),
	#[error("failed to read file '{0}': {1}")]
	DeserializeError(String, serde_json::Error),

}


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct CacheJson {
	pub contract_path: String,
	pub hash: String,
}

pub fn read_json_file(path: &PathBuf) -> Result<CacheJson, CacheError> {
	let file = read_to_string(path).map_err(|op| CacheError::FileNotFound(path.to_owned(), op))?;
	let data = serde_json::from_str::<CacheJson>(file.as_str()).map_err(|op| CacheError::DeserializeError(file, op))?;
	return Ok(data);
}
