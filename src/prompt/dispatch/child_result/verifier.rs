//! The verifier gate (ARCH §6 "gate_return_on — the delivery-hold").
//!
//! A **verifier** is an ordinary child agent (§2.5) whose terminal
//! response is its verdict. The gate is realized without any stored flag:
//!
//! - `dispatch(verifier)` on `worker_return` forks a verifier off the
//!   **worker's terminal ref** ([`dispatch`]) — so it inherits the work it
//!   must judge — while its id stays `<parent>-<sub>` (it reports to the
//!   gating parent). The worker result is left undelivered in the inbox:
//!   the *hold* is that disk state, "a worker result present + a verifier
//!   dispatched for it + not yet approved", queried, never flagged
//!   ([`already_gated`], the ancestry of the verifier over the worker ref).
//! - On the verifier's return, its verdict is derived from its own result
//!   message ([`verdict`]): epitaph `final-response` + a leading `APPROVE`
//!   line ⇒ `verifier_approve`; anything else (a `REJECT`, or a
//!   non-final-response epitaph that reached no verdict) ⇒ `verifier_reject`
//!   — a total partition, no third state.
//! - `verifier_approve` drains the held worker result through the ordinary
//!   `deliver_result` (§2.6); `verifier_reject` re-dispatches the worker
//!   with the verdict as `with: verifier.feedback` and discards the
//!   rejected result. The gated worker result is found by ancestry
//!   ([`find_gated_worker`]), not a sidecar link.

use super::ChildResult;
use crate::config::{Action, Event};
use crate::prompt::{ChildDispatchRequest, Deps, Error, WORKER_ROLE, child_dispatch, role};
use crate::template::GitRunner;
use std::path::Path;

/// Role name of a verifier child (§6). Its soul is `souls/verifier.md` in
/// the governing config commit; the gate is otherwise config-only.
pub(super) const VERIFIER_ROLE: &str = "verifier";

/// The `with:` token naming the verifier's own response as the re-dispatch
/// steering on rejection (`dispatch(worker, with: verifier.feedback)`, §6).
pub(super) const FEEDBACK: &str = "verifier.feedback";

/// Derive a returning verifier's verdict event (§6): `final-response` +
/// a leading `APPROVE` line approves; every other finished outcome rejects.
pub(super) fn verdict(cr: &ChildResult) -> Event {
    let approved = cr.epitaph == crate::prompt::inbox::Epitaph::FinalResponse.as_str()
        && cr
            .response
            .as_deref()
            .is_some_and(|r| r.trim_start().starts_with("APPROVE"));
    if approved {
        Event::VerifierApprove
    } else {
        Event::VerifierReject
    }
}

/// Open the gate: fork a verifier off the worker's terminal ref, unless one
/// already gates this worker ([`already_gated`] — the idempotent, disk-
/// derived hold). The worker result is left in the inbox by the caller.
pub(super) fn dispatch(
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    worker: &ChildResult,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    if already_gated(worktree, agent_id, &worker.terminal_ref, deps.git)? {
        return Ok(());
    }
    let goal = verifier_goal(worker.response.as_deref().unwrap_or(""));
    let req = ChildDispatchRequest {
        repo: workspace,
        parent_branch: agent_id,
        parent_worktree: worktree,
        role: VERIFIER_ROLE,
        goal: &goal,
        fork_point: Some(&worker.terminal_ref),
    };
    child_dispatch::run(&req, deps.git, deps.clock, deps.id_gen, deps.launcher)?;
    Ok(())
}

/// Execute a verifier-verdict action (§6): approve drains the held worker
/// result; reject re-dispatches the worker with feedback and discards it.
#[allow(clippy::too_many_arguments)]
pub(super) fn execute(
    action: &Action,
    event: Event,
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    verifier_cr: &ChildResult,
    results: &[ChildResult],
    deps: &Deps<'_>,
) -> Result<(), Error> {
    match (event, action) {
        (Event::VerifierApprove, Action::DeliverResult) => {
            if let Some(worker) = find_gated_worker(results, verifier_cr, worktree, deps.git) {
                super::deliver_result(worktree, agent_id, worker, deps.git)?;
            }
            consume(verifier_cr)
        }
        (Event::VerifierReject, Action::Dispatch { role, with, .. }) if role == WORKER_ROLE => {
            reject(workspace, agent_id, worktree, verifier_cr, results, with.as_deref(), deps)
        }
        _ => Err(Error::ActionUnsupported {
            action: format!("{action:?}"),
            event: event.as_str(),
        }),
    }
}

/// Re-dispatch a fresh worker off the parent tip with the verifier's
/// feedback as its goal, then discard the rejected worker result.
fn reject(
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    verifier_cr: &ChildResult,
    results: &[ChildResult],
    with: Option<&str>,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let feedback = match with {
        Some(FEEDBACK) | None => verifier_cr.response.clone().unwrap_or_default(),
        Some(literal) => literal.to_string(),
    };
    let req = ChildDispatchRequest {
        repo: workspace,
        parent_branch: agent_id,
        parent_worktree: worktree,
        role: WORKER_ROLE,
        goal: &feedback,
        fork_point: None,
    };
    child_dispatch::run(&req, deps.git, deps.clock, deps.id_gen, deps.launcher)?;
    if let Some(worker) = find_gated_worker(results, verifier_cr, worktree, deps.git) {
        consume(worker)?;
    }
    consume(verifier_cr)
}

/// The worker result this verifier gates: the pending result whose
/// terminal ref is an ancestor of the verifier's (the verifier forked off
/// it, §6). Ancestry is the link — no sidecar.
fn find_gated_worker<'a>(
    results: &'a [ChildResult],
    verifier_cr: &ChildResult,
    worktree: &Path,
    git: &dyn GitRunner,
) -> Option<&'a ChildResult> {
    results.iter().find(|cr| {
        cr.child_id != verifier_cr.child_id
            && cr.path.exists()
            && is_ancestor(worktree, &cr.terminal_ref, &verifier_cr.terminal_ref, git)
    })
}

/// Whether a verifier already gates the worker at `worker_ref` (§6): an
/// `agents/<parent>-*` child that is a verifier and forked off (has as
/// ancestor) that ref. Enumerated from git — the hold is a query.
fn already_gated(
    worktree: &Path,
    parent: &str,
    worker_ref: &str,
    git: &dyn GitRunner,
) -> Result<bool, Error> {
    let pattern = format!("agents/{parent}-*");
    let out = git
        .run_capture(
            worktree,
            &["branch", "--list", "--format=%(refname:short)", &pattern],
        )
        .map_err(|source| Error::Git {
            op: "gate branch list",
            source,
        })?;
    for line in out.lines() {
        let branch = line.trim();
        if branch.is_empty() {
            continue;
        }
        let id = branch.strip_prefix("agents/").unwrap_or(branch);
        if role::derive(worktree, branch, id, git)?.as_deref() == Some(VERIFIER_ROLE)
            && is_ancestor(worktree, worker_ref, branch, git)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `git merge-base --is-ancestor <a> <d>`: exit 0 (an `Ok`) iff `a` is an
/// ancestor of `d`. A non-zero exit (or a bad ref) is `Err` — read as "not
/// an ancestor", never propagated.
fn is_ancestor(worktree: &Path, ancestor: &str, descendant: &str, git: &dyn GitRunner) -> bool {
    git.run(worktree, &["merge-base", "--is-ancestor", ancestor, descendant])
        .is_ok()
}

/// Remove a consumed inbox result message.
fn consume(cr: &ChildResult) -> Result<(), Error> {
    std::fs::remove_file(&cr.path).map_err(Error::Io)
}

/// The boilerplate goal handed to a verifier at dispatch (§6). The worker's
/// work is already in the verifier's inherited tree; the response is quoted
/// for convenience.
fn verifier_goal(worker_response: &str) -> String {
    format!(
        "You are a verifier judging a worker's completed work — its output below and the \
         work products in your tree. Reply with a single leading verdict line: `APPROVE` if \
         the work satisfies the task, or `REJECT` followed by specific, actionable feedback.\n\n\
         Worker's final response:\n{worker_response}\n"
    )
}
