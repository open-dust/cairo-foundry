use crate::cli::formatter::Formattable;
use clap::Subcommand;
use serde::Serialize;
use std::{error, fmt};
use thiserror::Error;

/// list module: contains everything related to the `List` command
mod list;
// test module: contains everything related to the `Test` command
pub mod test;

#[derive(Error, Debug)]
// Todo: Think about a better way to bubble up the errors
#[allow(clippy::large_enum_variant)]
pub enum CommandError {
	#[error(transparent)]
	ListCommandError(#[from] list::ListCommandError),
	#[error(transparent)]
	TestCommandError(#[from] test::TestCommandError),
}

/// Enum of all supported commands
#[derive(Subcommand)]
pub enum Commands {
	/// List test files
	List(list::ListArgs),
	// Test cairo programs
	Test(test::TestArgs),
}

/// Behaviour of a command
pub trait CommandExecution<F: Formattable, E: error::Error + Into<CommandError>> {
	fn exec(&self) -> Result<F, E>;
}

enum CommandOutputs {
	List(list::ListOutput),
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
			CommandOutputs::Test(output) => output.serialize(serializer),
		}
	}
}

impl fmt::Display for Output {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.0 {
			CommandOutputs::List(output) => output.fmt(f),
			CommandOutputs::Test(output) => output.fmt(f),
		}
	}
}

impl CommandExecution<Output, CommandError> for Commands {
	fn exec(&self) -> Result<Output, CommandError> {
		match &self {
			Commands::List(args) =>
				args.exec().map_err(|e| e.into()).map(|o| Output(CommandOutputs::List(o))),
			Commands::Test(args) =>
				args.exec().map_err(|e| e.into()).map(|o| Output(CommandOutputs::Test(o))),
		}
	}
}
