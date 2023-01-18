use std::rc::Rc;

use cairo_rs::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::{
	BuiltinHintProcessor, HintFunc,
};

use crate::hints;

/// Create, setup and return a HintProcessor supporting our custom hints
pub fn setup_hint_processor() -> BuiltinHintProcessor {
	let skip_hint = Rc::new(HintFunc(Box::new(hints::skip)));
	let mock_call_hint = Rc::new(HintFunc(Box::new(hints::mock_call)));
	let expect_revert_hint = Rc::new(HintFunc(Box::new(hints::expect_revert)));
	let mut hint_processor = BuiltinHintProcessor::new_empty();
	hint_processor.add_hint(String::from("skip()"), skip_hint);
	hint_processor.add_hint(String::from("expect_revert()"), expect_revert_hint);
	hint_processor.add_hint(
		String::from("mock_call(func_to_mock, mock_ret_value)"),
		mock_call_hint,
	);
	hint_processor
}
