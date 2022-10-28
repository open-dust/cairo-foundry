use std::path::Path;

use cairo_rs::{
	bigint,
	hint_processor::hint_processor_definition::HintProcessor,
	types::program::Program,
	vm::{errors::cairo_run_errors::CairoRunError, runners::cairo_runner::CairoRunner},
};
use num_bigint::BigInt;
use uuid::Uuid;

use crate::hints::EXECUTION_UUID_VAR_NAME;

pub fn cairo_run<'a>(
	path: &'a Path,
	entrypoint: &'a str,
	trace_enabled: bool,
	hint_processor: &'a dyn HintProcessor,
	execution_uudi: Uuid,
) -> Result<CairoRunner<'a>, CairoRunError> {
	let program = match Program::new(path, entrypoint) {
		Ok(program) => program,
		Err(error) => return Err(CairoRunError::Program(error)),
	};

	let mut cairo_runner = CairoRunner::new(&program, trace_enabled, hint_processor)?;
	cairo_runner.initialize_segments(None);

	let end = match cairo_runner.initialize_main_entrypoint() {
		Ok(end) => end,
		Err(error) => return Err(CairoRunError::Runner(error)),
	};

	cairo_runner.exec_scopes.assign_or_update_variable(
		EXECUTION_UUID_VAR_NAME,
		Box::new(bigint!(execution_uudi.as_u128())),
	);
	if let Err(error) = cairo_runner.initialize_vm() {
		return Err(CairoRunError::Runner(error))
	}

	if let Err(error) = cairo_runner.run_until_pc(end) {
		return Err(CairoRunError::VirtualMachine(error))
	}

	if let Err(error) = cairo_runner.vm.verify_auto_deductions() {
		return Err(CairoRunError::VirtualMachine(error))
	}

	if let Err(error) = cairo_runner.relocate() {
		return Err(CairoRunError::Trace(error))
	}

	Ok(cairo_runner)
}
