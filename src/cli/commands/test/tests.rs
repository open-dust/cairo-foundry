use crate::cli::commands::{test::TestArgs, CommandExecution};
use std::path::PathBuf;

#[test]
fn test_cairo_contracts() {
	let res = TestArgs {
		root: PathBuf::from("./test_cairo_contracts"),
	}
	.exec()
	.expect("Execution of `test_valid_program.cairo` should be a success");
	assert_eq!(res.to_string(), "");
}

#[test]
fn test_cairo_hints() {
	let res = TestArgs {
		root: PathBuf::from("./test_cairo_hints"),
	}
	.exec()
	.expect("Execution of `test_valid_program.cairo` should be a success");
	assert_eq!(res.to_string(), "");
}
