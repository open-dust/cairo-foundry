use crate::cli::commands::test::tests::run_single_test;
use rstest::rstest;

#[rstest]
#[case("src/hints/expect_revert/failing.cairo", false)]
#[case("src/hints/expect_revert/success.cairo", true)]
fn test_expect_revert(#[case] path: &str, #[case] expected_success: bool) {
	let path = std::path::PathBuf::from(path);
	let (_, actual_success) = run_single_test("test_expect_revert", &path);
	assert_eq!(expected_success, actual_success);
}
