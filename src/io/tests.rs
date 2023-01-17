use assert_matches::assert_matches;

use std::path::PathBuf;

use crate::io::ListTestsError;

use super::list_tests;

#[test]
fn list_test_files_recursively() {
	let root = PathBuf::from("./test_cairo_contracts");

	let result = list_tests(&root);

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

	let result = list_tests(&root);

	assert_matches!(result, Err(ListTestsError::WalkDir(r, _)) if r == "invalid".to_string());
}
