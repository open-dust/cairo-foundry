use crate::cli::commands::{test::TestArgs, CommandExecution};
use std::path::PathBuf;

#[test]
fn test_cairo_contracts() {
	TestArgs {
		root: PathBuf::from("./test_cairo_contracts"),
	}
	.exec()
	.expect("Execution of `test_valid_program.cairo` should be a success");
}

#[test]
fn test_cairo_hints() {
	TestArgs {
		root: PathBuf::from("./test_cairo_hints"),
	}
	.exec()
	.expect("Execution of `test_valid_program.cairo` should be a success");
}
