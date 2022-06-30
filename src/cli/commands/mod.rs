use crate::cli::formatter::Formattable;
use clap::Subcommand;

/// list module: contains everything related to the `List` command
pub mod list;

/// Enum of all supported commands
#[derive(Subcommand)]
pub enum Commands {
	/// List test files
	List(list::List),
}

/// Bahaviour of a command
pub trait Command {
	type Output: Formattable;
	fn exec(&self) -> Result<Self::Output, String>;
}

impl Commands {
	pub fn exec(&self) -> Result<impl Formattable, String> {
		match &self {
			Commands::List(args) => args.exec(),
		}
	}
}
