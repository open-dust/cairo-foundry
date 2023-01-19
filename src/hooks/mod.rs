#[cfg(test)]
mod tests;

use std::{collections::HashMap, ops::Add};

use cairo_rs::{
	types::{exec_scope::ExecutionScopes, instruction::Opcode},
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use num_bigint::BigInt;

use crate::hints::MOCK_CALL_KEY;

pub const HOOKS_VAR_NAME: &str = "hooks";
pub const MAX_STEPS_VAR_NAME: &str = "max_steps";

/// Called before an instruction is executed by the virtual machine (VM).
///
/// This function is called before the VM will execute an instruction.
/// The given `&mut VirtualMachine` allows the VM state to be modified
/// according to given `&mut ExecutionScopes`.
///
/// When no error is encountered, returns an empty success value.
/// Otherwise, returns a VirtualMachineError
///
/// `mock_call` hint is using `pre_step_instruction` to modify
/// VM allocation pointer (ap) and program counter (pc) in order to return mocked value.
pub fn pre_step_instruction(
	vm: &mut VirtualMachine,
	exec_scopes: &mut ExecutionScopes,
	_constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
	let instruction = vm.decode_current_instruction()?;

	ensure_max_steps_not_reached(vm, exec_scopes)?;

	if instruction.opcode == Opcode::Call {
		let (operands, _operands_mem_addresses, _deduced_operands) =
			vm.compute_operands(&instruction)?;

		let new_pc = vm.compute_new_pc(&instruction, &operands)?;

		let mocks = exec_scopes
			.get_any_boxed_mut(MOCK_CALL_KEY)?
			.downcast_mut::<HashMap<usize, BigInt>>()
			.ok_or_else(|| {
				VirtualMachineError::VariableNotInScopeError(MOCK_CALL_KEY.to_string())
			})?;

		if let Some(mocked_ret_value) = mocks.get(&new_pc.offset) {
			let pc = *vm.get_pc();
			let ap = vm.get_ap();
			vm.insert_value(&ap, mocked_ret_value)?;
			vm.set_pc(pc.add(2));
			vm.set_ap(ap.offset + 1);
			vm.skip_next_instruction_execution();
		}
	}

	Ok(())
}

/// Called after an instruction is executed by the virtual machine (VM).
///
/// This function is called after the VM have executed an instruction.
/// The given `&mut VirtualMachine` allows the VM state to be modified
/// according to given `&mut ExecutionScopes`.
///
/// When no error is encountered, returns an empty success value.
/// Otherwise, returns a VirtualMachineError
pub fn post_step_instruction(
	_vm: &mut VirtualMachine,
	_exec_scopes: &mut ExecutionScopes,
	_constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
	Ok(())
}

pub fn ensure_max_steps_not_reached(
	vm: &mut VirtualMachine,
	exec_scopes: &mut ExecutionScopes,
) -> Result<(), VirtualMachineError> {
	if *vm.get_current_step() >= exec_scopes.get::<u64>(MAX_STEPS_VAR_NAME)? as usize {
		// TODO: find a better way to express custom errors
		Err(VirtualMachineError::CustomHint(format!(
			"max_steps reached: {}",
			*vm.get_current_step()
		)))
	} else {
		Ok(())
	}
}
