#[cfg(test)]
mod tests;

use std::collections::HashMap;

use cairo_rs::{
	hint_processor::hint_processor_definition::HintReference,
	serde::deserialize_program::ApTracking,
	types::exec_scope::ExecutionScopes,
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use num_bigint::BigInt;

/// Skip a test
/// You can use this hint to prepare tests for functionality that isn't completed yet.
///
/// Returns nothing
///
/// # Examples
///
/// Basic usage in a `.cairo` file:
///
/// ```cairo
/// func test_to_be_implemented_later() {
///     %{ skip() %}
/// }
/// ```
pub fn skip(
	_vm: &mut VirtualMachine,
	_exec_scopes: &mut ExecutionScopes,
	_ids_data: &HashMap<String, HintReference>,
	_ap_tracking: &ApTracking,
	_constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
	Err(VirtualMachineError::CustomHint("skip".to_string()))
}
