mod tests;
use dirs;
use std::{
	fmt::Debug,
	fs::{File, read_to_string},
	io::{self, Write},
	path::{PathBuf, Path},
	process::Command,
};
use thiserror::Error;

use std::path::StripPrefixError;
use std::io::BufReader;
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

pub fn read_json_file(path: &PathBuf) -> Result<CacheJson, Error> {
	let file = read_to_string(path).map_err(|op| Error::FileNotFound(path.to_owned(), op))?;
	let data = serde_json::from_str::<CacheJson>(file.as_str()).map_err(|op| Error::DeserializeError(file, op))?;
	return Ok(data);
}
