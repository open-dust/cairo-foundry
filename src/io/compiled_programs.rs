use std::{io};

use cairo_rs::serde::deserialize_program::ProgramJson;
use regex::Regex;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ListTestEntrypointsError {
	#[error("Failed to read the compiled cairo program file")]
	FailedToReadCompiledProgram(#[from] io::Error),
	#[error("Failed to parse the content of the compiled cairo program as json")]
	InvalidCompiledProgramFormat(#[from] serde_json::Error),
}

/// Get the list of test entrypoint from a compiled cairo file.
/// test entrypoint are function starting with "test_".
/// The function will return a list of test entrypoint as `String` (ie: "test_function");
///
/// return a vector of entrypoints
pub fn list_test_entrypoints(
	program_json: &ProgramJson,
) -> Result<Vec<String>, ListTestEntrypointsError> {
	let re = Regex::new(r"__main__.(test_\w+)$").expect("Should be a valid regex");
	let mut test_entrypoints = Vec::new();

	for (key, value) in program_json.identifiers.iter() {
		if re.is_match(key) && value.type_ == Some("function".into()) {
			// capture 0 refers to the whole match
			// capture n-1 refers to the next to last match
			// captures are denoted with () in regex
			for capture in re.captures_iter(key) {
				// regex __main__.(test_\w+)$ has 2 captures
				// capture 0 is the whole match
				// capture 1 is the first (and last) capture in this regex
				test_entrypoints.push(capture[1].to_string());
			}
		}
	}
	Ok(test_entrypoints)
}
