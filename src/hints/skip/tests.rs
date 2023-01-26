use rstest::rstest;

use crate::cli::commands::test::{tests::run_single_test, TestCommandError, TestStatus};

#[rstest]
#[case("src/hints/skip/test_cairo_programs/skip.cairo", TestStatus::FAILURE)]
fn skip(#[case] path: &str, #[case] expected_success: TestStatus) -> Result<(), TestCommandError> {
	let path = std::path::PathBuf::from(path);
	let result = run_single_test("test_skip", &path, 1000000).expect("Should be Ok").success;
	assert_eq!(expected_success, result);
	Ok(())
}
