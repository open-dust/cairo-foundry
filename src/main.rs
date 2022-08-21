use cairo_foundry::cli::{
	self,
	commands::CommandExecution,
	formatter::{self, Formatter},
};
use clap::Parser;
use log::error;

fn main() {
	env_logger::init();

	let cli = cli::Args::parse();

	let formatter = formatter::make(&cli);

	match cli.command.exec() {
		Ok(output) => print!("{}", formatter.format(&output)),
		Err(error) => error!("{}", error),
	};
}
