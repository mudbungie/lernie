//! The `lernie` harness binary.
//!
//! v0.3 surface:
//!
//! - `new <path>` — scaffold a conversation repo from the embedded
//!   template (ARCH §2.2).
//! - `prompt <repo> <message>` — drive one root conversation
//!   end-to-end: spawn a `<conv-id>` branch off `main` (no prefix —
//!   §2.3), commit the snapshot before the model call, land the
//!   response as a follow-up commit, dispatch the terminal compactor
//!   off the branch tip (§2.7), and `--no-ff` merge the result back
//!   to `main` (§2.6). Prints the conversation branch name.
//! - `dispatch compactor <repo> <branch>` — run the terminal
//!   compactor against an already-spawned conversation branch. Same
//!   shape `prompt` uses internally; exposed for external callers and
//!   future non-root-conversation dispatch cases (verifier, adversary,
//!   v0.4+).

use clap::{Parser, Subcommand};
use lernie::harness_root;
use lernie::prompt::{
    self, CompactorRequest, IdGen, NanoIdGen, SpawnAdapter, SpawnDispatcher, SystemClock,
};
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
    /// Create a new conversation repo. With no argument, scaffolds at
    /// `<harness-root>/conversations/<auto-id>/` (ARCH §2.2). With a
    /// path argument, scaffolds there literally.
    New {
        /// Destination path. Must not already exist as a non-empty
        /// directory. Optional — when omitted, an auto-generated id
        /// under the harness root is used.
        path: Option<PathBuf>,
    },
    /// Send one user message through the configured worker role on a
    /// fresh root-conversation branch. Prints the branch name on
    /// success.
    Prompt {
        /// Path to an existing conversation repo (created by `lernie new`).
        repo: PathBuf,
        /// The user message to send.
        message: String,
    },
    /// Dispatch a subagent. The v0.3 surface has one role
    /// (`compactor`); v0.4+ adds verifier, worker, adversary, etc.
    /// through the same primitive.
    Dispatch {
        #[command(subcommand)]
        role: DispatchRole,
    },
}

#[derive(Subcommand)]
enum DispatchRole {
    /// Run terminal compaction against a conversation branch: spawn a
    /// compactor branch off its tip (hyphenated descent of the
    /// parent's id, ARCH §2.2), write `summary/<seq>.md`, and
    /// `--no-ff` merge back into the conversation branch. Does not
    /// merge the conversation into `main` — that is the caller's
    /// responsibility (§2.6).
    Compactor {
        /// Path to the conversation repo.
        repo: PathBuf,
        /// Conversation branch to compact (the bare `<conv-id>`).
        branch: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::New { path } => {
            let dest = match path {
                Some(p) => p,
                None => match harness_root::resolve() {
                    Ok(root) => root.join("conversations").join(NanoIdGen.short()),
                    Err(e) => {
                        eprintln!("lernie new: {e}");
                        return ExitCode::FAILURE;
                    }
                },
            };
            match template::scaffold(&dest, &RealGit::new()) {
                Ok(()) => {
                    println!("{}", dest.display());
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("lernie new: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        Command::Prompt { repo, message } => {
            let dispatcher = match SpawnDispatcher::new() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("lernie prompt: cannot resolve current binary: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let harness = match harness_root::resolve() {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("lernie prompt: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let deps = prompt::Deps {
                adapter: &SpawnAdapter,
                git: &RealGit::new(),
                clock: &SystemClock,
                id_gen: &NanoIdGen,
                dispatcher: &dispatcher,
                harness_root: &harness,
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
/// re-entry; the invariant is that the interface is the CLI. The
/// branch name IS the conversation id (ARCH §2.3 — no prefix), so it
/// also names the worktree directory at `<repo>/<branch>/` (§2.2).
fn run_compactor_cli(repo: &PathBuf, branch: &str) -> Result<(), prompt::Error> {
    let worktree = repo.join(branch);
    let req = CompactorRequest {
        repo,
        parent_conv_id: branch,
        parent_worktree: &worktree,
    };
    prompt::compactor::run(&req, &RealGit::new(), &SystemClock, &NanoIdGen)
}
