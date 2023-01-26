use crate::cli::commands::test::{tests::run_single_test, TestCommandError, TestStatus};
use rstest::rstest;

#[rstest]
#[case(
	"src/hints/expect_revert/test_cairo_programs/failing.cairo",
	TestStatus::FAILURE
)]
#[case(
	"src/hints/expect_revert/test_cairo_programs/success.cairo",
	TestStatus::SUCCESS
)]
fn expect_revert(
	#[case] path: &str,
	#[case] expected_success: TestStatus,
) -> Result<(), TestCommandError> {
	let path = std::path::PathBuf::from(path);
	let result = run_single_test("test_expect_revert", &path, 1000000)
		.expect("Should be Ok")
		.success;
	assert_eq!(expected_success, result);
	Ok(())
}
