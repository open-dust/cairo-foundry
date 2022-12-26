use crate::cli::commands::test::{tests::run_single_test, TestCommandError};
use rstest::rstest;

#[rstest]
#[case("src/hints/expect_revert/failing.cairo", false)]
#[case("src/hints/expect_revert/success.cairo", true)]
fn test_expect_revert(
	#[case] path: &str,
	#[case] expected_success: bool,
) -> Result<(), TestCommandError> {
	let path = std::path::PathBuf::from(path);
	let result: bool = run_single_test("test_expect_revert", &path)
		.expect("Should be Ok")
		.success
		.into();
	assert_eq!(expected_success, result);
	Ok(())
}
