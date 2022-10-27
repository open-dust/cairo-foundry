use dirs;
use std::{
	fmt::Debug,
	fs::File,
	io::{self, Write},
	path::PathBuf,
	process::Command,
};
use thiserror::Error;

const JSON_FILE_EXTENTION: &str = "json";

#[derive(Error, Debug)]
pub enum Error {
	#[error("failed to execute a process: {0}")]
	RunProcess(io::Error),
	#[error("failed to compile {0}")]
	Compilation(String),
	#[error("file '{0}' has no stem")]
	StemlessFile(String),
	#[error("cache directory does not exist on this platform")]
	CacheDirSupported,
	#[error("failed to create file '{0}': {1}")]
	FileCreation(String, io::Error),
	#[error("failed to create directory'{0}': {1}")]
	DirCreation(String, io::Error),
	#[error("failed to write to file: {0}")]
	WriteToFile(io::Error),
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
/// ```ignore
/// let mut program = PathBuf::from("path_to_your_program");
/// let compiled_program_path = compile(&program);
/// ```
pub fn compile(path_to_cairo_file: &PathBuf) -> Result<PathBuf, Error> {
	// Use cairo-compile binary in order to compile the .cairo file
	let compilation_output = Command::new("cairo-compile")
		.args([&path_to_cairo_file])
		.output()
		.map_err(Error::RunProcess)?;

	// Check if the compilation was successful
	if !compilation_output.status.success() {
		return Err(Error::Compilation(
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

	// Build path to save the  compiled file
	let path_to_cache_dir = dirs::cache_dir().ok_or(Error::CacheDirSupported)?;

	let mut compiled_program_path = PathBuf::new();
	compiled_program_path.push(&path_to_cache_dir);
	compiled_program_path.push("cairo-foundry");
	std::fs::create_dir_all(&compiled_program_path).map_err(|e| {
		Error::DirCreation(compiled_program_path.as_path().display().to_string(), e)
	})?;
	compiled_program_path.push(filename);
	compiled_program_path.set_extension(JSON_FILE_EXTENTION);

	// Create a file to store command output inside a json file
	let mut file = File::create(&compiled_program_path)
		.map_err(|e| Error::FileCreation(path_to_cache_dir.as_path().display().to_string(), e))?;
	file.write_all(&compilation_output.stdout).map_err(Error::WriteToFile)?;

	Ok(compiled_program_path)
}
