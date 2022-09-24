use cairo_foundry::cli::{
	self,
	commands::CommandExecution,
	formatter::{self, Formatter},
};
use clap::Parser;
use confy;
use log::error;
use serde_derive::{Deserialize, Serialize};

const CONFIGURATION_FILENAME: &str = "config.toml";

#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
	args: String,
}

/// Tries to retrieve the Args from the CLI. If the user left the CLI empty, tries to
/// retrieve the args from the configuration file.
fn get_args() -> cli::Args {
	// Try to parse arguments from stdin
	let args_from_stdin: Result<cli::Args, clap::Error> = cli::Args::try_parse();

	match args_from_stdin {
		Err(stdin_err) => {
			// Rust trick to help with type system
			let stdin_err: clap::Error = stdin_err;

			match stdin_err.kind() {
				// This means the user left the CLI empty. We can try parsing the configuration file
				clap::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
					let cfg: Config = confy::load_path(CONFIGURATION_FILENAME).unwrap();
					// Need to prefix with `cairo-foundry ` to be able to parse it later on as a string
					let conf_args = format!("cairo-foundry {}", cfg.args);

					// Try to parse from the configuration file
					let args_from_config: Result<cli::Args, clap::Error> =
						cli::Args::try_parse_from(conf_args.split_ascii_whitespace());

					match args_from_config {
						Err(cfg_err) => {
							// Rust trick to help with type system
							let cfg_err: clap::Error = cfg_err;

							// Error is NOT that the configuration file was empty, so something is
							// indeed wrong in the configuration file.
							if cfg_err.kind()
								!= clap::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
							{
								eprintln!("\nERROR in your configuration file: \n");
							}

							// Print the error and exit
							cfg_err.exit()
						},
						Ok(args) => args,
					}
				},
				_ => {
					// Print the error and exit
					stdin_err.exit();
				},
			}
		},
		Ok(args) => args,
	}
}

fn main() {
	env_logger::init();

	let args = get_args();

	let formatter = formatter::make(&args);

	match args.command.exec() {
		Ok(output) => print!("{}", formatter.format(&output)),
		Err(error) => error!("{}", error),
	};
}
