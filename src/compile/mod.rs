use dirs;
use std::{
	fmt::Debug,
	fs::File,
	io::{self, Write},
	path::PathBuf,
	process::Command,
};
use thiserror::Error;
use which::{which, Error as WhichError};

const JSON_FILE_EXTENTION: &str = "json";
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
/// ```no_run
/// # use std::path::PathBuf;
/// # use cairo_foundry::compile::{compile, Error};
/// # fn foo() -> Result<(), Error> {
///     let mut program = PathBuf::from("path_to_your_program");
///     let compiled_program_path = compile(&program)?;
/// # Ok(())
/// # }
/// ```
pub fn compile(path_to_cairo_file: &PathBuf) -> Result<PathBuf, Error> {
	let path_to_cairo_compiler = which(CAIRO_COMPILE_BINARY)?;

	// Use cairo-compile binary in order to compile the .cairo file
	let compilation_output = Command::new(CAIRO_COMPILE_BINARY)
		.args([&path_to_cairo_file])
		.output()
		.map_err(Error::RunProcess)?;

	// Check if the compilation was successful
	if !compilation_output.status.success() {
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

	// Retrieve only the file name to create a clean compiled file name.
	let filename = path_to_cairo_file
		.file_stem()
		.ok_or_else(|| Error::StemlessFile(path_to_cairo_file.as_path().display().to_string()))?;

	let path_to_cache_dir = dirs::cache_dir().ok_or(Error::CacheDirSupported)?;

	// Build path to save the  compiled file
	let mut compiled_program_path = PathBuf::new();
	compiled_program_path.push(&path_to_cache_dir);
	compiled_program_path.push("compiled-cairo-files");
	std::fs::create_dir_all(&compiled_program_path).map_err(|e| {
		Error::DirCreation(compiled_program_path.as_path().display().to_string(), e)
	})?;
	compiled_program_path.push(filename);
	compiled_program_path.set_extension(JSON_FILE_EXTENTION);

	// Create a file to store command output inside a json file
	let mut file = File::create(&compiled_program_path)
		.map_err(|e| Error::FileCreation(path_to_cache_dir.as_path().display().to_string(), e))?;
	file.write_all(&compilation_output.stdout)
		.map_err(|e| Error::WriteToFile(path_to_cache_dir.as_path().display().to_string(), e))?;

	Ok(compiled_program_path)
}
