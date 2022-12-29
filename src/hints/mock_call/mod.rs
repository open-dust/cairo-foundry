use std::collections::HashMap;

use cairo_rs::{
	hint_processor::{
		builtin_hint_processor::hint_utils::{get_integer_from_var_name, get_ptr_from_var_name},
		hint_processor_definition::HintReference,
	},
	serde::deserialize_program::ApTracking,
	types::{exec_scope::ExecutionScopes, relocatable::Relocatable},
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

#[cfg(test)]
mod tests;

pub const MOCK_CALL_KEY: &str = "mock_call";
pub const MOCK_CALL_FELT_KEY: &str = "mock_call_felt";

pub fn mock_call(
	vm: &mut VirtualMachine,
	exec_scopes: &mut ExecutionScopes,
	ids_data: &HashMap<String, HintReference>,
	ap_tracking: &ApTracking,
	_constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
	let func_to_mock = get_ptr_from_var_name("func_to_mock", vm, ids_data, ap_tracking)?;
	let mock_value_len = get_integer_from_var_name("mock_value_len", vm, ids_data, ap_tracking)?;
	let mock_value = get_ptr_from_var_name("mock_value", vm, ids_data, ap_tracking)?;

	let mocks = exec_scopes
		.get_any_boxed_mut(MOCK_CALL_KEY)?
		.downcast_mut::<HashMap<usize, (usize, Relocatable)>>()
		.ok_or_else(|| VirtualMachineError::VariableNotInScopeError(MOCK_CALL_KEY.to_string()))?;

	mocks.insert(
		func_to_mock.offset,
		((*mock_value_len).to_usize().unwrap_or_default(), mock_value),
	);
	Ok(())
}

pub fn mock_call_felt(
	vm: &mut VirtualMachine,
	exec_scopes: &mut ExecutionScopes,
	ids_data: &HashMap<String, HintReference>,
	ap_tracking: &ApTracking,
	_constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
	let func_to_mock = get_ptr_from_var_name("func_to_mock", vm, ids_data, ap_tracking)?;
	let mock_ret_value = get_integer_from_var_name("mock_ret_value", vm, ids_data, ap_tracking)?;

	let mocks = exec_scopes
		.get_any_boxed_mut(MOCK_CALL_FELT_KEY)?
		.downcast_mut::<HashMap<usize, BigInt>>()
		.ok_or_else(|| {
			VirtualMachineError::VariableNotInScopeError(MOCK_CALL_FELT_KEY.to_string())
		})?;
	mocks.insert(func_to_mock.offset, (*mock_ret_value).clone());

	Ok(())
}
