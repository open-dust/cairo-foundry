#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;
use walkdir::WalkDir;

lazy_static! {
	static ref TEST_FILE_REGEX: Regex = Regex::new(r"^test_.*\.cairo$").unwrap();
}

#[derive(Debug, Error)]
pub enum ListTestsError {
	#[error("Failed to walk directory '{0}'")]
	WalkDir(String, #[source] walkdir::Error),
}

pub fn list_tests(root: &Path) -> Result<Vec<PathBuf>, ListTestsError> {
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
			Err(err) => Some(Err(ListTestsError::WalkDir(
				root.display().to_string(),
				err,
			))),
		})
		.collect::<Result<Vec<_>, ListTestsError>>()?;
	test_files.sort();

	Ok(test_files)
}
