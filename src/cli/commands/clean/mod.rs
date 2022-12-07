use std::{fmt::Display, fs, path::PathBuf};

mod tests;

use clap::Args;
use serde::Serialize;

use crate::compile::Error;

use super::CommandExecution;

#[derive(Args, Debug)]
pub struct CleanArgs {}

#[derive(Debug, Serialize, Default)]
pub struct CleanOutput(String);

impl Display for CleanOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", &self.0)
	}
}

fn clear_directory(path: &str) -> Result<(), String> {
	let path_to_cache_dir = dirs::cache_dir()
		.ok_or(Error::CacheDirSupported)
		.map_err(|err| err.to_string())?;

	let mut dir = PathBuf::new();
	dir.push(&path_to_cache_dir);
	dir.push(path);

	if dir.exists() {
		fs::remove_dir_all(dir).map_err(|err| err.to_string())?;
	}

	Ok(())
}

impl CommandExecution<CleanOutput> for CleanArgs {
	fn exec(&self) -> Result<CleanOutput, String> {
		clear_directory("cairo-foundry-cache").unwrap();
		clear_directory("compiled-cairo-files").unwrap();
		Ok(Default::default())
	}
}
