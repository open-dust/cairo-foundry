use cairo_foundry::cli::Args;
use clap::Parser;

fn main() {
	env_logger::init();

	let cli = Args::parse();
	cli.command.exec();
}
