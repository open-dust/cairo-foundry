use std::collections::HashMap;

use cairo_rs::{
	types::{exec_scope::ExecutionScopes, instruction::Opcode, relocatable::Relocatable},
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use num_bigint::BigInt;

use crate::hints::{MOCK_CALL_FELT_KEY, MOCK_CALL_KEY};

pub const HOOKS_VAR_NAME: &str = "hooks";

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

	if instruction.opcode == Opcode::Call {
		let (operands, _operands_mem_addresses) = vm.compute_operands(&instruction)?;

		let new_pc = vm.compute_new_pc(&instruction, &operands)?;

		let mocks_felt = exec_scopes
			.get_any_boxed_mut(MOCK_CALL_FELT_KEY)?
			.downcast_mut::<HashMap<usize, BigInt>>()
			.ok_or_else(|| {
				VirtualMachineError::VariableNotInScopeError(MOCK_CALL_FELT_KEY.to_string())
			})?;

		if let Some(mocked_ret_value) = mocks_felt.get(&new_pc.offset) {
			let pc = vm.get_pc().clone();
			let ap = vm.get_ap();
			vm.insert_value(&ap, mocked_ret_value)?;
			let new_app = ap.add(1)?;
			vm.set_ap(new_app.offset);
			vm.set_pc(pc.add(2)?);
			vm.skip_next_instruction_execution();
		}

		let mocks = exec_scopes
			.get_any_boxed_mut(MOCK_CALL_KEY)?
			.downcast_mut::<HashMap<usize, (usize, Relocatable)>>()
			.ok_or_else(|| {
				VirtualMachineError::VariableNotInScopeError(MOCK_CALL_KEY.to_string())
			})?;

		if let Some((mocked_value_len, mocked_value)) = mocks.get(&new_pc.offset) {
			let pc = vm.get_pc().clone();
			let mocked_values = vm.get_integer_range(mocked_value, *mocked_value_len)?;
			let mut tmp_buffer = Vec::new();
			let old_ap = vm.get_ap();
			mocked_values.into_iter().for_each(|mocked_value_i| {
				tmp_buffer.push((*mocked_value_i).clone());
			});
			println!("{:?}", &tmp_buffer);
			tmp_buffer.into_iter().try_for_each(
				|tmp: BigInt| -> Result<(), VirtualMachineError> {
					let ap = vm.get_ap();
					vm.insert_value(&ap, tmp)?;
					let new_app = ap.add(1)?;
					vm.set_ap(new_app.offset);
					Ok(())
				},
			)?;
			let injected_values = vm.get_integer_range(&old_ap, *mocked_value_len);
			println!("{:?}", &injected_values);
			vm.set_pc(pc.add(2)?);
			vm.skip_next_instruction_execution();
		}
	}

	Ok(())
}
