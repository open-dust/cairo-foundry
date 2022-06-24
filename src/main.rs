use cairo_foundry::cli::Args;
use clap::Parser;

fn main() {
	env_logger::init();
	println!("Hello, world!");

	let cli = Args::parse();
	cli.command.exec();
}
