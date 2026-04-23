//! The `lernie` harness binary.
//!
//! v0.2 surface:
//!
//! - `new <path>` — scaffold a conversation repo from the embedded
//!   template (ARCH §2.2).
//! - `prompt <repo> <message>` — drive one exchange end-to-end: spawn
//!   an `ex/<ts>-<id>` branch off `main`, commit the snapshot before
//!   the model call, land the response as a follow-up commit, dispatch
//!   the terminal compactor off the branch tip (§2.7), and `--no-ff`
//!   merge the result back to `main` (§2.6). Prints the exchange
//!   branch name.
//! - `dispatch compactor <repo> <branch>` — run the terminal
//!   compactor against an already-spawned exchange branch. Same shape
//!   `prompt` uses internally; exposed for external callers and future
//!   non-exchange dispatch cases (verifier, adversary, v0.4+).

use clap::{Parser, Subcommand};
use lernie::prompt::{self, CompactorRequest, NanoIdGen, SpawnAdapter, SystemClock};
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
    /// Dispatch a subagent. The v0.2 surface has one role
    /// (`compactor`); v0.4+ adds verifier, worker, adversary, etc.
    /// through the same primitive.
    Dispatch {
        #[command(subcommand)]
        role: DispatchRole,
    },
}

#[derive(Subcommand)]
enum DispatchRole {
    /// Run terminal compaction against an exchange branch: spawn an
    /// invocation branch off its tip, write `.agent/compactions/<seq>.md`,
    /// and `--no-ff` merge back into the exchange branch. Does not
    /// merge the exchange into `main` — that is the caller's
    /// responsibility (§2.6).
    Compactor {
        /// Path to the conversation repo.
        repo: PathBuf,
        /// Exchange branch to compact (`ex/<ts>-<short-id>`).
        branch: String,
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
        Command::Dispatch {
            role: DispatchRole::Compactor { repo, branch },
        } => match run_compactor_cli(&repo, &branch) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("lernie dispatch compactor: {e}");
                ExitCode::FAILURE
            }
        },
    }
}

/// CLI handler for `lernie dispatch compactor`. Builds a
/// [`CompactorRequest`] from the repo + branch and runs the stub
/// through the in-process entry point. §3.4 permits in-process
/// re-entry; the invariant is that the interface is the CLI.
fn run_compactor_cli(repo: &PathBuf, branch: &str) -> Result<(), prompt::Error> {
    let Some(exchange_id) = branch.strip_prefix("ex/") else {
        // Surface as an adapter-agnostic git-shaped error rather than
        // coining a new variant — the failure mode is "wrong branch
        // name", which the operator sees with the offending name in
        // context via the stderr print in `main`.
        return Err(prompt::Error::Git {
            op: "dispatch compactor",
            source: std::io::Error::other(format!(
                "expected ex/<ts>-<short-id> branch name, got {branch:?}"
            )),
        });
    };
    let worktree = repo.join(".lernie/worktrees/ex").join(exchange_id);
    let req = CompactorRequest {
        repo,
        parent_branch: branch,
        parent_worktree: &worktree,
        exchange_id,
    };
    prompt::compactor::run(&req, &RealGit::new(), &SystemClock, &NanoIdGen)
}
