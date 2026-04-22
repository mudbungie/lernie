//! The `lernie` harness binary.
//!
//! v0.1 surface is intentionally small: one subcommand, `new`, which
//! scaffolds a conversation repo from the embedded template (ARCH §2.2).
//! The `prompt` subcommand arrives in bl-e048.

use clap::{Parser, Subcommand};
use lernie::template::{self, RealGit};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "lernie", about = "Git-backed agent harness")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new conversation repo at <path>.
    New {
        /// Destination path. Must not already exist as a non-empty
        /// directory.
        path: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::New { path } => match template::scaffold(&path, &RealGit::new()) {
            Ok(()) => {
                println!("created {}", path.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("lernie new: {e}");
                ExitCode::FAILURE
            }
        },
    }
}
