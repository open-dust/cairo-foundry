use std::path::PathBuf;

use clap::Args;

#[derive(Args, Debug)]
pub struct List {
	/// Root path
	#[clap(short, long)]
	root: PathBuf,
}
