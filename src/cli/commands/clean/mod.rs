use std::{fmt::Display, fs, path::PathBuf};

mod tests;

use clap::Args;
use serde::Serialize;

use crate::compile::Error::CacheDirSupported;

use thiserror::Error;

use super::CommandExecution;

#[derive(Args, Debug)]
pub struct CleanArgs {}

#[derive(Debug, Serialize, Default)]
pub struct CleanOutput(String);

#[derive(Error, Debug)]
pub enum CleanCommandError {
	#[error(transparent)]
	CleanCacheDirSupported(#[from] crate::compile::Error),
	// TODO: add  the directory path and the error returned from remove_dir_all
	#[error("Cannot remove directory")]
	DirDeletion,
}

impl Display for CleanOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", &self.0)
	}
}

fn clear_directory(path: &str) -> Result<(), CleanCommandError> {
	let path_to_cache_dir = dirs::cache_dir().ok_or(CacheDirSupported)?;

	let mut dir = PathBuf::new();
	dir.push(&path_to_cache_dir);
	dir.push(path);

	if dir.exists() {
		fs::remove_dir_all(dir).map_err(|_err| CleanCommandError::DirDeletion)?;
	}

	Ok(())
}

impl CommandExecution<CleanOutput, CleanCommandError> for CleanArgs {
	fn exec(&self) -> Result<CleanOutput, CleanCommandError> {
		clear_directory("cairo-foundry-cache")?;
		clear_directory("compiled-cairo-files")?;
		Ok(Default::default())
	}
}
