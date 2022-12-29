use crate::cli::commands::test::{tests::run_single_test, TestCommandError};
use rstest::rstest;

#[rstest]
#[case("src/hints/mock_call/mock_single_felt.cairo", true)]
#[case("src/hints/mock_call/mock_ptr_felt.cairo", true)]
fn test_mock_call(
	#[case] path: &str,
	#[case] expected_success: bool,
) -> Result<(), TestCommandError> {
	let path = std::path::PathBuf::from(path);
	let result: bool =
		run_single_test("test_mock_call", &path).expect("Should be Ok").success.into();
	assert_eq!(expected_success, result);
	Ok(())
}
