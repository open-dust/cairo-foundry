use super::Formatter;
use serde::Serialize;
use serde_json;

pub struct JsonFormatter {}

impl<Output> Formatter<Output> for JsonFormatter
where
	Output: Serialize,
{
	fn format(&self, output: &Output) -> String {
		serde_json::to_string(&output).expect("Unable to format output to JSON")
	}
}

#[cfg(test)]
mod test {
	use super::{Formatter, JsonFormatter};
	use serde::Serialize;

	#[test]
	fn formatter_should_format_object() {
		#[derive(Serialize)]
		struct MyObject {
			greeting: String,
		}

		assert_eq!(
			"{\"greeting\":\"Hi!\"}",
			JsonFormatter {}.format(&MyObject {
				greeting: "Hi!".into()
			})
		);
	}
}
