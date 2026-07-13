//! The `lernie` harness binary.
//!
//! Surface:
//!
//! - `new <path>` — scaffold a conversation repo (ARCH §2.2).
//! - `prompt <repo> <message>` — drive one root conversation: spawn the
//!   `<conv-id>` branch, model-call via `bz` (§4.4), compact (§2.6).
//! - `dispatch <role> <repo> <branch> [--goal <text>]` — subagent dispatch
//!   re-entry (ARCH §3.4); per-role `--goal` rules validated in
//!   [`run_dispatch_cli`], not the clap surface.
//! - `stop <repo> <branch> [--stop-children]` — SIGTERM the executor's
//!   pgid (ARCH §2.9); idempotent for already-stopped branches.
//!   `--stop-children` also walks the id namespace to stop descendants.
//! - `message <workspace> <agent> <content>` — deposit into an agent's
//!   inbox and probe the executor lock (§2.11, [`prompt::inbox::cli_run`]).
//! - `scan <workspace>` — the operator sweep-and-flush (§2.11, §8).
//!   Hand/cron only; never wired into any driver hot path.
//! - `advance <workspace> <agent>` — one hop of the §6 workflow chain:
//!   take the lease, deliver, step, exec the successor. The detached
//!   launch target of every §2.11 launch seam.
//! - `tool <name>` — in-process built-in tool entry (ARCH §3.3):
//!   `tool_use.input` JSON on stdin, bytes on stdout.

use clap::{Parser, Subcommand};
use lernie::harness_root;
use lernie::prompt::{
    self, IdGen, NanoIdGen, SpawnAdapter, SpawnDispatcher, SpawnTool, SystemClock, tool::builtin,
};
use lernie::template::{self, RealGit};
use std::{io, path::Path, path::PathBuf, process::ExitCode};

#[derive(Parser)]
#[command(name = "lernie", about = "Git-backed agent harness", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new conversation repo (ARCH §2.2). No argument scaffolds
    /// at `<data-root>/conversations/<auto-id>/`; a path scaffolds there.
    New {
        /// Destination path. Optional — an auto-id under the data root
        /// is used when omitted.
        path: Option<PathBuf>,
    },
    /// Send one user message on a fresh root branch; prints its name.
    Prompt {
        /// Path to an existing conversation repo (created by `lernie new`).
        repo: PathBuf,
        /// The user message to send.
        message: String,
    },
    /// Dispatch a subagent (ARCH §2.5, §3.4). `<role>` is `compactor`
    /// (§2.7) or `worker` (§2.5); future roles slot in by name.
    Dispatch {
        /// Role name. v0.4: `compactor` | `worker`.
        role: String,
        /// Path to the conversation repo.
        repo: PathBuf,
        /// Dispatching branch — compacted (`compactor`) or the parent off
        /// whose tip the worker spawns.
        branch: String,
        /// Per-call goal text. Required for `worker`; rejected for
        /// `compactor` (built-in boilerplate, §2.7).
        #[arg(long)]
        goal: Option<String>,
    },
    /// Stop a conversation branch (ARCH §2.9 SIGTERM). Default stops the
    /// one agent; `--stop-children` also stops every descendant
    /// (`<branch>-*`, §2.3) — the opt-in agent→agent cascade.
    Stop {
        repo: PathBuf,
        branch: String,
        /// Also stop the agent's whole subagent subtree (§2.9).
        #[arg(long)]
        stop_children: bool,
    },
    /// Deposit a message into an agent's inbox and probe the executor
    /// lock (ARCH §2.11, §3.4). Sender from `LERNIE_CONV_BRANCH`.
    Message {
        /// Path to the workspace (conversation repo) root.
        workspace: PathBuf,
        /// Recipient agent id (== branch name / hyphenated descent).
        agent: String,
        /// Message content — the body of the deposited file.
        content: String,
    },
    /// Operator verb: one workspace-wide silent-death sweep + inbox flush
    /// (ARCH §2.11, §8). Hand/cron only; never on a driver hot path.
    Scan {
        /// Path to the workspace (conversation repo) root.
        workspace: PathBuf,
    },
    /// Drive one agent's branch forward (ARCH §6): take the lease (adopt
    /// LERNIE_LOCK_FD or acquire), deliver pending mail, run the next
    /// step, and exec the successor hop. The target every launch seam
    /// spawns; also an operator verb.
    Advance {
        /// Path to the workspace (conversation repo) root.
        workspace: PathBuf,
        /// Agent id (== branch name / hyphenated descent) to drive.
        agent: String,
    },
    /// In-process built-in tool entry (ARCH §3.3): `tool_use.input` JSON
    /// on stdin, bytes on stdout, exit 0/non-zero. Third resolver hop
    /// (`<data-root>/tools/lernie-tool-<name>` → PATH → `<lernie> tool …`).
    Tool {
        /// Tool name as the model emitted it (e.g. `read_file`).
        name: String,
    },
}

/// Uniform failure exit: `<prefix>: <error>` on stderr, non-zero.
fn fail(prefix: &str, e: impl std::fmt::Display) -> ExitCode {
    eprintln!("{prefix}: {e}");
    ExitCode::FAILURE
}

/// Uniform success/failure exit for product-less verbs.
fn ok_or_fail(prefix: &str, r: Result<(), impl std::fmt::Display>) -> ExitCode {
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => fail(prefix, e),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::New { path } => {
            // descriptions-always (§3.3) snapshots the data-root pools
            // into the new repo at creation, so `roots` is always resolved.
            let roots = match harness_root::resolve() {
                Ok(r) => r,
                Err(e) => return fail("lernie new", e),
            };
            let dest =
                path.unwrap_or_else(|| roots.data.join("conversations").join(NanoIdGen.short()));
            match template::scaffold(&dest, &roots.data, &RealGit::new()) {
                Ok(()) => {
                    println!("{}", dest.display());
                    ExitCode::SUCCESS
                }
                Err(e) => fail("lernie new", e),
            }
        }
        Command::Prompt { repo, message } => {
            // No workspace scan: drivers touch only their own branch (§2.11).
            prompt::stop::become_pgid_leader(); // §2.9 cascade leader
            prompt::install_stop_handler(); // §2.9 step-3 stopped deposit
            let dispatcher = match SpawnDispatcher::new() {
                Ok(d) => d,
                Err(e) => return fail("lernie prompt: cannot resolve current binary", e),
            };
            let roots = match harness_root::resolve() {
                Ok(r) => r,
                Err(e) => return fail("lernie prompt", e),
            };
            let tool_executor = SpawnTool::new(&roots.data, &SystemClock);
            // §2.11 exit launch: the detached `lernie advance` spawn (§6).
            let launcher = match prompt::inbox::AdvanceLauncher::current() {
                Ok(l) => l,
                Err(e) => return fail("lernie prompt: cannot resolve current binary", e),
            };
            let deps = prompt::Deps {
                adapter: &SpawnAdapter,
                sleeper: &prompt::RealSleeper,
                git: &RealGit::new(),
                clock: &SystemClock,
                id_gen: &NanoIdGen,
                dispatcher: &dispatcher,
                tool_executor: &tool_executor,
                config_root: &roots.config,
                stop: prompt::stop_flag(),
                launcher: &launcher,
            };
            match prompt::run(&repo, &message, &deps) {
                Ok(branch) => {
                    println!("{branch}");
                    ExitCode::SUCCESS
                }
                Err(e) => fail("lernie prompt", e),
            }
        }
        Command::Dispatch {
            role,
            repo,
            branch,
            goal,
        } => {
            prompt::stop::become_pgid_leader(); // §2.9: child executor takes its own pgid
            // No workspace scan here — dispatch re-entry is a hot path (§2.11).
            ok_or_fail(
                &format!("lernie dispatch {role}"),
                prompt::dispatch_cli::run(&role, &repo, &branch, goal.as_deref()),
            )
        }
        Command::Advance { workspace, agent } => run_advance_cli(&workspace, &agent),
        Command::Stop {
            repo,
            branch,
            stop_children,
        } => ok_or_fail(
            "lernie stop",
            prompt::stop::cli_run(&repo, &branch, stop_children),
        ),
        Command::Message {
            workspace,
            agent,
            content,
        } => ok_or_fail(
            "lernie message",
            prompt::inbox::cli_run(&workspace, &agent, &content),
        ),
        Command::Scan { workspace } => match prompt::inbox::scan::cli_run(&workspace) {
            Ok(report) => {
                println!("{report}");
                ExitCode::SUCCESS
            }
            Err(e) => fail("lernie scan", e),
        },
        Command::Tool { name } => {
            let mut stdin = io::stdin().lock();
            let mut stdout = io::stdout().lock();
            let mut stderr = io::stderr().lock();
            match builtin::run(&name, &mut stdin, &mut stdout, &mut stderr) {
                // Tool exit codes ride within `u8` (POSIX), so `as u8` is faithful.
                Ok(code) => ExitCode::from(code as u8),
                Err(e) => fail(&format!("lernie tool {name}"), e),
            }
        }
    }
}

/// CLI handler for `lernie advance <workspace> <agent>` (ARCH §6): one
/// hop of the workflow chain. The library does everything up to the
/// exec ([`prompt::dispatch::advance::cli::cli_run`]); this shim only
/// performs the `exec` itself — a successful `execve` replaces this
/// image (the §6 exec baton, lock fd riding it), so the call returning
/// at all is the failure path.
fn run_advance_cli(workspace: &Path, agent: &str) -> ExitCode {
    prompt::stop::become_pgid_leader(); // §2.9: every driver takes its own pgid
    prompt::install_stop_handler(); // §2.9 step-3 stopped deposit
    match prompt::dispatch::advance::cli::cli_run(workspace, agent) {
        Ok(prompt::dispatch::advance::cli::AdvanceHandoff::Exec(mut cmd)) => {
            use std::os::unix::process::CommandExt;
            fail("lernie advance: exec successor", cmd.exec())
        }
        Ok(prompt::dispatch::advance::cli::AdvanceHandoff::Done(_)) => ExitCode::SUCCESS,
        Err(e) => fail("lernie advance", e),
    }
}
