use std::{fmt::Display, fs, io, path::PathBuf};

mod tests;

use clap::Args;
use serde::Serialize;

use thiserror::Error;

use super::CommandExecution;

use crate::compile::cache;
#[derive(Args, Debug)]
pub struct CleanArgs {}

#[derive(Debug, Serialize)]
pub struct CleanOutput {
	/// The list of cleaned dirs
	pub dirs: Vec<(PathBuf, bool)>,
}

#[derive(Error, Debug)]
pub enum CleanCommandError {
	#[error(transparent)]
	CacheDirNotSupportedError(#[from] cache::CacheDirNotSupportedError),
	#[error("Cannot remove directory {dir}: {err}")]
	DirDeletion { dir: String, err: io::Error },
}

impl Display for CleanOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for (dir, deleted) in self.dirs.iter() {
			if *deleted {
				write!(f, "Cleaning {}: done\n", dir.display().to_string())?;
			} else {
				write!(f, "Cleaning {}: not found\n", dir.display().to_string())?;
			}
		}
		write!(f, "Cache cleaned successfully.\n")
	}
}

fn clean_cache_path(path: &str) -> Result<(PathBuf, bool), CleanCommandError> {
	let dir = cache::cache_dir()?.join(path);
	let mut deleted = false;

	if dir.exists() {
		fs::remove_dir_all(&dir).map_err(|err| CleanCommandError::DirDeletion {
			dir: dir.as_path().display().to_string(),
			err,
		})?;
		deleted = true;
	}

	Ok((dir, deleted))
}

impl CommandExecution<CleanOutput, CleanCommandError> for CleanArgs {
	fn exec(&self) -> Result<CleanOutput, CleanCommandError> {
		let mut dirs: Vec<(PathBuf, bool)> = Vec::new();

		let paths_to_clean = [
			cache::CAIRO_FOUNDRY_CACHE_DIR,
			cache::CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR,
		];

		for path in paths_to_clean.iter() {
			let result = clean_cache_path(path)?;
			dirs.push(result)
		}

		Ok(CleanOutput { dirs })
	}
}
