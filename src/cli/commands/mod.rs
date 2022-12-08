use crate::cli::formatter::Formattable;
use clap::Subcommand;
use serde::Serialize;
use std::fmt;

/// list module: contains everything related to the `List` command
mod list;
// test module: contains everything related to the `Test` command
pub mod test;
// clean module: contains everything related to the `Clean` command
mod clean;

/// Enum of all supported commands
#[derive(Subcommand)]
pub enum Commands {
	/// List test files
	List(list::ListArgs),
	// Test cairo programs
	Test(test::TestArgs),
	// Cleans the cache files
	Clean(clean::CleanArgs),
}

/// Bahaviour of a command
pub trait CommandExecution<F: Formattable> {
	fn exec(&self) -> Result<F, String>;
}

enum CommandOutputs {
	List(list::ListOutput),
	Test(test::TestOutput),
	Clean(clean::CleanOutput),
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
			CommandOutputs::Test(output) => output.serialize(serializer),
			CommandOutputs::Clean(output) => output.serialize(serializer),
		}
	}
}

impl fmt::Display for Output {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.0 {
			CommandOutputs::List(output) => output.fmt(f),
			CommandOutputs::Test(output) => output.fmt(f),
			CommandOutputs::Clean(output) => output.fmt(f),
		}
	}
}

impl CommandExecution<Output> for Commands {
	fn exec(&self) -> Result<Output, String> {
		match &self {
			Commands::List(args) => args.exec().map(|o| Output(CommandOutputs::List(o))),
			Commands::Execute(args) => args.exec().map(|o| Output(CommandOutputs::Execute(o))),
			Commands::Test(args) => args.exec().map(|o| Output(CommandOutputs::Test(o))),
			Commands::Clean(args) => args.exec().map(|o| Output(CommandOutputs::Clean(o))),
		}
	}
}
