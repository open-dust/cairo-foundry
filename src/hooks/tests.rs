use rstest::rstest;

use crate::cli::commands::test::{tests::run_single_test, TestCommandError, TestStatus};

#[rstest]
#[case(
	"src/hooks/test_cairo_programs/infinite_loop.cairo",
	TestStatus::SUCCESS
)]
fn test_infinite_loop(
	#[case] path: &str,
	#[case] expected_success: TestStatus,
) -> Result<(), TestCommandError> {
	let path = std::path::PathBuf::from(path);
	let result = run_single_test("test_infinite_loop_failing_test", &path, 1000)
		.expect("Should be Ok")
		.success;
	assert_eq!(expected_success, result);
	Ok(())
}
