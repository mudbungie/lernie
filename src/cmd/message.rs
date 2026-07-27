//! `lernie message` — deposit into an agent's inbox and probe the
//! executor lock (ARCH §2.11, §3.4). The sender is resolved from
//! `LERNIE_CONV_BRANCH` inside [`crate::prompt::inbox::cli_run`].
//!
//! **The failed-branch advisory (§2.10).** Messaging a branch whose
//! latest model call failed — retries exhausted, or a non-retryable
//! error — stays legal, and *is* the recovery path (the §2.9-shape
//! resume: deposit, driver, re-run against unchanged read state). But
//! the sender must not mistake such a branch for one that is merely
//! idle: the verb warns on stderr when the recipient was quiescent and
//! its latest step's `response.json` last segment terminated in an
//! `Error` (the same framing-only derivation the §8 silent-death sweep
//! reads, [`latest_step_outcome`]). The deposit and the exit code are
//! untouched — the advisory informs, it never declines.

use super::{Error, Fx, Outcome};
use crate::prompt::inbox::{self, ProbeOutcome};
use crate::prompt::step::latest_step_outcome;
use crate::provider::segment;
use std::path::{Path, PathBuf};

/// `lernie message <workspace> <agent> <content>`.
#[derive(clap::Args, Debug)]
pub struct Args {
    pub workspace: PathBuf,
    pub agent: String,
    pub content: String,
}

/// Deposit then probe-and-launch — product-less on success (§3.4). The
/// detached-launch target is [`Fx::driver_target`](super::Fx::driver_target).
/// The failed-branch state is read *before* the deposit (the launched
/// driver may already be re-running the step after it), and the advisory
/// prints only when the probe found the branch quiescent — a held lock
/// means a live executor is still working it.
pub fn run(args: Args, fx: &mut Fx) -> Result<Outcome, Error> {
    crate::name::require_agent_id(&args.agent).map_err(|e| Error::new("message", e))?;
    let failed = branch_failed(&args.workspace, &args.agent);
    let probe = inbox::cli_run(
        &args.workspace,
        &args.agent,
        &args.content,
        &fx.driver_target,
    )
    .map_err(|e| Error::new("message", e))?;
    if failed && probe == ProbeOutcome::Launched {
        eprintln!("{}", failed_branch_note(&args.agent));
    }
    Ok(Outcome::Quiet)
}

/// Did the recipient's latest step's model call fail (§2.10)? The §8
/// sweep's own framing-only read ([`latest_step_outcome`]); no step tree
/// or no readable response is `false` — nothing to advise on.
pub(super) fn branch_failed(workspace: &Path, agent: &str) -> bool {
    latest_step_outcome(workspace, agent) == Some(segment::Outcome::Failed)
}

/// The stderr advisory for a deposit into a quiescent branch whose
/// latest model call failed (§2.10): queued and driven, never declined —
/// but named, so a branch that went quiet is discoverable from the verb
/// that touches it, not only from `lernie scan`.
pub(super) fn failed_branch_note(agent: &str) -> String {
    format!(
        "lernie: {agent}: latest model call failed (§2.10) — message queued and a driver \
         launched, but if the cause persists the branch will not advance; \
         see steps/{agent}/ or run `lernie scan`"
    )
}
