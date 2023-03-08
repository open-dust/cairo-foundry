use cairo_rs::types::program::Program;
use regex::Regex;
use std::collections::HashMap;

/// Retrieve return signatures for each function of a cairo program
///
/// Return an HashMap containing return signatures of each function:
/// - key is function symbol
/// - value is list of types present in return signatures
pub fn retrieve_return_signatures(program: &Program) -> HashMap<String, Vec<String>> {
	let re_tuple = Regex::new(r"^\(.*\)$").expect("Should be a valid regexp");
	let re_type =
		Regex::new(r"\w+\s*:\s*(?P<type_name>(\w+\.?)+\*?)").expect("Should be a valid regex");
	let mut signatures: HashMap<String, Vec<String>> = HashMap::new();
	for (key, value) in program.identifiers.iter() {
		if value.type_.as_deref() == Some("type_definition") {
			let mut return_types = Vec::new();
			let cairo_type = value.cairo_type.clone().unwrap();

			if re_tuple.is_match(&cairo_type) {
				if re_type.is_match(&cairo_type) {
					for capture in re_type.captures_iter(&cairo_type) {
						let captured = capture["type_name"].to_string();
						return_types.push(captured);
					}
				}
			} else {
				return_types.push(cairo_type.clone());
			}
			if key.ends_with(".Return") {
				signatures.insert(
					String::from(key.strip_suffix(".Return").unwrap()),
					return_types,
				);
			} else {
				signatures.insert(key.to_owned(), return_types);
			}
		}
	}
	signatures
}

#[cfg(test)]
mod test {
	use std::{fs::File, io::BufReader};

	use super::retrieve_return_signatures;
	use crate::compile::compile;
	use cairo_rs::serde::deserialize_program::deserialize_program;

	#[test]
	fn get_function_return_signatures() {
		let current_dir = std::env::current_dir().unwrap();
		let root_dir = current_dir.join("test_cairo_contracts");

		let contract_path = root_dir.join("test_retrieve_function_signatures.cairo");
		let compiled_contract_path = compile(&contract_path).unwrap();
		let file = File::open(compiled_contract_path).unwrap();
		let reader = BufReader::new(file);
		let program = deserialize_program(reader, None).unwrap();
		let signatures = retrieve_return_signatures(&program);

		assert_eq!(signatures["__main__.array_sum"], ["felt"]);
		assert_eq!(signatures["__main__.test_array_sum_negative"], [""; 0]);
		assert_eq!(
			signatures["__main__.get_account"],
			[
				"__main__.Account",
				"starkware.cairo.common.uint256.Uint256",
				"felt"
			]
		);
	}
}
