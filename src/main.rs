use cairo_foundry::cli::{self, formatter::Formatter};
use clap::Parser;
use log::error;

fn main() {
	env_logger::init();

	let cli = cli::Args::parse();

	let formatter = Box::new(cli::formatter::text::TextFormatter {});

	match cli.command.exec() {
		Ok(output) => println!("{}", formatter.format(&output)),
		Err(error) => error!("{}", error),
	};
}
