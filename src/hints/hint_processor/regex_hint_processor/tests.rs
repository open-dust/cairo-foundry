use std::collections::HashMap;

use crate::{
    hint_processor::regex_hint_processor::HintFunc,
    types::exec_scope::ExecutionScopes,
    vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};

#[deny(unused_imports)]
use super::HintProcessor;
use super::RegexHintProcessor;
use regex::Regex;

const REGEX: &str = r#"expect_revert\('([\w ]+)'\)"#;
const MY_HINT_CODE: &str = "expect_revert('This should revert')";

#[test]
fn test_hint_regex() {
    let regex = Regex::new(REGEX).unwrap();
    assert!(regex.is_match(MY_HINT_CODE));
}

#[test]
fn test_custom_regex_hints() {
    let mut hint_processor = RegexHintProcessor::default();
    fn hint_func(
        _vm: &mut VirtualMachine,
        _exec_scopes: &mut ExecutionScopes,
        args: Vec<String>,
    ) -> Result<(), VirtualMachineError> {
        assert_eq!(args[0], "This should revert");
        Ok(())
    }
    hint_processor.add_hint(REGEX.to_string(), HintFunc(Box::new(hint_func)));

    let hint_data = hint_processor
        .compile_hint(
            MY_HINT_CODE,
            &Default::default(),
            &Default::default(),
            &Default::default(),
        )
        .unwrap();

    hint_processor
        .execute_hint(
            &mut VirtualMachine::new(Default::default(), true),
            &mut ExecutionScopes::new(),
            &hint_data,
            &HashMap::new(),
        )
        .unwrap();
}