#[cfg(test)]
mod tests;

use std::collections::HashMap;

use cairo_rs::{
	hint_processor::{
		builtin_hint_processor::hint_utils::{get_integer_from_var_name, get_ptr_from_var_name},
		hint_processor_definition::HintReference,
	},
	serde::deserialize_program::ApTracking,
	types::exec_scope::ExecutionScopes,
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use num_bigint::BigInt;

pub const MOCK_CALL_KEY: &str = "mock_call";

/// `mock_call` mocks the return value of a specific function.
///  This function is designed to be a hint, executable by the HintExecutor of the cairoVM
///
/// ## Parameters
///
/// * `vm`: a mutable reference to the VirtualMachine, which is responsible for executing the code.
/// * `exec_scopes`: a mutable reference to the ExecutionScopes, which stores variables and other
///   data for a specific scope of execution.
/// * `ids_data`: a reference to a hash map containing the cairo runtime identifiers and their
///   corresponding data.
/// * `ap_tracking`: a reference to the ApTracking, which keeps track of the allocation pointer
///   updates.
/// * `_constants`: a reference to a hash map containing constant values.
///
///
/// # Examples
///
/// ```no_run
/// # use std::rc::Rc;
/// # use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::{BuiltinHintProcessor, HintFunc,};
/// # use crate::hints;
/// # fn foo() ->BuiltinHintProcessor {
/// 	let mock_call_hint = Rc::new(HintFunc(Box::new(hints::mock_call)));
///     let mut hint_processor = BuiltinHintProcessor::new_empty();
/// 	hint_processor.add_hint(String::from("mock_call"), mock_call_hint);
/// # 	hint_processor
/// #  }
/// ```
pub fn mock_call(
	vm: &mut VirtualMachine,
	exec_scopes: &mut ExecutionScopes,
	ids_data: &HashMap<String, HintReference>,
	ap_tracking: &ApTracking,
	_constants: &HashMap<String, BigInt>,
) -> Result<(), VirtualMachineError> {
	let func_to_mock = get_ptr_from_var_name("func_to_mock", vm, ids_data, ap_tracking)?;
	let mock_ret_value = get_integer_from_var_name("mock_ret_value", vm, ids_data, ap_tracking)?;

	let mocks = exec_scopes
		.get_any_boxed_mut(MOCK_CALL_KEY)?
		.downcast_mut::<HashMap<usize, BigInt>>()
		.ok_or_else(|| VirtualMachineError::VariableNotInScopeError(MOCK_CALL_KEY.to_string()))?;
	mocks.insert(func_to_mock.offset, (*mock_ret_value).clone());

	Ok(())
}
