use std::{collections::HashMap, path::Path};

use cairo_rs::{
	bigint,
	cairo_run::write_output,
	hint_processor::hint_processor_definition::HintProcessor,
	types::program::Program,
	vm::{
		errors::{cairo_run_errors::CairoRunError, vm_errors::VirtualMachineError},
		hook::Hooks,
		runners::cairo_runner::CairoRunner,
		vm_core::VirtualMachine,
	},
};
use num_bigint::BigInt;
use uuid::Uuid;

use crate::{
	hints::{EXECUTION_UUID_VAR_NAME, EXPECT_REVERT_FLAG, MOCK_CALL_KEY},
	hooks::HOOKS_VAR_NAME,
};

pub fn cairo_pre_run<'a>(
	path: &'a Path,
	entrypoint: &'a str,
	trace_enabled: bool,
	print_output: bool,
	hint_processor: &'a dyn HintProcessor,
	execution_uudi: Uuid,
	opt_hooks: Option<Hooks>,
) -> Result<(CairoRunner, VirtualMachine), CairoRunError> {
	let program = match Program::new(path, entrypoint) {
		Ok(program) => program,
		Err(error) => return Err(CairoRunError::Program(error)),
	};

	cairo_run(
		program,
		trace_enabled,
		print_output,
		hint_processor,
		execution_uudi,
		opt_hooks,
	)
}

pub fn cairo_run<'a>(
	program: Program,
	trace_enabled: bool,
	print_output: bool,
	hint_processor: &'a dyn HintProcessor,
	execution_uudi: Uuid,
	opt_hooks: Option<Hooks>,
) -> Result<(CairoRunner, VirtualMachine), CairoRunError> {

	let mut cairo_runner = CairoRunner::new(&program)?;
	let mut vm = VirtualMachine::new(program.prime, trace_enabled);
	let end = cairo_runner.initialize(&mut vm)?;

	cairo_runner
		.exec_scopes
		.insert_value(EXECUTION_UUID_VAR_NAME, bigint!(execution_uudi.as_u128()));
	if let Some(hooks) = opt_hooks {
		cairo_runner.exec_scopes.insert_value(HOOKS_VAR_NAME, hooks);
	}

	// Init exec context for mock_call
	let hashmap: HashMap<usize, BigInt> = HashMap::new();
	cairo_runner.exec_scopes.insert_value(MOCK_CALL_KEY, hashmap);

	let execution_result = cairo_runner.run_until_pc(end, &mut vm, hint_processor);
	let should_revert = cairo_runner.exec_scopes.get_any_boxed_ref(EXPECT_REVERT_FLAG).is_ok();

	match execution_result {
		Ok(_) if should_revert => Err(VirtualMachineError::CustomHint(
			EXPECT_REVERT_FLAG.to_string(),
		)),
		Err(_) if should_revert => Ok(()),
		_ => execution_result,
	}
	.map_err(CairoRunError::VirtualMachine)?;

	vm.verify_auto_deductions().map_err(CairoRunError::VirtualMachine)?;

	cairo_runner.relocate(&mut vm).map_err(CairoRunError::Trace)?;

	if print_output {
		write_output(&mut cairo_runner, &mut vm)?;
	}

	Ok((cairo_runner, vm))
}
