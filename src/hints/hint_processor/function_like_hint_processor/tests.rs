use crate::cli::commands::test::{tests::run_single_test, TestCommandError, TestStatus};
use rstest::rstest;

#[rstest]
#[case(
	"src/hints/hint_processor/function_like_hint_processor/test_cairo_programs/wrong_custom_hint.cairo",
	TestStatus::FAILURE
)]
fn wrong_custom_hint(
	#[case] path: &str,
	#[case] expected_success: TestStatus,
) -> Result<(), TestCommandError> {
	let path = std::path::PathBuf::from(path);
	let result = run_single_test("test_wrong_custom_hint", &path).expect("Should be Ok").success;
	assert_eq!(expected_success, result);
	Ok(())
}
