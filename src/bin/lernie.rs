//! The `lernie` harness binary.
//!
//! v0.2 surface:
//!
//! - `new <path>` — scaffold a conversation repo from the embedded
//!   template (ARCH §2.2).
//! - `prompt <repo> <message>` — drive one exchange: spawn an
//!   `ex/<ts>-<id>` branch off `main` with a worktree under
//!   `<repo>/.lernie/worktrees/ex/…`, commit a step snapshot before the
//!   model call, invoke the provider adapter, land the response as a
//!   follow-up commit, and print the branch name. Merge-back (§2.6)
//!   is a separate v0.2 task; `main`'s HEAD is untouched.

use clap::{Parser, Subcommand};
use lernie::prompt::{self, NanoIdGen, SpawnAdapter, SystemClock};
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
    /// Send one user message through the configured worker role on a
    /// fresh exchange branch. Prints the branch name on success.
    Prompt {
        /// Path to an existing conversation repo (created by `lernie new`).
        repo: PathBuf,
        /// The user message to send.
        message: String,
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
        Command::Prompt { repo, message } => {
            let deps = prompt::Deps {
                adapter: &SpawnAdapter,
                git: &RealGit::new(),
                clock: &SystemClock,
                id_gen: &NanoIdGen,
            };
            match prompt::run(&repo, &message, &deps) {
                Ok(branch) => {
                    println!("{branch}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("lernie prompt: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}
