use std::collections::HashMap;

use cairo_rs::{
	types::{exec_scope::ExecutionScopes, instruction::Opcode},
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use num_bigint::BigInt;

use crate::hints::MOCK_CALL_KEY;

pub const HOOKS_VAR_NAME: &str = "hooks";

pub fn pre_step_instruction(
	vm: &mut VirtualMachine,
	exec_scopes: &mut ExecutionScopes,
	_constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
	let instruction = vm.decode_current_instruction()?;
	if instruction.opcode == Opcode::Call {
		let (operands, _operands_mem_addresses) = vm.compute_operands(&instruction)?;

		let new_pc = vm.compute_new_pc(&instruction, &operands)?;

		let mocks = exec_scopes
			.get_any_boxed_mut(MOCK_CALL_KEY)?
			.downcast_mut::<HashMap<usize, BigInt>>()
			.ok_or_else(|| {
				VirtualMachineError::VariableNotInScopeError(MOCK_CALL_KEY.to_string())
			})?;

		if let Some(mocked_ret_value) = mocks.get(&new_pc.offset) {
			let pc = vm.get_pc().clone();
			let ap = vm.get_ap();
			vm.insert_value(&ap, mocked_ret_value)?;
			vm.set_pc(pc.add(2)?);
			vm.set_ap(ap.offset + 1);
			vm.skip_next_instruction_execution();
		}
	}

	Ok(())
}
