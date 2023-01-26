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
<<<<<<< HEAD
	let result = run_single_test("test_wrong_custom_hint", &path, 1000000)
		.expect("Should be Ok")
		.success;
=======
	let result = run_single_test("test_wrong_custom_hint", &path).expect("Should be Ok").success;
>>>>>>> a43eb506328d221943341de872eb1a34bd608ca6
	assert_eq!(expected_success, result);
	Ok(())
}
