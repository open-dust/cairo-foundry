use lazy_static::lazy_static;
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

pub fn write_to_output_buffer(execution_uuid: &Uuid, data: &str) {
	let mut hashmap_lock = HINT_OUTPUT_BUFFER.write().unwrap();
	let opt_buffer = hashmap_lock.get_mut(execution_uuid);
	if let Some(buffer) = opt_buffer {
		buffer.push_str(data);
	}
}
