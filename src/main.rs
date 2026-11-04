use clap::Parser;
use std::path::PathBuf;

use wipe::{PatternSpec, wipe_paths};

#[derive(Debug, Parser)]
#[command(version, about = "Overwrite and remove regular files on Linux")]
struct Arguments {
	/// Recursively process regular files in directories.
	#[arg(short, long)]
	recursive: bool,

	/// Comma-separated overwrite patterns: 0, 1, r (random), or a hexadecimal pattern (e.g. 0xDEADBEEF).
	#[arg(short, long, value_delimiter = ',', default_value = "0")]
	passes: Vec<PatternSpec>,

	/// Files and directories to wipe.
	#[arg(required = true)]
	paths: Vec<PathBuf>,
}

fn main() {
	let arguments = Arguments::parse();
	if let Err(error) = wipe_paths(&arguments.paths, arguments.recursive, &arguments.passes) {
		eprintln!("wipe: {error}");
		std::process::exit(1);
	}
}
