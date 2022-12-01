#[cfg(test)]
mod tests;

use std::{fmt, path::PathBuf};

use clap::{Args, ValueHint};
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

use super::CommandExecution;

/// List command
#[derive(Args, Debug)]
pub struct ListArgs {
	/// Root path
	#[clap(short, long, value_hint=ValueHint::DirPath, value_parser=path_is_valid_directory)]
	pub root: PathBuf,
}

/// Function used to validate directory type of the specified Path
/// `path: &str` the path to test
/// Returns the `PathBuf` for the given path
/// or Err, which means that path does not exist or is not a dir.
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

impl CommandExecution<ListOutput> for ListArgs {
	/// This Command List and store all Cairo tests files within root directory.
	/// Every file in Root directory must follow the following regex: "^test_.*\.cairo$"
	/// to be added to the result.
	///
	/// Returns a `ListOutput` struct with tests files in a `PathBuf` vector.
	fn exec(&self) -> Result<ListOutput, String> {
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
				Err(err) => Some(Err(err)),
			})
			.collect::<Result<Vec<_>, _>>()
			.map_err(|err| err.to_string())?;
		test_files.sort();

		Ok(ListOutput { files: test_files })
	}
}
