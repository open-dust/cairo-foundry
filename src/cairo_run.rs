use std::path::Path;

use cairo_rs::{
	bigint,
	cairo_run::write_output,
	hint_processor::hint_processor_definition::HintProcessor,
	types::program::Program,
	vm::{
		errors::{cairo_run_errors::CairoRunError, vm_errors::VirtualMachineError},
		runners::cairo_runner::CairoRunner,
		vm_core::VirtualMachine,
	},
};
use num_bigint::BigInt;
use uuid::Uuid;

use crate::hints::{EXECUTION_UUID_VAR_NAME, EXPECT_REVERT_FLAG};

pub fn cairo_run<'a>(
	path: &'a Path,
	entrypoint: &'a str,
	trace_enabled: bool,
	print_output: bool,
	hint_processor: &'a dyn HintProcessor,
	execution_uudi: Uuid,
) -> Result<(CairoRunner, VirtualMachine), CairoRunError> {
	let program = match Program::new(path, entrypoint) {
		Ok(program) => program,
		Err(error) => return Err(CairoRunError::Program(error)),
	};

	let mut cairo_runner = CairoRunner::new(&program)?;
	let mut vm = VirtualMachine::new(program.prime, trace_enabled);
	let end = cairo_runner.initialize(&mut vm)?;

	cairo_runner.exec_scopes.assign_or_update_variable(
		EXECUTION_UUID_VAR_NAME,
		Box::new(bigint!(execution_uudi.as_u128())),
	);

	let execution_result = cairo_runner.run_until_pc(end, &mut vm, hint_processor);

	match cairo_runner.exec_scopes.get_any_boxed_ref(EXPECT_REVERT_FLAG) {
		Ok(_) if execution_result.is_ok() => Err(CairoRunError::VirtualMachine(
			VirtualMachineError::CustomHint(EXPECT_REVERT_FLAG.to_string()),
		))?,
		Err(_) if execution_result.is_err() =>
			execution_result.map_err(CairoRunError::VirtualMachine)?,
		_ => {},
	}

	vm.verify_auto_deductions().map_err(CairoRunError::VirtualMachine)?;

	cairo_runner.relocate(&mut vm).map_err(CairoRunError::Trace)?;

	if print_output {
		write_output(&mut cairo_runner, &mut vm)?;
	}

	Ok((cairo_runner, vm))
}
