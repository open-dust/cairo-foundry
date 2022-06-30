use super::Command;
use clap::{Args, ValueHint};
use lazy_static::lazy_static;
use log::info;
use regex::Regex;
use std::{fmt, path::PathBuf};
use walkdir::WalkDir;

/// List command
#[derive(Args, Debug)]
pub struct List {
	/// Root path
	#[clap(short, long, value_hint=ValueHint::DirPath, value_parser=path_is_valid_directory)]
	root: PathBuf,
}

fn path_is_valid_directory(path: &str) -> Result<PathBuf, String> {
	let path = PathBuf::from(path);
	if path.exists() && path.is_dir() {
		Ok(path)
	} else {
		Err(format!("\"{}\" is not a valid directory", path.display()))
	}
}

/// List command output
#[derive(Debug)]
pub struct Output {
	/// The list of test files found
	files: Vec<PathBuf>,
}

impl fmt::Display for Output {
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

impl Command for List {
	type Output = Output;

	fn exec(&self) -> Result<Output, String> {
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

		Ok(Output { files: test_files })
	}
}

#[cfg(test)]
mod test {
	use super::{List, Output};
	use crate::cli::commands::Command;
	use std::path::PathBuf;

	#[test]
	fn list_test_files_recursively() {
		let result = List {
			root: PathBuf::from("./test_starknet_projects"),
		}
		.exec();

		assert!(result.is_ok(), "{}", result.unwrap_err());
		assert_eq!(
			vec![
				PathBuf::from("./test_starknet_projects/no_builtin/test_contract.cairo"),
				PathBuf::from("./test_starknet_projects/with_HashBuiltin/test_contract.cairo")
			],
			result.unwrap().files
		)
	}

	#[test]
	fn returns_error_in_case_of_failure() {
		let result = List {
			root: PathBuf::from("invalid"),
		}
		.exec();

		assert!(result.is_err());
		assert_eq!(
			"IO error for operation on invalid: No such file or directory (os error 2)",
			result.unwrap_err().to_string()
		);
	}

	#[test]
	fn output_can_display_as_string() {
		let output = Output {
			files: vec![PathBuf::from("item 1"), PathBuf::from("item 2")],
		};

		assert_eq!("item 1\nitem 2", format!("{}", output));
	}
}
