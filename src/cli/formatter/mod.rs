use std::fmt::{self, Display};

pub mod text;

pub trait Formatter<Output> {
	fn format(&self, output: &Output) -> String;
}

pub trait Formattable: fmt::Display {}

impl<T> Formattable for T where T: Display {}
