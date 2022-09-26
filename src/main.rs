use cairo_foundry::cli::{
	commands::CommandExecution,
	formatter::{self, Formatter},
};
use cairo_foundry::get_args;
use log::error;

fn main() {
	env_logger::init();

	let args = get_args::get_args();

	let formatter = formatter::make(&args);

	match args.command.exec() {
		Ok(output) => print!("{}", formatter.format(&output)),
		Err(error) => error!("{}", error),
	};
}
