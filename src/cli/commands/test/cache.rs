use dirs;
use std::{
	fmt::Debug,
	fs::File,
	io::{self, Write},
	path::PathBuf,
	process::Command,
};
use thiserror::Error;
use which::{which, Error as WhichError};

use std::path::StripPrefixError;


#[derive(Error, Debug)]
pub enum Error {
	#[error("StripPrefixError: {0}")]
	StripPrefixError(#[from] StripPrefixError),
    #[error("failed to execute a process: {0}")]
    RunProcess(io::Error),
    #[error("binary '{0}' failed to compile '{1}'")]
    Compilation(String, String),
    #[error("file '{0}' has no stem")]
    StemlessFile(String),
    #[error("cache directory does not exist on this platform")]
    CacheDirSupported,
    #[error("failed to create file '{0}': {1}")]
    FileCreation(String, io::Error),
    #[error("failed to create directory '{0}': {1}")]
    DirCreation(String, io::Error),
    #[error("failed to write to file '{0}': {1}")]
    WriteToFile(String, io::Error),
}


pub fn create_compiled_contract_path(path_to_contract_file: &PathBuf, root: &PathBuf) -> Result<PathBuf, Error> {
	let cache_dir = dirs::cache_dir().ok_or(Error::CacheDirSupported)?;
    // parent of root
    let root_parent = root.parent().ok_or(Error::CacheDirSupported)?;
	let relative_path = path_to_contract_file.strip_prefix(root_parent).map_err(Error::StripPrefixError)?;
	let mut path_to_compiled_contract_path = PathBuf::new();
	path_to_compiled_contract_path.push(&cache_dir);
	path_to_compiled_contract_path.push("compiled-cairo-files");
	path_to_compiled_contract_path.push(&relative_path);
	path_to_compiled_contract_path.set_extension("json");
	return Ok(path_to_compiled_contract_path);
}
