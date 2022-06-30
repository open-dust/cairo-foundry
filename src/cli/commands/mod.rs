use std::fmt::Display;

use clap::Subcommand;
use log::error;

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
	type Output: Display;
	fn exec(&self) -> Result<Self::Output, String>;
}

impl Commands {
	pub fn exec(&self) {
		let result = match &self {
			Commands::List(args) => args.exec(),
		};

		match result {
			Ok(output) => println!("{}", output),
			Err(err) => error!("{}", err),
		};
	}
}
