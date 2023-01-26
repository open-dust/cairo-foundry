use rstest::rstest;

use crate::cli::commands::test::{tests::run_single_test, TestCommandError, TestStatus};

#[rstest]
#[case(
	"src/hints/mock_call/test_cairo_programs/mock_call.cairo",
	TestStatus::SUCCESS
)]
fn mock_call(
	#[case] path: &str,
	#[case] expected_success: TestStatus,
) -> Result<(), TestCommandError> {
	let path = std::path::PathBuf::from(path);
	let result = run_single_test("test_mock_call", &path, 1000000).expect("Should be Ok").success;
	assert_eq!(expected_success, result);
	Ok(())
}
