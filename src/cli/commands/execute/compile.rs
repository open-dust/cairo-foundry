use dirs;
use std::{fs::File, io::Write, path::PathBuf, process::Command};

const JSON_FILE_EXTENTION: &str = "json";

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
pub fn compile(program: &PathBuf) -> Result<PathBuf, String> {
	// Use cairo-compile binary in order to compile the .cairo file
	let compiled_output = Command::new("cairo-compile")
		.args([&program])
		.output()
		.expect("cairo-compile command failed to start");

	// Check if the compilation was successful
	if !compiled_output.status.success() {
		return Err(format!(
			"Compilation of {} failed: {:?}",
			program.display(),
			compiled_output
		))
	}

	// Retrieve only the file name to create a clean compiled file name.
	// Is safe to unwrap because the call to cairo-compile would have failed if program was not
	// terminated by .cairo.
	let filename = program.file_stem().unwrap();

	// Build path to save the  compiled file
	let dir = dirs::cache_dir().expect("This operating system is not supported by the dirs crate");

	let mut compiled_program_path = PathBuf::new();
	compiled_program_path.push(&dir);
	compiled_program_path.push(filename);
	compiled_program_path.set_extension(JSON_FILE_EXTENTION);

	// Create a file to store command output inside a json file
	let mut file = File::create(&compiled_program_path)
		.expect("Failed to create a file to store the compiled program");
	file.write_all(&compiled_output.stdout)
		.expect("Failed to write the compiled program to disk");

	Ok(compiled_program_path)
}
