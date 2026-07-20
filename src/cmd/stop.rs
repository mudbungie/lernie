//! `lernie stop` — SIGTERM a conversation branch's executor pgid (ARCH
//! §2.9). Idempotent for already-stopped branches; `--stop-children`
//! walks the id namespace to reach descendants.

use super::{Error, Fx, Outcome};
use crate::prompt::stop;
use std::path::PathBuf;

/// `lernie stop <repo> <branch> [--stop-children]`.
#[derive(clap::Args, Debug)]
pub struct Args {
    pub repo: PathBuf,
    pub branch: String,
    /// Also stop the agent's whole subagent subtree (§2.9).
    #[arg(long)]
    pub stop_children: bool,
}

/// Signal the pgid(s) — product-less on success (§3.4).
pub fn run(args: Args, _fx: &mut Fx) -> Result<Outcome, Error> {
    stop::cli_run(&args.repo, &args.branch, args.stop_children)
        .map_err(|e| Error::new("stop", e))?;
    Ok(Outcome::Quiet)
}
