use super::Formatter;
use std::fmt;

pub struct TextFormatter {}

impl<Output> Formatter<Output> for TextFormatter
where
	Output: fmt::Display,
{
	fn format(&self, output: &Output) -> String {
		format!("{}", output)
	}
}

#[cfg(test)]
mod test {
	use super::{Formatter, TextFormatter};
	use std::fmt;

	#[test]
	fn formatter_should_format_object() {
		struct MyObject {}
		impl fmt::Display for MyObject {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "Hello there!")
			}
		}

		assert_eq!("Hello there!", TextFormatter {}.format(&MyObject {}));
	}
}
