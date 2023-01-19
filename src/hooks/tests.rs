use std::path::PathBuf;

use crate::cli::commands::{test::TestArgs, CommandExecution};

#[test]
fn test_infinite_loop() {
	TestArgs {
		root: PathBuf::from("./test_cairo_contracts"),
		max_steps: 10000,
	}
	.exec()
	.unwrap();
}
