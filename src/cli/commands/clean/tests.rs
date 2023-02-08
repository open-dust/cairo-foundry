#[cfg(test)]
use std::{fs, path::PathBuf};

use crate::{
	cli::commands::{
		clean::{remove_dir_all_if_exists, CleanArgs, CleanCommandError, CleanOutput},
		CommandExecution,
	},
	compile::cache,
};

#[test]
fn output_can_display_as_string() {
	let output = CleanOutput {
		dirs: vec![
			(PathBuf::from("/dir1"), true),
			(PathBuf::from("/dir2"), false),
		],
	};

	let expected_output = "cleaned  : /dir1\nnot found: /dir2\nCache cleaned successfully.\n";

	assert_eq!(expected_output, format!("{output}"));
}

#[test]
fn clean_cache_dirs() -> Result<(), CleanCommandError> {
	let test_cache_dir = cache::cache_dir()?;
	fs::create_dir_all(test_cache_dir.join(cache::CAIRO_FOUNDRY_CACHE_DIR)).map_err(|e| {
		CleanCommandError::DirCreation {
			dir: test_cache_dir.as_path().display().to_string(),
			err: e,
		}
	})?;

	remove_dir_all_if_exists(&test_cache_dir.join(cache::CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR))?;

	let output = CleanArgs {}.exec()?;

	let expected_dirs = vec![
		(test_cache_dir.join(cache::CAIRO_FOUNDRY_CACHE_DIR), true),
		(
			test_cache_dir.join(cache::CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR),
			false,
		),
	];

	let expected_output = CleanOutput {
		dirs: expected_dirs,
	};

	assert_eq!(expected_output, output);

	remove_dir_all_if_exists(&test_cache_dir)?;

	Ok(())
}
