//! What the step loop does on its way out (ARCH §2.9, §2.11, §6).
//!
//! Three terminal shapes reach [`finish`], keyed by epitaph value (§2.6
//! — code branches on the value, never on shape): a `stopped` branch
//! (§2.9), an exhausted branch (§6), and the ordinary final-response
//! branch. Only a `stopped` branch has an outstanding deposit to make
//! here; the final response deposited in the loop and the exhausted
//! branch at the boundary check.
//!
//! **There is no terminal compaction** (§2.7: "There is no terminal
//! compaction stage anymore"). The v0.3 compactor dispatch that fired at
//! every final response is deleted: a child's result message carries its
//! own terminal response (§2.6), not a compactor product, and with
//! merge-back gone there is no merge payload to slim before returning.
//! Compaction now runs only at configured checkpoints during a branch's
//! life ([`crate::prompt::compactor::checkpoint`]).
//!
//! The stopped deposit is the §2.9 step-3 return performed *outside* the
//! signal handler ([`super::stop_signal`]) — the executor's SIGTERM
//! handler set a flag, the loop broke at a check point, and this is the
//! final deposit before the process exits. It reads the branch tip as the
//! terminal ref and deposits a `stopped`-epitaph result with no body (a
//! stopped agent has almost never finished speaking; [`deposit_result`]
//! renders a body-absent message either way). A root has no parent inbox,
//! so the deposit is a structural no-op there
//! ([`crate::prompt::inbox::deposit_child_result`]). The stop *signature*
//! — the missing trailing `end` on the branch's own `response.json` —
//! lives on a different tree and is untouched by this deposit.
//!
//! [`exit_launch`] is the closing act of the §2.11 exit protocol: after
//! [`super::run_exchange`] releases the executor lock, a driver is
//! spawned at the exiting agent itself, fire-and-forget. The launch is
//! decided by epitaph value (§2.11 pin 2): a final response launches;
//! `stopped` never does (a relaunch would resurrect the branch the
//! operator just killed); `budget-exhausted` never does (an epitaph-spam
//! cycle against a hard ceiling).
//!
//! [`deposit_result`]: crate::prompt::inbox::deposit_result

use super::super::budget;
use super::super::inbox::Epitaph;
use super::super::{Deps, Error};
use super::result_deposit::deposit_terminal;
use crate::config::Budgets;
use std::path::Path;

/// The §6 budget check at a model-call boundary: tokens/wall/depth derived
/// live over the tree (no stored counter, PRINCIPLES SSOT). On exhaustion
/// it writes `refs/lernie/budget-exhausted/<branch>`, deposits a
/// `budget-exhausted` result (the agent did not speak this step, so no
/// body), and returns `true` so the loop ceases — an ordinary terminal
/// state (§2.9). `false` continues the loop.
pub(super) fn budget_exhausted(
    repo: &Path,
    conv_id: &str,
    branch: &str,
    worktree: &Path,
    budgets: &Budgets,
    deps: &Deps<'_>,
) -> Result<bool, Error> {
    let Some(ex) = budget::check(repo, branch, budgets) else {
        return Ok(false);
    };
    eprintln!("lernie: budget {ex} on {branch}; stopping (§6)");
    budget::mark_exhausted(worktree, branch, deps.git).map_err(|source| Error::Git {
        op: "budget-exhausted update-ref",
        source,
    })?;
    deposit_terminal(
        repo,
        conv_id,
        worktree,
        Epitaph::BudgetExhausted,
        None,
        deps,
    )?;
    Ok(true)
}

/// Finish the exchange by epitaph value (§2.6). Only `stopped` has an
/// outstanding deposit here — its result is deposited on the way out
/// (§2.9 step 3). A final response deposited inside the loop and a
/// `budget-exhausted` branch at the boundary check, so both are no-ops.
/// No terminal compaction is dispatched (§2.7 — the stage is deleted).
pub(super) fn finish(
    repo: &Path,
    conv_id: &str,
    worktree: &Path,
    epitaph: Epitaph,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    if epitaph == Epitaph::Stopped {
        deposit_terminal(repo, conv_id, worktree, Epitaph::Stopped, None, deps)
    } else {
        Ok(())
    }
}

/// The self-directed launch closing the §2.11 exit protocol, called
/// *after* the executor lock is released: spawn a driver at this agent,
/// fire-and-forget, by epitaph value (§2.11 pin 2). Fire-and-forget is
/// literal — a launch failure is logged and swallowed, never propagated:
/// it falls into the accepted crash class (§2.11), where the stranding
/// is late, not lost, and the next touch (a reprompt, or a hand-run
/// `lernie scan`) heals it.
pub(super) fn exit_launch(workspace: &Path, agent_id: &str, epitaph: Epitaph, deps: &Deps<'_>) {
    // §2.11 pin 2: only a final response launches — stopped and
    // budget-exhausted never relaunch. (`died` never reaches an exit
    // path at all: a dead executor runs nothing.)
    if epitaph != Epitaph::FinalResponse {
        return;
    }
    if let Err(e) = deps.launcher.launch(workspace, agent_id) {
        eprintln!("lernie: exit launch for {agent_id}: {e} (accepted crash class, §2.11)");
    }
}
