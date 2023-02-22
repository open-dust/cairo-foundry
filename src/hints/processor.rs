use std::rc::Rc;

use crate::{
	hints,
	hints::hint_processor::function_like_hint_processor::{FunctionLikeHintProcessor, HintFunc},
};

/// Create, setup and return a HintProcessor supporting our custom hints
pub fn setup_hint_processor() -> FunctionLikeHintProcessor {
	let skip_hint = Rc::new(HintFunc(Box::new(hints::skip)));
	let mock_call_hint = Rc::new(HintFunc(Box::new(hints::mock_call)));
	let expect_revert_hint = Rc::new(HintFunc(Box::new(hints::expect_revert)));
	let mut hint_processor = FunctionLikeHintProcessor::new_empty();
	hint_processor.add_hint(String::from("skip"), skip_hint);
	hint_processor.add_hint(String::from("expect_revert"), expect_revert_hint);
	hint_processor.add_hint(String::from("mock_call"), mock_call_hint);
	hint_processor
}
