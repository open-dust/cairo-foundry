use cairo_rs::{
	any_box,
	hint_processor::{
		builtin_hint_processor::builtin_hint_processor_definition::{
			BuiltinHintProcessor, HintProcessorData,
		},
		hint_processor_definition::{HintProcessor, HintReference},
	},
	serde::deserialize_program::ApTracking,
	types::exec_scope::ExecutionScopes,
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use num_bigint::BigInt;
use std::{any::Any, collections::HashMap, rc::Rc};

#[cfg(test)]
mod tests;

#[allow(clippy::type_complexity)]
pub struct HintFunc(
	pub  Box<
		dyn Fn(
				&mut VirtualMachine,
				&mut ExecutionScopes,
				&HashMap<String, HintReference>,
				&ApTracking,
				&HashMap<String, BigInt>,
				&[String],
			) -> Result<(), VirtualMachineError>
			+ Sync,
	>,
);

pub enum Code {
	RawCode(String),
	Function(String, Vec<String>), // (name, args)
}

pub struct FunctionLikeHintProcessorData {
	pub code: Code,
	pub ap_tracking: ApTracking,
	pub ids_data: HashMap<String, HintReference>,
}

pub struct FunctionLikeHintProcessor {
	hints: HashMap<String, Rc<HintFunc>>,
	builtin_hint_processor: BuiltinHintProcessor,
}

impl FunctionLikeHintProcessor {
	pub fn new_empty() -> Self {
		FunctionLikeHintProcessor {
			hints: HashMap::new(),
			builtin_hint_processor: BuiltinHintProcessor::new_empty(),
		}
	}

	pub fn new(hints: HashMap<String, Rc<HintFunc>>) -> Self {
		FunctionLikeHintProcessor {
			hints,
			builtin_hint_processor: BuiltinHintProcessor::new_empty(),
		}
	}

	pub fn add_hint(&mut self, hint_code_function_like: String, func: Rc<HintFunc>) {
		self.hints.insert(hint_code_function_like, func);
	}
}

impl HintProcessor for FunctionLikeHintProcessor {
	fn execute_hint(
		&mut self,
		vm: &mut VirtualMachine,
		exec_scopes: &mut ExecutionScopes,
		hint_data: &Box<dyn std::any::Any>,
		constants: &HashMap<String, num_bigint::BigInt>,
	) -> Result<(), VirtualMachineError> {
		let hint_data = hint_data
			.downcast_ref::<FunctionLikeHintProcessorData>()
			.ok_or(VirtualMachineError::WrongHintData)?;

		match &hint_data.code {
			Code::RawCode(raw_code) => self.builtin_hint_processor.execute_hint(
				vm,
				exec_scopes,
				&any_box!(HintProcessorData {
					code: raw_code.clone(),
					ap_tracking: hint_data.ap_tracking.clone(),
					ids_data: hint_data.ids_data.clone(),
				}),
				constants,
			)?,
			Code::Function(name, args) => {
				let ptr_hint_func =
					self.hints.get(name).ok_or(VirtualMachineError::WrongHintData)?;
				ptr_hint_func.0(
					vm,
					exec_scopes,
					&hint_data.ids_data.clone(),
					&hint_data.ap_tracking.clone(),
					constants,
					args,
				)?
			},
		};

		Ok(())
	}

	fn compile_hint(
		&self,
		hint_code: &str,
		ap_tracking: &ApTracking,
		reference_ids: &HashMap<String, usize>,
		references: &HashMap<usize, HintReference>,
	) -> Result<Box<dyn std::any::Any>, VirtualMachineError> {
		let hint_code_trim = hint_code.trim();
		let index_of_opening_parenthesis = match hint_code_trim.find('(') {
			None =>
				return Ok(any_box!(FunctionLikeHintProcessorData {
					code: Code::RawCode(hint_code.to_string()),
					ap_tracking: ap_tracking.clone(),
					ids_data: get_ids_data(reference_ids, references)?,
				})),
			Some(i) => i,
		};

		if !hint_code_trim.ends_with(')') {
			return Ok(any_box!(FunctionLikeHintProcessorData {
				code: Code::RawCode(hint_code.to_string()),
				ap_tracking: ap_tracking.clone(),
				ids_data: get_ids_data(reference_ids, references)?,
			}))
		}
		let name_func = hint_code_trim[..index_of_opening_parenthesis].to_string();
		let list_args = hint_code_trim[index_of_opening_parenthesis + 1..]
			.split(',')
			.map(|x| x.to_string())
			.collect();

		Ok(any_box!(FunctionLikeHintProcessorData {
			code: Code::Function(name_func, list_args),
			ap_tracking: ap_tracking.clone(),
			ids_data: get_ids_data(reference_ids, references)?,
		}))
	}
}

fn get_ids_data(
	reference_ids: &HashMap<String, usize>,
	references: &HashMap<usize, HintReference>,
) -> Result<HashMap<String, HintReference>, VirtualMachineError> {
	let mut ids_data = HashMap::<String, HintReference>::new();
	for (path, ref_id) in reference_ids {
		let name = path.rsplit('.').next().ok_or(VirtualMachineError::FailedToGetIds)?;
		ids_data.insert(
			name.to_string(),
			references.get(ref_id).ok_or(VirtualMachineError::FailedToGetIds)?.clone(),
		);
	}
	Ok(ids_data)
}

impl Default for FunctionLikeHintProcessor {
	fn default() -> Self {
		Self::new(HashMap::new())
	}
}
