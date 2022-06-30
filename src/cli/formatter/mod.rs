use serde::Serialize;
use std::fmt::{self, Display};

pub mod json;
pub mod text;

pub trait Formatter<Output> {
	fn format(&self, output: &Output) -> String;
}

pub enum Formatters {
	Text(text::TextFormatter),
	JSON(json::JsonFormatter),
}

pub trait Formattable: fmt::Display + Serialize {}

impl<T> Formattable for T where T: Display + Serialize {}

pub fn make(args: &super::Args) -> Formatters {
	if args.json {
		Formatters::JSON(json::JsonFormatter {})
	} else {
		Formatters::Text(text::TextFormatter {})
	}
}

impl<Output> Formatter<Output> for Formatters
where
	Output: Formattable,
{
	fn format(&self, output: &Output) -> String {
		match self {
			Self::JSON(formatter) => formatter.format(output),
			Self::Text(formatter) => formatter.format(output),
		}
	}
}
