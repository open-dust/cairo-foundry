use crate::cli::commands::{clean::CleanArgs, CommandExecution};

#[test]
fn should_clean_properly() {
	let result = CleanArgs {}.exec();
	assert!(result.is_ok(), "{}", result.unwrap_err())
}
