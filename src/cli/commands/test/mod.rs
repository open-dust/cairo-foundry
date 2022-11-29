#[cfg(test)]
pub mod tests;

use crate::hints::{self, EXPECT_REVERT_FLAG};
use regex::Regex;

use cairo_rs::{
	hint_processor::builtin_hint_processor::builtin_hint_processor_definition::{
		BuiltinHintProcessor, HintFunc,
	},
	vm::{
		errors::{cairo_run_errors::CairoRunError, vm_errors::VirtualMachineError},
		hook::Hooks,
	},
};
use clap::{Args, ValueHint};
use colored::Colorize;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io;
use std::{
	fmt::{format, Display},
	fs::File,
	io::{BufReader, BufWriter},
	path::{self, PathBuf},
	sync::Arc,
	time::Instant,
};
use uuid::Uuid;

use serde_json::Map;
use std::io::Write;
use std::path::Path;

use super::{
	list::{path_is_valid_directory, ListArgs, ListOutput},
	CommandExecution,
};

use crate::{
	cairo_run::cairo_run,
	compile::compile,
	hints::{clear_buffer, expect_revert, get_buffer, init_buffer},
	hooks,
};

#[derive(Args, Debug)]
pub struct TestArgs {
	/// Path to a cairo directory
	#[clap(short, long, value_hint=ValueHint::DirPath, value_parser=path_is_valid_directory, default_value="./")]
	pub root: PathBuf,
}

pub struct TestResult {
	pub output: String,
	pub success: bool,
}

enum CacheStatus {
	Cached,
	Uncached,
}

pub struct CompiledCacheFile {
	path: PathBuf,
	status: CacheStatus,
}

#[derive(Serialize, Deserialize, Debug)]
struct CacheJson {
	contract_path: String,
	sha256: String,
}

fn compute_hash(filepath: &PathBuf) -> Result<String, String> {
	// hash filepath
	let mut hasher = Sha256::new();
	let mut file = File::open(filepath).map_err(|e| format!("Failed to open file: {}", e))?;
	io::copy(&mut file, &mut hasher).map_err(|e| format!("Failed to hash file: {}", e))?;
	let hash = hasher.finalize();
	return Ok(format!("{:x}", hash));
}

fn list_test_entrypoints(compiled_path: &PathBuf) -> Result<Vec<String>, String> {
	let re = Regex::new(r"__main__.(test_\w+)$").expect("Should be a valid regex");
	let data = std::fs::read_to_string(compiled_path)
		.map_err(|err| format!("File does not exist: {}", err))?;
	let json = serde_json::from_str::<Value>(&data)
		.map_err(|err| format!("Compilation output is not a valid JSON: {}", err))?;
	let mut test_entrypoints = Vec::new();

	let identifiers = json["identifiers"].as_object();
	match identifiers {
		Some(identifiers) => {
			for (key, value) in identifiers {
				if re.is_match(key) && value["type"] == "function" {
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
		},
		None => eprintln!("Compilation output does not contain identifiers"),
	}

	Ok(test_entrypoints)
}

/// Execute command output
#[derive(Debug, Serialize, Default)]
pub struct TestOutput(String);

impl Display for TestOutput {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", &self.0)
	}
}

fn setup_hint_processor() -> BuiltinHintProcessor {
	let skip_hint = HintFunc(Box::new(hints::skip));
	let mock_call_hint = HintFunc(Box::new(hints::mock_call));
	let expect_revert_hint = HintFunc(Box::new(expect_revert));
	let mut hint_processor = BuiltinHintProcessor::new_empty();
	hint_processor.add_hint(String::from("skip()"), skip_hint);
	hint_processor.add_hint(String::from("expect_revert()"), expect_revert_hint);
	hint_processor.add_hint(
		String::from("mock_call(func_to_mock, mock_ret_value)"),
		mock_call_hint,
	);
	hint_processor
}

fn setup_hooks() -> Hooks {
	Hooks::new(Arc::new(hooks::pre_step_instruction))
}

fn list_cairo_files(root: &PathBuf) -> Result<Vec<PathBuf>, String> {
	ListArgs {
		root: PathBuf::from(root),
	}
	.exec()
	.map(|cmd_output: ListOutput| cmd_output.files)
}

fn read_json_file(path: &PathBuf) -> Result<CacheJson, String> {
	// Open the file in read-only mode with buffer.
	let file = File::open(path).map_err(|op| format!("file does not exists {}", op))?;
	let reader = BufReader::new(file);
	let data =
		serde_json::from_reader(reader).map_err(|op| format!("file does not exists {}", op))?;
	return Ok(data);
}

fn compile_and_list_entrypoints(
	cache: Result<CompiledCacheFile, String>,
) -> Option<(PathBuf, PathBuf, Vec<String>)> {
	match cache {
		Ok(cache) => match cache.status {
			CacheStatus::Cached => {
				println!("Using cached compiled file");
				let compiled_path = cache.path.clone();
				let entrypoints =
					list_test_entrypoints(&cache.path).expect("Failed to list entrypoints");
				return Some((cache.path, compiled_path, entrypoints));
			},
			CacheStatus::Uncached => {
				let compiled_path = compile(&cache.path).expect("Failed to compile");
				let entrypoints =
					list_test_entrypoints(&compiled_path).expect("Failed to list entrypoints");
				return Some((cache.path, compiled_path, entrypoints));
			},
		},
		Err(err) => {
			eprintln!("{}", err);
			return None;
		},
	}
}

fn create_compiled_contract_path(path_to_code: &PathBuf) -> PathBuf {
	let filename = path_to_code.file_stem().expect("File does not have a file stem");

	let cache_dir = dirs::cache_dir().expect("Could not make cache directory");
	let mut path_to_compiled = PathBuf::new();
	path_to_compiled.push(&cache_dir);
	path_to_compiled.push("compiled-cairo-files");
	path_to_compiled.push(filename);
	path_to_compiled.set_extension("json");
	return path_to_compiled;
}

fn dump_json_file(path: &PathBuf, data: &CacheJson) -> Result<(), String> {
	let file = File::create(path).map_err(|op| format!("file does not exists {}", op))?;
	let writer = BufWriter::new(file);
	serde_json::to_writer_pretty(writer, data)
		.map_err(|op| format!("file does not exists {}", op))?;
	return Ok(());
}

fn read_cache(path_to_code: PathBuf) -> Result<CompiledCacheFile, String> {
	// read individual cache file
	// avoid same cache file because we're doing multiprocessing and getting race condition
	let cache_dir = dirs::cache_dir().expect("cache dir not supported");
	let filename = path_to_code.file_stem().unwrap().to_str().unwrap();

	let mut cache_path = PathBuf::new();
	cache_path.push(&cache_dir);
	cache_path.push("cairo-foundry-cache");

	// create dir to store cache files
	std::fs::create_dir_all(&cache_path).expect("Could not make cache directory");
	cache_path.push(format!("{}.json", filename));

	let data = read_json_file(&cache_path);
	// compute hash from file
	let hash_calculated = compute_hash(&path_to_code).unwrap();
	let contract_path = path_to_code.to_str().unwrap().to_string();

	match data {
		// json file exists
		Ok(cache_data) => {
			let compiled_contract_path = create_compiled_contract_path(&path_to_code);
			let hash_in_cache = cache_data.sha256;
			if *hash_in_cache == hash_calculated {
				return Ok(CompiledCacheFile {
					path: compiled_contract_path,
					status: CacheStatus::Cached,
				});
			} else {
				let data = CacheJson {
					contract_path,
					sha256: hash_calculated,
				};

				dump_json_file(&cache_path, &data)?;
				return Ok(CompiledCacheFile {
					path: path_to_code,
					status: CacheStatus::Uncached,
				});
			}
		},

		// json file does not exists
		Err(_) => {
			let data = CacheJson {
				contract_path,
				sha256: hash_calculated,
			};
			dump_json_file(&cache_path, &data)?;
			return Ok(CompiledCacheFile {
				path: path_to_code,
				status: CacheStatus::Uncached,
			});
		},
	}
}

fn purge_hint_buffer(execution_uuid: &Uuid, output: &mut String) {
	// Safe to unwrap as long as `init_buffer` has been called before
	let buffer = get_buffer(execution_uuid).unwrap();
	if !buffer.is_empty() {
		output.push_str(&format!("[{}]:\n{}", "captured stdout".blue(), buffer));
	}
	clear_buffer(execution_uuid);
}

pub(crate) fn test_single_entrypoint(
	path_to_compiled: &PathBuf,
	test_entrypoint: String,
	hint_processor: &BuiltinHintProcessor,
	hooks: Option<Hooks>,
) -> (String, bool) {
	let start = Instant::now();
	let mut output = String::new();
	let execution_uuid = Uuid::new_v4();
	init_buffer(execution_uuid);
	let res_cairo_run = cairo_run(
		&path_to_compiled,
		&test_entrypoint,
		false,
		false,
		hint_processor,
		execution_uuid,
		hooks,
	);
	let duration = start.elapsed();
	let (opt_runner_and_output, test_success) = match res_cairo_run {
		Ok(res) => {
			output.push_str(&format!(
				"[{}] {} ({:?})\n",
				"OK".green(),
				test_entrypoint,
				duration
			));
			(Some(res), true)
		},
		Err(CairoRunError::VirtualMachine(VirtualMachineError::CustomHint(
			custom_error_message,
		))) if custom_error_message == "skip" => {
			output.push_str(&format!("[{}] {}\n", "SKIPPED".yellow(), test_entrypoint,));
			(None, true)
		},
		Err(CairoRunError::VirtualMachine(VirtualMachineError::CustomHint(
			custom_error_message,
		))) if custom_error_message == EXPECT_REVERT_FLAG => {
			output.push_str(&format!(
				"[{}] {}\nError: execution did not revert while expect_revert() was specified\n\n",
				"FAILED".red(),
				test_entrypoint,
			));
			(None, false)
		},
		Err(e) => {
			output.push_str(&format!(
				"[{}] {}\nError: {}\n\n",
				"FAILED".red(),
				test_entrypoint,
				e
			));
			(None, false)
		},
	};

	purge_hint_buffer(&execution_uuid, &mut output);
	let (mut runner, mut vm) = match opt_runner_and_output {
		Some(runner_and_vm) => runner_and_vm,
		None => return (output, test_success),
	};

	// Display the exectution output if present
	match runner.get_output(&mut vm) {
		Ok(runner_output) => {
			if !runner_output.is_empty() {
				output.push_str(&format!(
					"[{}]:\n{}",
					"execution output".purple(),
					&runner_output
				));
			}
		},
		Err(e) => eprintln!("failed to get output from the cairo runner: {e}"),
	};

	output.push('\n');
	(output, test_success)
}

fn run_tests_for_one_file(
	hint_processor: &BuiltinHintProcessor,
	path_to_original: PathBuf,
	path_to_compiled: PathBuf,
	test_entrypoints: Vec<String>,
	hooks: Hooks,
) -> TestResult {
	let (tests_output, tests_success) = test_entrypoints
		.into_iter()
		.map(|test_entrypoint| {
			test_single_entrypoint(
				&path_to_compiled,
				test_entrypoint,
				&hint_processor,
				Some(hooks.clone()),
			)
		})
		.fold((String::new(), true), |mut a, b| {
			a.0.push_str(&b.0);
			a.1 &= b.1;
			a
		});

	TestResult {
		output: format!(
			"Running tests in file {}\n{}",
			path_to_original.display(),
			tests_output
		),
		success: tests_success,
	}
}

impl CommandExecution<TestOutput> for TestArgs {
	fn exec(&self) -> Result<TestOutput, String> {
		// Declare hints
		let hint_processor = setup_hint_processor();
		let hooks = setup_hooks();

		list_cairo_files(&self.root)?
			.into_par_iter()
			.map(|op| read_cache(op))
			.filter_map(compile_and_list_entrypoints)
			.map(|(path_to_original, path_to_compiled, test_entrypoints)| {
				run_tests_for_one_file(
					&hint_processor,
					path_to_original,
					path_to_compiled,
					test_entrypoints,
					hooks.clone(),
				)
			})
			.for_each(|test_result| println!("{}", test_result.output));

		Ok(Default::default())
	}
}
