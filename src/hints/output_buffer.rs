use std::{collections::HashMap, sync::RwLock};

use lazy_static::lazy_static;
use uuid::Uuid;

pub const EXECUTION_UUID_VAR_NAME: &str = "cairo-foundry-execution-uuid";

lazy_static! {
	/// the `HINT_OUTPUT_BUFFER` is a hasmap<Uuid,String> protected from concurrency
	/// with a RwLock used by `fn test_single_entrypoint` to store the output of the `cairo_run`
	/// command for each test entrypoint.
	///
	/// Before test execution, an Uuid is associated for the test, test is executed
	/// and `cairo_run` output is captured in the `HINT_OUTPUT_BUFFER`
	///
	/// # Example:
	/// ```
	/// # use uuid::Uuid;
	/// # use cairo_foundry::hints::output_buffer::{clear_buffer, get_buffer, init_buffer, write_to_output_buffer};
	///
	/// let execution_uuid = Uuid::new_v4();
	///	init_buffer(execution_uuid);
	///
	/// write_to_output_buffer(&execution_uuid, "foo");
	/// let buffer = get_buffer(&execution_uuid).unwrap();
	/// assert_eq!(buffer, "foo");
	///
	/// clear_buffer(&execution_uuid);
	/// let buffer = get_buffer(&execution_uuid);
	/// assert_eq!(buffer, None);
	/// ```
	static ref HINT_OUTPUT_BUFFER: RwLock<HashMap<Uuid, String>> = RwLock::new(HashMap::new());
}

/// Insert a new key 'execution_uuid` in the HINT_OUTPUT_BUFFER HashMap with a default
/// empty String value.
/// Returns nothing.
///
/// The given `execution_uuid` is the one used to identify the cairo test entrypoint
pub fn init_buffer(execution_uuid: Uuid) {
	HINT_OUTPUT_BUFFER.write().unwrap().insert(execution_uuid, String::new());
}

/// Remove the key `execution_uuid` in the HINT_OUTPUT_BUFFER HashMap.
/// Returns nothing.
///
/// The given `execution_uuid` is the one used to identify the cairo test entrypoint
pub fn clear_buffer(execution_uuid: &Uuid) {
	HINT_OUTPUT_BUFFER.write().unwrap().remove(execution_uuid);
}

/// Returns the cloned value of key `execution_uuid` in the HINT_OUTPUT_BUFFER HashMap.
///
/// The given `execution_uuid` is the one used to identify the cairo test entrypoint
pub fn get_buffer(execution_uuid: &Uuid) -> Option<String> {
	HINT_OUTPUT_BUFFER.read().unwrap().get(execution_uuid).cloned()
}

/// Append string `data` to the value of key `execution_uuid` in the HINT_OUTPUT_BUFFER HashMap.
///
/// The given `execution_uuid` is the one used to identify the cairo test entrypoint
#[allow(dead_code)]
pub fn write_to_output_buffer(execution_uuid: &Uuid, data: &str) {
	let mut hashmap_lock = HINT_OUTPUT_BUFFER.write().unwrap();
	let opt_buffer = hashmap_lock.get_mut(execution_uuid);
	if let Some(buffer) = opt_buffer {
		buffer.push_str(data);
	}
}
