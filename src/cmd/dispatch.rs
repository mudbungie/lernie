//! `lernie dispatch <role>` — subagent dispatch re-entry (ARCH §2.5,
//! §3.4). The §2.9 `become_pgid_leader` prelude is the binding's, run
//! before [`run`]. Per-role `--goal` rules and open-set role validity
//! live in [`crate::prompt::dispatch_cli::run`], not the clap surface.

use super::{Error, Fx, Outcome};
use crate::prompt::dispatch_cli;
use std::path::PathBuf;

/// `lernie dispatch <role> <repo> <branch> [--goal <text>] [--from <ref>]
/// [--name <name>] [--pin <dest>=<src>]...`.
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
    /// Fork the child off this ref instead of the parent's tip (ARCH
    /// §2.3, §7.2). The child stays `<parent>-<sub>`, so its return
    /// address is unchanged (§2.6); its governing config follows the ref
    /// (§2.2).
    #[arg(long)]
    pub from: Option<String>,
    /// Display name for the child (ARCH §2.3): one unbroken word, unique
    /// among the workspace's living agents, set here and never rewritten.
    /// `lernie message` accepts it in place of the child's agent id.
    #[arg(long)]
    pub name: Option<String>,
    /// Pin a caller-supplied document (ARCH §2.5): freeze `<src>`'s
    /// exact bytes at worktree-relative `<dest>` on the child's dispatch
    /// commit, beside `goal.md` and `soul.md`. Repeatable; validated —
    /// and refused — before any branch or ref exists
    /// ([`crate::prompt::pinned_doc`]). Exact parity with
    /// `lernie prompt --pin`.
    #[arg(long = "pin", value_name = "DEST=SRC")]
    pub pin: Vec<String>,
}

/// Fork the role's child through the front door — product-less on
/// success (§3.4). The failure prefix is `dispatch <role>`, as today.
/// The detached-launch target is [`Fx::driver_target`](super::Fx::driver_target).
pub fn run(args: Args, fx: &mut Fx) -> Result<Outcome, Error> {
    crate::name::require_agent_id(&args.branch)
        .map_err(|e| Error::new(format!("dispatch {}", args.role), e))?;
    // Pins load first — parity with `prompt` (ARCH §2.5): every refusal
    // precedes the fork, so no branch, ref or inbox exists when one fires.
    let pins = crate::prompt::pinned_doc::load(&args.pin)
        .map_err(|e| Error::new(format!("dispatch {}", args.role), e))?;
    dispatch_cli::run(
        &args.role,
        &args.repo,
        &args.branch,
        args.goal.as_deref(),
        args.from.as_deref(),
        args.name.as_deref(),
        &pins,
        &fx.driver_target,
    )
    .map_err(|e| Error::new(format!("dispatch {}", args.role), e))?;
    Ok(Outcome::Quiet)
}
