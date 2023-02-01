use std::io;
use std::{fmt::Display, fs, path::PathBuf};

mod tests;

use clap::Args;
use serde::Serialize;

use crate::compile::cache;
use crate::compile::Error::CacheDirSupported;

use thiserror::Error;

use super::CommandExecution;

#[derive(Args, Debug)]
pub struct CleanArgs {}

#[derive(Debug, Serialize)]
pub struct CleanOutput {
	/// The list of cleaned dirs
	pub dirs: Vec<PathBuf>,
}

#[derive(Error, Debug)]
pub enum CleanCommandError {
	#[error(transparent)]
	CleanCacheDirSupported(#[from] crate::compile::Error),
	#[error("Cannot remove directory {dir}: {err}")]
	DirDeletion { dir: PathBuf, err: io::Error },
}

impl Display for CleanOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.dirs.is_empty() {
			write!(f, "No directory to clean")
		} else {
			write!(
				f,
				"Cleaned successfully: \n{}\n",
				self.dirs
					.iter()
					.map(|path| path.display().to_string())
					.collect::<Vec<_>>()
					.join("\n")
			)
		}
	}
}

fn clean_cache_dir(path: &str) -> Result<Option<PathBuf>, CleanCommandError> {
	let path_to_cache_dir = dirs::cache_dir().ok_or(CacheDirSupported)?;

	let mut dir = PathBuf::new();
	dir.push(&path_to_cache_dir);
	dir.push(path);

	if dir.exists() {
		fs::remove_dir_all(&dir).map_err(|err| CleanCommandError::DirDeletion {
			// TODO: avoid .clone
			dir: dir.clone(),
			err,
		})?;
		return Ok(Some(dir));
	}

	Ok(None)
}

impl CommandExecution<CleanOutput, CleanCommandError> for CleanArgs {
	fn exec(&self) -> Result<CleanOutput, CleanCommandError> {
		let mut cleaned_dirs: Vec<PathBuf> = Vec::new();

		let cleaned_foundry_cache_dir = clean_cache_dir(cache::CAIRO_FOUNDRY_CACHE_DIR)?;
		if cleaned_foundry_cache_dir.is_some() {
			cleaned_dirs.push(cleaned_foundry_cache_dir.unwrap());
		}
		let cleaned_compiled_contract_dir =
			clean_cache_dir(cache::CAIRO_FOUNDRY_COMPILED_CONTRACT_DIR)?;

		if cleaned_compiled_contract_dir.is_some() {
			cleaned_dirs.push(cleaned_compiled_contract_dir.unwrap());
		}

		Ok(CleanOutput { dirs: cleaned_dirs })
	}
}
