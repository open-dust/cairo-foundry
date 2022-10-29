use std::collections::HashMap;

use cairo_rs::{
	hint_processor::hint_processor_definition::HintReference,
	serde::deserialize_program::ApTracking,
	types::exec_scope::ExecutionScopes,
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use num_bigint::BigInt;

pub fn expect_revert(
	vm: &mut VirtualMachine,
	_exec_scopes: &mut ExecutionScopes,
	_ids_data: &HashMap<String, HintReference>,
	_ap_tracking: &ApTracking,
	_constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
	match vm.step_instruction() {
		Ok(_) => Err(VirtualMachineError::CustomHint(
			"expect_revert_did_not_revert".to_string(),
		)),
		Err(_) => Err(VirtualMachineError::CustomHint(
			"except_revert_reverted".to_string(),
		)),
	}
}
