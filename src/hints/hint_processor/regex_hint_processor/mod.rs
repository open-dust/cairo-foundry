#[cfg(test)]
mod tests;
use regex::Regex;
use std::any::Any;
use std::collections::HashMap;

use crate::{
    any_box,
    serde::deserialize_program::ApTracking,
    types::exec_scope::ExecutionScopes,
    vm::{errors::vm_errors::VirtualMachineError, vm_core::VirtualMachine},
};

use super::hint_processor_definition::HintProcessor;

#[allow(clippy::type_complexity)]
pub struct HintFunc(
    pub  Box<
        dyn Fn(
            &mut VirtualMachine,
            &mut ExecutionScopes,
            Vec<String>,
        ) -> Result<(), VirtualMachineError>,
    >,
);

pub struct HintProcessorData {
    pub code: String,
}

pub struct RegexHintProcessor {
    hints: HashMap<String, HintFunc>,
}

impl RegexHintProcessor {
    pub fn new(hints: HashMap<String, HintFunc>) -> Self {
        Self { hints }
    }

    pub fn add_hint(&mut self, hint_code_regex: String, func: HintFunc) {
        self.hints.insert(hint_code_regex, func);
    }
}

impl HintProcessor for RegexHintProcessor {
    fn compile_hint(
        &self,
        hint_code: &str,
        _ap_tracking: &ApTracking,
        _reference_ids: &HashMap<String, usize>,
        _references: &HashMap<usize, super::hint_processor_definition::HintReference>,
    ) -> Result<Box<dyn std::any::Any>, VirtualMachineError> {
        Ok(any_box!(HintProcessorData {
            code: hint_code.to_string()
        }))
    }

    fn execute_hint(
        &self,
        vm: &mut crate::vm::vm_core::VirtualMachine,
        exec_scopes: &mut crate::types::exec_scope::ExecutionScopes,
        hint_data: &Box<dyn std::any::Any>,
        _constants: &HashMap<String, num_bigint::BigInt>,
    ) -> Result<(), VirtualMachineError> {
        let hint_data = hint_data
            .downcast_ref::<HintProcessorData>()
            .ok_or(VirtualMachineError::WrongHintData)?;

        for hint_regex_string in self.hints.keys() {
            let hint_regex = Regex::new(hint_regex_string.as_str())
                .map_err(|_| VirtualMachineError::WrongHintData)?;
            if hint_regex.is_match(&hint_data.code) {
                for captures in hint_regex.captures_iter(&hint_data.code) {
                    let args: Vec<String> = captures
                        .iter()
                        .skip(1) // skip the first capture, which is the whole match
                        .map(|x| x.unwrap().as_str().to_string())
                        .collect();
                    match self.hints.get(hint_regex_string) {
                        Some(func) => return func.0(vm, exec_scopes, args),
                        None => {}
                    }
                }
            }
        }
        Ok(())
    }
}

impl Default for RegexHintProcessor {
    fn default() -> Self {
        Self::new(HashMap::new())
    }
}