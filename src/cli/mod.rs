use clap::Parser;

/**
 * Commands module
 *
 * This module contains all available commands of the CLI
 */
pub mod commands;

/**
 * Formatter module
 *
 * This module contains all formatters that can be used to print commands output
 */
pub mod formatter;

use commands::Commands;

/// Store the command line arguments
#[derive(Parser)]
#[clap(version, about)]
#[clap(propagate_version = true)]
pub struct Args {
	/// command to run
	#[clap(subcommand)]
	pub command: Commands,

	/// Format the command output in JSON
	#[clap(long)]
	pub json: bool,
}
