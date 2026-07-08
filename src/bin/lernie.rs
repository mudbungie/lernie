//! The `lernie` harness binary.
//!
//! Surface:
//!
//! - `new <path>` — scaffold a conversation repo from the embedded
//!   template (ARCH §2.2).
//! - `prompt <repo> <message>` — drive one root conversation
//!   end-to-end: spawn a `<conv-id>` branch off `main` (no prefix —
//!   §2.3), commit the snapshot, drive the model call via `bz` (§4.4),
//!   dispatch the terminal compactor off the branch tip (§2.7), and
//!   `--no-ff` merge the result back to `main` (§2.6). Prints the
//!   conversation branch name.
//! - `dispatch <role> <repo> <branch> [--goal <text>]` — re-entry
//!   point for subagent dispatch (ARCH §3.4). `<role>` is positional
//!   so the surface generalizes across the v0.3 compactor, the v0.4
//!   worker, and future verifier/critic/etc. (§2.5). `--goal` is
//!   required for `worker` and forbidden for `compactor` (whose goal
//!   is built-in boilerplate, §2.7); per-role validation lives here
//!   rather than in the clap surface so adding a role is a one-line
//!   match arm rather than a Subcommand-tree edit.
//! - `stop <repo> <branch>` — cascading SIGTERM on the harness's
//!   pgid (ARCH §2.9); idempotent for already-stopped branches.
//! - `tool <name>` — in-process built-in tool entry (ARCH §3.3): the
//!   tool executor falls through to `<lernie> tool <name>` after
//!   external lookups miss. Reads `tool_use.input` JSON from stdin,
//!   writes bytes to stdout, exits 0/non-zero per the §3.3 contract.

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
    /// Dispatch a subagent (ARCH §2.5, §3.4). `<role>` selects which
    /// subagent shape — `compactor` (terminal compaction, §2.7) or
    /// `worker` (per-call goal, §2.5). Future roles slot in by name
    /// without a clap surface change.
    Dispatch {
        /// Role name. v0.4: `compactor` | `worker`.
        role: String,
        /// Path to the conversation repo.
        repo: PathBuf,
        /// Dispatching branch. For `compactor`, the conversation
        /// branch being compacted; for `worker`, the parent branch
        /// off whose tip the worker is spawned.
        branch: String,
        /// Per-call goal text. Required for `worker`; rejected for
        /// `compactor` (whose goal is built-in boilerplate, §2.7).
        #[arg(long)]
        goal: Option<String>,
    },
    /// Stop a conversation branch (ARCH §2.9 cascading SIGTERM).
    Stop { repo: PathBuf, branch: String },
    /// In-process built-in tool entry (ARCH §3.3). Reads
    /// `tool_use.input` JSON from stdin, writes raw result bytes to
    /// stdout, and exits 0 on success or non-zero on failure (the
    /// stderr message is concatenated into `tool_result.content` by
    /// the executor when `is_error` is set). The tool executor
    /// resolves to this subcommand as the third hop of §3.3 lookup
    /// (`<harness-root>/tools/lernie-tool-<name>` → PATH →
    /// `<lernie> tool <name>`).
    Tool {
        /// Tool name as the model emitted it on the `tool_use` block
        /// (e.g. `read_file`).
        name: String,
    },
}

/// Role name for the v0.3 terminal compactor (ARCH §2.7). Built-in
/// boilerplate goal — `--goal` on the CLI is rejected for this role.
const ROLE_COMPACTOR: &str = "compactor";
/// Role name for the v0.4 worker subagent (ARCH §2.5). Per-call goal,
/// supplied via `--goal <text>`.
const ROLE_WORKER: &str = "worker";

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
            prompt::stop::become_pgid_leader(); // §2.9 cascade leader
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
            let tool_executor = SpawnTool::new(&harness, &SystemClock);
            let deps = prompt::Deps {
                adapter: &SpawnAdapter,
                sleeper: &prompt::RealSleeper,
                git: &RealGit::new(),
                clock: &SystemClock,
                id_gen: &NanoIdGen,
                dispatcher: &dispatcher,
                tool_executor: &tool_executor,
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
        Command::Tool { name } => {
            let stdin = io::stdin();
            let stdout = io::stdout();
            let stderr = io::stderr();
            let mut stdin = stdin.lock();
            let mut stdout = stdout.lock();
            let mut stderr = stderr.lock();
            match builtin::run(&name, &mut stdin, &mut stdout, &mut stderr) {
                // Tool exit codes ride within `u8` (POSIX 0..=255 plus
                // 128+signo); `as u8` is a faithful narrowing because
                // every legal value already fits.
                Ok(code) => ExitCode::from(code as u8),
                Err(e) => {
                    eprintln!("lernie tool {name}: {e}");
                    ExitCode::FAILURE
                }
            }
        }
    }
}

/// CLI handler for `lernie dispatch <role>`. The branch name IS the
/// conversation id (ARCH §2.3 — no prefix), so it also names the
/// worktree directory at `<repo>/<branch>/` (§2.2). Per-role argument
/// rules (`--goal` required for worker, rejected for compactor) are
/// enforced here and surfaced as a non-zero exit with a `lernie
/// dispatch <role>:` prefix matching the existing error style.
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

/// Errors specific to the dispatch CLI argument shape. Joined with
/// [`prompt::Error`] (the in-process role implementations' error
/// type) under one `Display` so the eprintln formatting stays
/// uniform across cases.
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
    // Print the spawned subagent branch so callers — notably the
    // `dispatch` built-in (ARCH §3.3) — can capture it as the
    // `tool_result` handle without listing branches to find it.
    let sub_branch = prompt::worker::run(&req, &RealGit::new(), &SystemClock, &NanoIdGen)?;
    println!("{sub_branch}");
    Ok(())
}
