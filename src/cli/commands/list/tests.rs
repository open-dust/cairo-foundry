use std::path::PathBuf;

use super::ListOutput;

#[test]
fn output_can_display_as_string() {
	let output = ListOutput {
		files: vec![PathBuf::from("item 1"), PathBuf::from("item 2")],
	};

	assert_eq!("item 1\nitem 2", format!("{}", output));
}
