//! The `lernie` harness binary.
//!
//! Surface:
//!
//! - `new <path>` — scaffold a conversation repo from the embedded
//!   template (ARCH §2.2).
//! - `prompt <repo> <message>` — drive one root conversation
//!   end-to-end: spawn the `<conv-id>` branch, drive the model call via
//!   `bz` (§4.4), compact, and `--no-ff` merge back to `main` (§2.6).
//! - `dispatch <role> <repo> <branch> [--goal <text>]` — subagent
//!   dispatch re-entry (ARCH §3.4). `<role>` is positional; per-role
//!   `--goal` rules (required for `worker`, rejected for `compactor`)
//!   are validated in [`run_dispatch_cli`] rather than the clap surface.
//! - `stop <repo> <branch>` — cascading SIGTERM on the harness's
//!   pgid (ARCH §2.9); idempotent for already-stopped branches.
//! - `message <workspace> <agent> <content>` — deposit into an agent's
//!   inbox and probe the executor lock (§2.11, [`prompt::inbox::cli_run`]).
//! - `tool <name>` — in-process built-in tool entry (ARCH §3.3): the
//!   executor falls through to `<lernie> tool <name>` after external
//!   lookups miss. Reads `tool_use.input` JSON on stdin, bytes on stdout.

use clap::{Parser, Subcommand};
use lernie::harness_root;
use lernie::prompt::{
    self, CompactorRequest, IdGen, NanoIdGen, SpawnAdapter, SpawnDispatcher, SpawnTool,
    SystemClock, WorkerRequest, tool::builtin,
};
use lernie::template::{self, RealGit};
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "lernie", about = "Git-backed agent harness", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new conversation repo (ARCH §2.2). With no argument,
    /// scaffolds at `<data-root>/conversations/<auto-id>/`; with a path
    /// argument, scaffolds there literally.
    New {
        /// Destination path. Optional — an auto-generated id under the
        /// data root is used when omitted.
        path: Option<PathBuf>,
    },
    /// Send one user message through the worker role on a fresh
    /// root-conversation branch. Prints the branch name.
    Prompt {
        /// Path to an existing conversation repo (created by `lernie new`).
        repo: PathBuf,
        /// The user message to send.
        message: String,
    },
    /// Dispatch a subagent (ARCH §2.5, §3.4). `<role>` is `compactor`
    /// (terminal compaction, §2.7) or `worker` (per-call goal, §2.5);
    /// future roles slot in by name without a clap surface change.
    Dispatch {
        /// Role name. v0.4: `compactor` | `worker`.
        role: String,
        /// Path to the conversation repo.
        repo: PathBuf,
        /// Dispatching branch — the branch being compacted (`compactor`)
        /// or the parent off whose tip the worker spawns.
        branch: String,
        /// Per-call goal text. Required for `worker`; rejected for
        /// `compactor` (whose goal is built-in boilerplate, §2.7).
        #[arg(long)]
        goal: Option<String>,
    },
    /// Stop a conversation branch (ARCH §2.9 cascading SIGTERM).
    Stop { repo: PathBuf, branch: String },
    /// Deposit a message into an existing agent's inbox and probe the
    /// executor lock (ARCH §2.11, §3.4). Sender is harness-derived from
    /// `LERNIE_CONV_BRANCH`, never model-supplied.
    Message {
        /// Path to the workspace (conversation repo) root.
        workspace: PathBuf,
        /// Recipient agent id (== its branch name / hyphenated descent).
        agent: String,
        /// Message content — the body of the deposited file.
        content: String,
    },
    /// In-process built-in tool entry (ARCH §3.3). Reads
    /// `tool_use.input` JSON on stdin, writes bytes on stdout, exits
    /// 0/non-zero per the §3.3 contract. The executor resolves here as
    /// the third lookup hop (`<data-root>/tools/lernie-tool-<name>` →
    /// PATH → `<lernie> tool <name>`).
    Tool {
        /// Tool name as the model emitted it on the `tool_use` block
        /// (e.g. `read_file`).
        name: String,
    },
}

/// Role name for the v0.3 terminal compactor (§2.7); `--goal` rejected.
const ROLE_COMPACTOR: &str = "compactor";
/// Role name for the v0.4 worker subagent (§2.5); `--goal` required.
const ROLE_WORKER: &str = "worker";

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::New { path } => {
            // `roots` is always resolved: descriptions-always (§3.3)
            // snapshots the data-root pools into the new repo at creation.
            let roots = match harness_root::resolve() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("lernie new: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let dest =
                path.unwrap_or_else(|| roots.data.join("conversations").join(NanoIdGen.short()));
            match template::scaffold(&dest, &roots.data, &RealGit::new()) {
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
            prompt::stop::become_pgid_leader(); // §2.9 cascade leader
            let dispatcher = match SpawnDispatcher::new() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("lernie prompt: cannot resolve current binary: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let roots = match harness_root::resolve() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("lernie prompt: {e}");
                    return ExitCode::FAILURE;
                }
            };
            let tool_executor = SpawnTool::new(&roots.data, &SystemClock);
            let deps = prompt::Deps {
                adapter: &SpawnAdapter,
                sleeper: &prompt::RealSleeper,
                git: &RealGit::new(),
                clock: &SystemClock,
                id_gen: &NanoIdGen,
                dispatcher: &dispatcher,
                tool_executor: &tool_executor,
                config_root: &roots.config,
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
            role,
            repo,
            branch,
            goal,
        } => run_dispatch_cli(&role, &repo, &branch, goal.as_deref()),
        Command::Stop { repo, branch } => match prompt::stop::cli_run(&repo, &branch) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("lernie stop: {e}");
                ExitCode::FAILURE
            }
        },
        Command::Message {
            workspace,
            agent,
            content,
        } => match prompt::inbox::cli_run(&workspace, &agent, &content) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("lernie message: {e}");
                ExitCode::FAILURE
            }
        },
        Command::Tool { name } => {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let stderr = io::stderr();
            let mut stdin = stdin.lock();
            let mut stdout = stdout.lock();
            let mut stderr = stderr.lock();
            match builtin::run(&name, &mut stdin, &mut stdout, &mut stderr) {
                // Tool exit codes ride within `u8` (POSIX 0..=255 plus
                // 128+signo), so `as u8` is a faithful narrowing.
                Ok(code) => ExitCode::from(code as u8),
                Err(e) => {
                    eprintln!("lernie tool {name}: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

/// CLI handler for `lernie dispatch <role>` (ARCH §3.4). Per-role
/// `--goal` rules are enforced here and surfaced as a non-zero exit
/// with a `lernie dispatch <role>:` prefix.
fn run_dispatch_cli(role: &str, repo: &Path, branch: &str, goal: Option<&str>) -> ExitCode {
    let outcome = match role {
        ROLE_COMPACTOR => run_compactor_cli(repo, branch, goal),
        ROLE_WORKER => run_worker_cli(repo, branch, goal),
        other => Err(DispatchCliError::UnknownRole(other.to_owned())),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lernie dispatch {role}: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Errors specific to the dispatch CLI argument shape, joined with
/// [`prompt::Error`] under one `Display` for uniform `eprintln`.
#[derive(Debug)]
enum DispatchCliError {
    UnknownRole(String),
    GoalRequired(&'static str),
    GoalForbidden(&'static str),
    Inner(prompt::Error),
}

impl std::fmt::Display for DispatchCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRole(r) => write!(f, "unknown role {r:?}"),
            Self::GoalRequired(r) => write!(f, "--goal is required for role {r:?}"),
            Self::GoalForbidden(r) => write!(
                f,
                "--goal is not accepted for role {r:?} (built-in boilerplate)"
            ),
            Self::Inner(e) => write!(f, "{e}"),
        }
    }
}

impl From<prompt::Error> for DispatchCliError {
    fn from(value: prompt::Error) -> Self {
        Self::Inner(value)
    }
}

fn run_compactor_cli(
    repo: &Path,
    branch: &str,
    goal: Option<&str>,
) -> Result<(), DispatchCliError> {
    if goal.is_some() {
        return Err(DispatchCliError::GoalForbidden(ROLE_COMPACTOR));
    }
    let worktree = repo.join(branch);
    let req = CompactorRequest {
        repo,
        parent_conv_id: branch,
        parent_worktree: &worktree,
    };
    Ok(prompt::compactor::run(
        &req,
        &RealGit::new(),
        &SystemClock,
        &NanoIdGen,
    )?)
}

fn run_worker_cli(
    repo: &Path,
    parent_branch: &str,
    goal: Option<&str>,
) -> Result<(), DispatchCliError> {
    let goal = goal.ok_or(DispatchCliError::GoalRequired(ROLE_WORKER))?;
    let parent_worktree = repo.join(parent_branch);
    let req = WorkerRequest {
        repo,
        parent_branch,
        parent_worktree: &parent_worktree,
        goal,
    };
    // Print the spawned subagent branch so the `dispatch` built-in can
    // capture it as the `tool_result` handle (ARCH §3.3).
    let sub_branch = prompt::worker::run(&req, &RealGit::new(), &SystemClock, &NanoIdGen)?;
    println!("{sub_branch}");
    Ok(())
}
