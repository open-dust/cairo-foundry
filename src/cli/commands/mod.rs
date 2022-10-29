use crate::cli::formatter::Formattable;
use clap::Subcommand;
use serde::Serialize;
use std::fmt;

/// execute module: contains everything related to the `Execute` command
mod execute;
/// list module: contains everything related to the `List` command
mod list;
// test module: contains everything related to the `Test` command
pub mod test;

/// Enum of all supported commands
#[derive(Subcommand)]
pub enum Commands {
	/// List test files
	List(list::ListArgs),
	/// Execute compiled cairo program
	Execute(execute::ExecuteArgs),
	// Test cairo programs
	Test(test::TestArgs),
}

/// Bahaviour of a command
pub trait CommandExecution<F: Formattable> {
	fn exec(&self) -> Result<F, String>;
}

enum CommandOutputs {
	List(list::ListOutput),
	Execute(execute::ExecuteOutput),
	Test(test::TestOutput),
}

/// The executed command output
pub struct Output(CommandOutputs);

impl Serialize for Output {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match &self.0 {
			CommandOutputs::List(output) => output.serialize(serializer),
			CommandOutputs::Execute(output) => output.serialize(serializer),
			CommandOutputs::Test(output) => output.serialize(serializer),
		}
	}
}

impl fmt::Display for Output {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.0 {
			CommandOutputs::List(output) => output.fmt(f),
			CommandOutputs::Execute(output) => output.fmt(f),
			CommandOutputs::Test(output) => output.fmt(f),
		}
	}
}

impl CommandExecution<Output> for Commands {
	fn exec(&self) -> Result<Output, String> {
		match &self {
			Commands::List(args) => args.exec().map(|o| Output(CommandOutputs::List(o))),
			Commands::Execute(args) => args.exec().map(|o| Output(CommandOutputs::Execute(o))),
			Commands::Test(args) => args.exec().map(|o| Output(CommandOutputs::Test(o))),
		}
	}
}
