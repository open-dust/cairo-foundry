use assert_matches::assert_matches;

use super::{ListArgs, ListOutput};
use crate::cli::commands::{list::ListCommandError, CommandExecution};
use std::path::PathBuf;

#[test]
fn list_test_files_recursively() {
	let result = ListArgs {
		root: PathBuf::from("./test_cairo_contracts"),
	}
	.exec();

	assert!(result.is_ok(), "{}", result.unwrap_err());
	assert_eq!(
		vec![
			PathBuf::from("./test_cairo_contracts/test_invalid_program.cairo"),
			PathBuf::from("./test_cairo_contracts/test_valid_program.cairo"),
		],
		result.unwrap().files
	)
}

#[test]
fn returns_error_in_case_of_failure() {
	let result = ListArgs {
		root: PathBuf::from("invalid"),
	}
	.exec();

	assert_matches!(result, Err(ListCommandError::ListFilesError(_)));
}

#[test]
fn output_can_display_as_string() {
	let output = ListOutput {
		files: vec![PathBuf::from("item 1"), PathBuf::from("item 2")],
	};

	assert_eq!("item 1\nitem 2", format!("{}", output));
}
