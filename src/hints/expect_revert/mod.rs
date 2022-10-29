use std::collections::HashMap;

use cairo_rs::{
	hint_processor::hint_processor_definition::HintReference,
	serde::deserialize_program::ApTracking,
	types::exec_scope::ExecutionScopes,
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use num_bigint::BigInt;

#[cfg(test)]
mod tests;

pub const EXPECT_REVERT_FLAG: &str = "expect_revert";

pub fn expect_revert(
	_vm: &mut VirtualMachine,
	exec_scopes: &mut ExecutionScopes,
	_ids_data: &HashMap<String, HintReference>,
	_ap_tracking: &ApTracking,
	_constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
	exec_scopes.assign_or_update_variable(EXPECT_REVERT_FLAG, Box::new(true));
	Ok(())
}
