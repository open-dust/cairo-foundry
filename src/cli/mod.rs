use clap::Parser;

pub mod commands;

use commands::Commands;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Args {
	/// command to run
	#[clap(subcommand)]
	pub command: Commands,
}
