#[cfg(test)]
mod test {
	use crate::cli::commands::{execute::ExecuteArgs, CommandExecution};
	use dirs;
	use std::path::PathBuf;

	const JSON_FILE_EXTENTION: &str = "json";

	#[test]
	fn valid_programs() {
		let res = ExecuteArgs {
			program: PathBuf::from("./test_cairo_contracts/valid_program.cairo"),
		}
		.exec()
		.expect("Execution of `valid_program.cairo` should be a success");
		assert_eq!(res.to_string(), "50\n");
	}

	#[test]
	fn invalid_programs() {
		// Recover the compiled file of valid_Program
		let dir = dirs::cache_dir().unwrap();
		let mut path = PathBuf::new();
		path.push(&dir);
		path.push("valid_program");
		path.set_extension(JSON_FILE_EXTENTION);

		assert!(
			ExecuteArgs {
				program: PathBuf::from(&path),
			}
			.exec()
			.is_err()
		);
		// Invalid File (Starknet Contract)
		assert!(
			ExecuteArgs {
				program: PathBuf::from("./test_cairo_contracts/invalid_program.cairo"),
			}
			.exec()
			.is_err()
		);
	}
}
