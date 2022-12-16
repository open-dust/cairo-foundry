#[cfg(test)]
mod tests;

use std::{fmt, path::PathBuf};

use clap::{Args, ValueHint};
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use serde::Serialize;
use thiserror::Error;
use walkdir::WalkDir;

use super::CommandExecution;

/// List command
#[derive(Args, Debug)]
pub struct ListArgs {
	/// Root path
	#[clap(short, long, value_hint=ValueHint::DirPath, value_parser=path_is_valid_directory)]
	pub root: PathBuf,
}

#[derive(Error, Debug)]
pub enum ListCommandError {
	#[error(transparent)]
	ListFilesError(#[from] walkdir::Error),
}

/// Function used to validate directory type of the specified Path
/// `path: &str` the Path to test
/// Returns the `PathBuf` for the given path
/// or an Err with the Path if it does not exist or if it is not a directory.
pub fn path_is_valid_directory(path: &str) -> Result<PathBuf, String> {
	let path = PathBuf::from(path);
	if path.exists() && path.is_dir() {
		Ok(path)
	} else {
		Err(format!("\"{}\" is not a valid directory", path.display()))
	}
}

/// List command output
#[derive(Debug, Serialize)]
pub struct ListOutput {
	/// The list of test files found
	pub files: Vec<PathBuf>,
}

impl fmt::Display for ListOutput {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			self.files
				.iter()
				.map(|path| path.display().to_string())
				.collect::<Vec<_>>()
				.join("\n")
		)
	}
}

impl CommandExecution<ListOutput, ListCommandError> for ListArgs {
	/// Implementation of CommandExecution Trait for the List Command
	///
	/// The List Command lists and returns the 'ListOutput' of all the valid
	/// Cairo tests files within the ListArgs root directory(PathBuf).
	/// To be valid, the filepath must follow the following regex:
	///    "^test_.*\.cairo$"
	///
	/// Examples
	///    test_cairo_contracts/test_invalid_program.cairo > Valid
	///    test_hints/test/minor/failing.cairo > Valid
	///    test_cairo_hints/test_mock_call.cairo.test > Invalid, ends with "test" not ".cairo"
	///    mytest_hints/test_skip.cairo > Invalid, starts with "mytests" not "test_"
	///
	/// When using the cairo-compile command, the root directory is the one specified by the option "--root"
	///
	/// Returns a `ListOutput` struct with all valid tests files in the `.files: vector<PathBuf>`
	/// or an error `ListCommandError`, the first Error encoutered during the processing of the root directory.
	fn exec(&self) -> Result<ListOutput, ListCommandError> {
		info!("Listing files within directory {:?}", self.root);

		lazy_static! {
			static ref TEST_FILE_REGEX: Regex = Regex::new(r"^test_.*\.cairo$").unwrap();
		}

		let mut test_files = WalkDir::new(&self.root)
			.into_iter()
			.filter_map(|entry_result| match entry_result {
				Ok(entry) => {
					if entry.path().is_file()
						&& TEST_FILE_REGEX.is_match(&entry.file_name().to_string_lossy())
					{
						Some(Ok(entry.path().to_path_buf()))
					} else {
						None
					}
				},
				Err(err) => Some(Err(ListCommandError::ListFilesError(err))),
			})
			.collect::<Result<Vec<_>, ListCommandError>>()?;
		test_files.sort();

		Ok(ListOutput { files: test_files })
	}
}
