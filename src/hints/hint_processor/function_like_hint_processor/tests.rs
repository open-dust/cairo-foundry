use assert_matches::assert_matches;
use cairo_rs::{
	hint_processor::hint_processor_definition::HintProcessor,
	types::exec_scope::ExecutionScopes,
	vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};
use std::{collections::HashMap, rc::Rc};

use crate::{
	hints,
	hints::hint_processor::function_like_hint_processor::{FunctionLikeHintProcessor, HintFunc},
};
use rstest::rstest;

#[rstest]
#[case("expect_revert)'This should revert'(")]
fn test_function_like_hint_should_return_unknown_hint_error(
	#[case] hint_code: &str,
) -> Result<(), VirtualMachineError> {
	let mut hint_processor = FunctionLikeHintProcessor::default();
	let expect_revert_hint = Rc::new(HintFunc(Box::new(hints::expect_revert)));
	hint_processor.add_hint(String::from("expect_revert"), expect_revert_hint);

	let hint_data = hint_processor
		.compile_hint(
			hint_code,
			&Default::default(),
			&Default::default(),
			&Default::default(),
		)
		.unwrap();

	let result = hint_processor.execute_hint(
		&mut VirtualMachine::new(Default::default(), true, vec![]),
		&mut ExecutionScopes::new(),
		&hint_data,
		&HashMap::new(),
	);

	assert_matches!(result, Err(VirtualMachineError::UnknownHint(code)) if code == *hint_code.to_string());
	Ok(())
}
