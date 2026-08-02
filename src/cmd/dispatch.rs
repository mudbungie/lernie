//! `lernie dispatch <role>` — subagent dispatch re-entry (ARCH §2.5,
//! §3.4). The §2.9 `become_pgid_leader` prelude is the binding's, run
//! before [`run`]. Per-role `--goal` rules and open-set role validity
//! live in [`crate::prompt::dispatch_cli::run`], not the clap surface.

use super::{Error, Fx, Outcome};
use crate::prompt::dispatch_cli;
use std::path::PathBuf;

/// `lernie dispatch <role> <repo> <branch> [--goal <text>] [--name <name>]`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Role to fork the child as (`souls/<role>.md` + a `roles:` entry).
    pub role: String,
    /// Path to the workspace (conversation repo) root.
    pub repo: PathBuf,
    /// Agent id of the dispatching parent (== branch name).
    pub branch: String,
    #[arg(long)]
    pub goal: Option<String>,
    /// Display name for the child (ARCH §2.3): one unbroken word, unique
    /// among the workspace's living agents, set here and never rewritten.
    /// `lernie message` accepts it in place of the child's agent id.
    #[arg(long)]
    pub name: Option<String>,
}

/// Fork the role's child through the front door — product-less on
/// success (§3.4). The failure prefix is `dispatch <role>`, as today.
/// The detached-launch target is [`Fx::driver_target`](super::Fx::driver_target).
pub fn run(args: Args, fx: &mut Fx) -> Result<Outcome, Error> {
    crate::name::require_agent_id(&args.branch)
        .map_err(|e| Error::new(format!("dispatch {}", args.role), e))?;
    dispatch_cli::run(
        &args.role,
        &args.repo,
        &args.branch,
        args.goal.as_deref(),
        args.name.as_deref(),
        &fx.driver_target,
    )
    .map_err(|e| Error::new(format!("dispatch {}", args.role), e))?;
    Ok(Outcome::Quiet)
}
