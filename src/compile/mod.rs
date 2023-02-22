use cairo_rs::serde::deserialize_program::ProgramJson;
use log::warn;
use serde_json::Value;
use std::{fmt::Debug, io, path::PathBuf, process::Command};
use thiserror::Error;
use which::{which, Error as WhichError};

use self::cache::{get_compile_cache_path, hash_file, CacheError, CompileCacheItem};

pub mod cache;

const CAIRO_COMPILE_BINARY: &str = "cairo-compile";

#[derive(Error, Debug)]
pub enum Error {
	#[error("binary '{CAIRO_COMPILE_BINARY}' not found: {0}")]
	CairoCompileBinaryNotFound(#[from] WhichError),
	#[error("failed to execute a process: {0}")]
	RunProcess(io::Error),
	#[error("binary '{0}' failed to compile '{1}'")]
	Compilation(String, String),
	#[error("file '{0}' has no stem")]
	StemlessFile(String),
	#[error("cache directory does not exist on this platform")]
	CacheDirSupported,
	#[error("failed to create file '{0}': {1}")]
	FileCreation(String, io::Error),
	#[error("failed to create directory '{0}': {1}")]
	DirCreation(String, io::Error),
	#[error("failed to write to file '{0}': {1}")]
	WriteToFile(String, io::Error),
	#[error(transparent)]
	Json(#[from] serde_json::Error),
	#[error(transparent)]
	CacheError(#[from] CacheError),
}

/// Compile a cairo file.
///
/// The given `&PathBuf` will be compiled as a simple Cairo file which can then be executed.
/// The generated file will be a JSON file saved in your cache directory.
///
/// Returns the `PathBuf` to the generated file and print the displayable element from the given
/// `PathBuf' or nothing, which means that everything is fine.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// # use std::path::PathBuf;
/// # use cairo_foundry::compile::{compile, Error};
/// # fn foo() -> Result<(), Error> {
///     let mut program = PathBuf::from("path_to_your_program");
///     let compiled_program_path = compile(&program)?;
/// # Ok(())
/// # }
/// ```
pub fn compile(path_to_cairo_file: &PathBuf) -> Result<ProgramJson, Error> {
	let cache_path = get_compile_cache_path(path_to_cairo_file)?;

	let hash = hash_file(path_to_cairo_file)?;

	if cache_path.exists() {
		match CompileCacheItem::read(&cache_path) {
			Ok(cache) =>
				if cache.hash == hash {
					let program_json: ProgramJson = serde_json::from_value(cache.program_json)?;
					return Ok(program_json)
				},
			Err(err) => warn!(
				"Error while reading cache {}: {err}",
				cache_path.display().to_string()
			),
		}
	}

	let compile_output = compile_cairo_file(path_to_cairo_file)?;

	let program_json: Value = serde_json::from_slice(&compile_output)?;

	let cache = CompileCacheItem {
		program_json: program_json.clone(),
		hash,
	};

	let program_json: ProgramJson = serde_json::from_value(program_json)?;

	cache.write(&cache_path)?;

	Ok(program_json)
}

pub fn compile_cairo_file(path_to_cairo_file: &PathBuf) -> Result<Vec<u8>, Error> {
	let path_to_cairo_compiler = which(CAIRO_COMPILE_BINARY)?;

	// Use cairo-compile binary in order to compile the .cairo file
	let compilation_output = Command::new(CAIRO_COMPILE_BINARY)
		.args([&path_to_cairo_file])
		.output()
		.map_err(Error::RunProcess)?;

	if compilation_output.status.success() {
		Ok(compilation_output.stdout)
	} else {
		return Err(Error::Compilation(
			path_to_cairo_compiler.as_path().display().to_string(),
			String::from_utf8(compilation_output.stderr).unwrap_or_else(|e| {
				format!(
					"{} with non utf8 error message: {}",
					path_to_cairo_file.as_path().display(),
					e
				)
			}),
		))
	}
}
