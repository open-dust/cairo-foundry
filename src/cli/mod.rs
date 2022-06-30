use clap::Parser;

/**
 * Commands module
 *
 * This module contains all available commands of the CLI
 */
pub mod commands;

use commands::Commands;

/// Store the command line arguments
#[derive(Parser)]
#[clap(version, about)]
#[clap(propagate_version = true)]
pub struct Args {
	/// command to run
	#[clap(subcommand)]
	pub command: Commands,
}
