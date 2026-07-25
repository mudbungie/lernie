//! `lernie message` — deposit into an agent's inbox and probe the
//! executor lock (ARCH §2.11, §3.4). The sender is resolved from
//! `LERNIE_CONV_BRANCH` inside [`crate::prompt::inbox::cli_run`].

use super::{Error, Fx, Outcome};
use crate::prompt::inbox;
use std::path::PathBuf;

/// `lernie message <workspace> <agent> <content>`.
#[derive(clap::Args, Debug)]
pub struct Args {
    pub workspace: PathBuf,
    pub agent: String,
    pub content: String,
}

/// Deposit then probe-and-launch — product-less on success (§3.4). The
/// detached-launch target is [`Fx::driver_target`](super::Fx::driver_target).
pub fn run(args: Args, fx: &mut Fx) -> Result<Outcome, Error> {
    crate::name::require_agent_id(&args.agent).map_err(|e| Error::new("message", e))?;
    inbox::cli_run(
        &args.workspace,
        &args.agent,
        &args.content,
        &fx.driver_target,
    )
    .map_err(|e| Error::new("message", e))?;
    Ok(Outcome::Quiet)
}
