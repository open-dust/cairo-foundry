use std::{fmt::Display, path::PathBuf};

use clap::{Args, ValueHint};
use cleopatra_cairo::cairo_run;
use serde::Serialize;

use super::CommandExecution;
//----------------
use cairo_rs::cairo_run::cairo_run;
use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::{
    BuiltinHintProcessor, HintFunc,
};
use cairo_rs::hint_processor::builtin_hint_processor::hint_utils::get_integer_from_var_name;
use cairo_rs::hint_processor::hint_processor_definition::HintReference;
use cairo_rs::hint_processor::proxies::{
    exec_scopes_proxy::ExecutionScopesProxy, vm_proxy::VMProxy,
};
use cairo_rs::serde::deserialize_program::ApTracking;
use cairo_rs::vm::errors::vm_errors::VirtualMachineError;
use std::collections::HashMap;
use std::path::Path;
//-----------------

#[derive(Args, Debug)]
pub struct ExecuteArgs {
	/// Path to a json compiled cairo program
	#[clap(short, long, value_hint=ValueHint::FilePath, value_parser=is_json)]
	program: PathBuf,
}

fn is_json(path: &str) -> Result<PathBuf, String> {
	let path = PathBuf::from(path);
	if path.exists() && path.is_file() {
		match path.extension() {
			Some(ext) if ext == "json" => Ok(path),
			_ => Err(format!("\"{}\" is not a json file", path.display())),
		}
	} else {
		Err(format!("\"{}\" is not a valid file", path.display()))
	}
}

/// Execute command output
#[derive(Debug, Serialize)]
pub struct ExecuteOutput {}

impl Display for ExecuteOutput {
	fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Ok(())
	}
}

impl CommandExecution<ExecuteOutput> for ExecuteArgs {
	fn exec(&self) -> Result<ExecuteOutput, String> {
		let mut cairo_runner = cairo_run::cairo_run(&self.program).map_err(|e| {
			format!(
				"failed to run the program \"{}\": {}",
				self.program.display(),
				e,
			)
		})?;

		cairo_run::write_output(&mut cairo_runner).map_err(|e| {
			format!(
				"failed to print the program output \"{}\": {}",
				self.program.display(),
				e,
			)
		})?;

		Ok(ExecuteOutput {})
	}
}
//---------------------------------
// hint assertion test
fn less_than_a_hint(
    vm_proxy: &mut VMProxy,
    _exec_scopes_proxy: &mut ExecutionScopesProxy,
    ids_data: &HashMap<String, HintReference>,
    ap_tracking: &ApTracking,
) -> Result<(), VirtualMachineError> {
    let a = get_integer_from_var_name("a", vm_proxy, ids_data, ap_tracking)?;
    let b = get_integer_from_var_name("b", vm_proxy, ids_data, ap_tracking)?;
    println!("{}", a < b);
    Ok(())
}

fn main() {
    // Wrap the Rust hint implementation in a Box smart pointer inside a HintFunc 
    //let hint = HintFunc(Box::new(print_a_hint));
    let hint = HintFunc(Box::new(less_than_a_hint));

    //Instantiate the hint processor
    let mut hint_processor = BuiltinHintProcessor::new_empty();

    //Add the custom hint, together with the Python code
    hint_processor.add_hint(String::from("print(ids.a > ids.b)"), hint);

    //Run the cairo program
    cairo_run(
        Path::new("custom_hint.json"),
        "main",
        false,
        &hint_processor,
    )
    .expect("Couldn't run program");
}
//-------------------------------------------------------

#[cfg(test)]
mod test {
	use super::*;
	#[test]
	fn valid_programs() {
		assert!(ExecuteArgs {
			program: PathBuf::from(
				"./test_starknet_projects/compiled_programs/valid_program_a.json"
			),
		}
		.exec()
		.is_ok());

		assert!(ExecuteArgs {
			program: PathBuf::from(
				"./test_starknet_projects/compiled_programs/valid_program_b.json"
			),
		}
		.exec()
		.is_ok());
	}

	#[test]
	fn invalid_programs() {
		assert!(ExecuteArgs {
			program: PathBuf::from(
				"./test_starknet_projects/compiled_programs/invalid_odd_length_hex.json"
			),
		}
		.exec()
		.is_err());

		assert!(ExecuteArgs {
			program: PathBuf::from(
				"./test_starknet_projects/compiled_programs/invalid_even_length_hex.json"
			),
		}
		.exec()
		.is_err());
	}
}
