use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;
use walkdir::WalkDir;

lazy_static! {
	static ref TEST_FILE_REGEX: Regex = Regex::new(r"^test_.*\.cairo$").unwrap();
}

#[derive(Debug, Error)]
pub enum ListTestsFilesError {
	#[error("Failed to walk directory '{0}'")]
	WalkDir(String, #[source] walkdir::Error),
}

pub fn list_test_files(root: &Path) -> Result<Vec<PathBuf>, ListTestsFilesError> {
	let mut test_files = WalkDir::new(root)
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
			Err(err) => Some(Err(ListTestsFilesError::WalkDir(
				root.display().to_string(),
				err,
			))),
		})
		.collect::<Result<Vec<_>, ListTestsFilesError>>()?;
	test_files.sort();

	Ok(test_files)
}

#[cfg(test)]
mod tests {
	use super::*;
	use assert_matches::assert_matches;

	#[test]
	fn list_test_files_recursively() {
		let root = PathBuf::from("./test_cairo_contracts");

		let result = list_test_files(&root);

		assert!(result.is_ok(), "{}", result.unwrap_err());
		assert_eq!(
			vec![
				PathBuf::from("./test_cairo_contracts/test_invalid_program.cairo"),
				PathBuf::from("./test_cairo_contracts/test_valid_program.cairo"),
			],
			result.unwrap()
		)
	}

	#[test]
	fn returns_error_in_case_of_failure() {
		let root = PathBuf::from("invalid");

		let result = list_test_files(&root);

		assert_matches!(result, Err(ListTestsFilesError::WalkDir(r, _)) if &r == "invalid");
	}
}
