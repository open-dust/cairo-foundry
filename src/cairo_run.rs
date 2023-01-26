use std::collections::HashMap;

use cairo_rs::{
	bigint,
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
	hints::{output_buffer::EXECUTION_UUID_VAR_NAME, EXPECT_REVERT_FLAG, MOCK_CALL_KEY},
	hooks::{HOOKS_VAR_NAME, MAX_STEPS_VAR_NAME},
};

/// Execute a cairo program
///
/// A `CairoRunner` and a `VirtualMachine` will be created to execute the given `Program`.
/// Hint and `Hooks` (if any) will be applied by the `VirtualMachine`
///
/// When no error is encountered, returns the `CairoRunner` and `VirtualMachine`.
/// Otherwise, returns a `CairoRunError`
///
/// `cairo_run` is the last step after cairo files have been listed and compiled.
/// Each *test* functions will be executed by `cairo_run` with hooks and hints applied.
pub fn cairo_run(
	program: Program,
	hint_processor: &mut dyn HintProcessor,
	execution_uuid: Uuid,
	opt_hooks: Option<Hooks>,
	max_steps: u64,
) -> Result<(CairoRunner, VirtualMachine), CairoRunError> {
	// 2023-01-06: FIXME: avoid hardcoded default layout & proof mode ?
	let mut cairo_runner = CairoRunner::new(&program, "small", false)?;
	let mut vm = VirtualMachine::new(program.prime, false, program.error_message_attributes);
	let end = cairo_runner.initialize(&mut vm)?;

	cairo_runner
		.exec_scopes
		.insert_value(EXECUTION_UUID_VAR_NAME, bigint!(execution_uuid.as_u128()));
	if let Some(hooks) = opt_hooks {
		cairo_runner.exec_scopes.insert_value(HOOKS_VAR_NAME, hooks);
		cairo_runner.exec_scopes.insert_value(MAX_STEPS_VAR_NAME, max_steps);
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

	cairo_runner.end_run(false, false, &mut vm, hint_processor)?;
	vm.verify_auto_deductions().map_err(CairoRunError::VirtualMachine)?;

	cairo_runner.relocate(&mut vm).map_err(CairoRunError::Trace)?;

	Ok((cairo_runner, vm))
}
