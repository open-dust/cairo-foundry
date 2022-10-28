use cairo_rs::{
	hint_processor::{
		builtin_hint_processor::hint_utils::get_integer_from_var_name,
		hint_processor_definition::HintReference,
		proxies::{exec_scopes_proxy::ExecutionScopesProxy, vm_proxy::VMProxy},
	},
	serde::deserialize_program::ApTracking,
	vm::errors::vm_errors::VirtualMachineError,
};
use lazy_static::lazy_static;
use num_traits::cast::ToPrimitive;
use std::{collections::HashMap, sync::RwLock};
use uuid::Uuid;

pub const EXECUTION_UUID_VAR_NAME: &str = "cairo-foundry-execution-uuid";

lazy_static! {
	static ref HINT_OUTPUT_BUFFER: RwLock<HashMap<Uuid, String>> = RwLock::new(HashMap::new());
}

pub fn init_buffer(execution_uuid: Uuid) {
	HINT_OUTPUT_BUFFER.write().unwrap().insert(execution_uuid, String::new());
}

pub fn clear_buffer(execution_uuid: &Uuid) {
	HINT_OUTPUT_BUFFER.write().unwrap().remove(execution_uuid);
}

pub fn get_buffer(execution_uuid: &Uuid) -> Option<String> {
	HINT_OUTPUT_BUFFER.read().unwrap().get(execution_uuid).cloned()
}

fn write_to_output_buffer(execution_uuid: &Uuid, data: &str) {
	let mut hashmap_lock = HINT_OUTPUT_BUFFER.write().unwrap();
	let opt_buffer = hashmap_lock.get_mut(execution_uuid);
	if let Some(buffer) = opt_buffer {
		buffer.push_str(data);
	}
}

pub fn greater_than(
	vm_proxy: &mut VMProxy,
	exec_scopes_proxy: &mut ExecutionScopesProxy,
	ids_data: &HashMap<String, HintReference>,
	ap_tracking: &ApTracking,
) -> Result<(), VirtualMachineError> {
	let a = get_integer_from_var_name("a", vm_proxy, ids_data, ap_tracking)?;
	let b = get_integer_from_var_name("b", vm_proxy, ids_data, ap_tracking)?;
	let execution_uuid = Uuid::from_u128(
		exec_scopes_proxy.get_int(EXECUTION_UUID_VAR_NAME).unwrap().to_u128().unwrap(),
	);
	write_to_output_buffer(&execution_uuid, &format!("{}\n", a > b));
	Ok(())
}
