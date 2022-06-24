use clap::Subcommand;
use log::info;

pub mod list;

#[derive(Subcommand)]
pub enum Commands {
	/// List test files
	List(list::List),
}

impl Commands {
	pub fn exec(&self) {
		match &self {
			Commands::List(args) => info!("Listing files with args {:?}", args),
		}
	}
}
