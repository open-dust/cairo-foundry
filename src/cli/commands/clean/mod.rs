#[cfg(test)]
mod tests;

use std::{fmt::Display, fs, io, path::PathBuf};

use clap::Args;
use serde::Serialize;

use thiserror::Error;

use super::CommandExecution;

use crate::compile::cache;
#[derive(Args, Debug)]
pub struct CleanArgs {}

#[derive(Debug, Serialize, PartialEq)]
pub struct CleanOutput {
	pub dirs: Vec<(PathBuf, bool)>,
}

#[derive(Error, Debug)]
pub enum CleanCommandError {
	#[error(transparent)]
	CacheDirNotSupported(#[from] cache::CacheDirNotSupported),
	#[error("Cannot remove directory {dir}: {err}")]
	DirDeletion { dir: String, err: io::Error },
}

impl Display for CleanOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (dir, deleted) in self.dirs.iter() {
			if *deleted {
				write!(f, "cleaned  : {}\n", dir.display().to_string())?;
			} else {
				write!(f, "not found: {}\n", dir.display().to_string())?;
			}
		}
		write!(f, "Cache cleaned successfully.\n")
	}
}

fn remove_dir_all_if_exists(dir: &PathBuf) -> Result<bool, CleanCommandError> {
	if dir.exists() {
		fs::remove_dir_all(&dir).map_err(|err| CleanCommandError::DirDeletion {
			dir: dir.as_path().display().to_string(),
			err,
		})?;
		return Ok(true);
	}
	Ok(false)
}

impl CommandExecution<CleanOutput, CleanCommandError> for CleanArgs {
	fn exec(&self) -> Result<CleanOutput, CleanCommandError> {
		let cache_dir = cache::cache_dir()?;

		let mut dirs: Vec<(PathBuf, bool)> = Vec::new();

		let paths_to_clean = [
			cache::CAIRO_FOUNDRY_CACHE_DIR,
			cache::CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR,
		];

		for path in paths_to_clean.iter() {
			let dir = cache_dir.join(path);
			let deleted = remove_dir_all_if_exists(&dir)?;
			dirs.push((dir, deleted));
		}

		Ok(CleanOutput { dirs })
	}
}
