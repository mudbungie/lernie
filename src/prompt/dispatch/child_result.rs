//! The delivered-child-result and checkpoint-flush seams of the §6
//! binding interpreter — `advance` *is* the interpreter (ARCH §6).
//!
//! Two lifecycle circumstances the hop derives from disk and matches
//! against the flat binding list, exactly as the branch's own terminal is
//! ([`super::super::workflow_actions`]):
//!
//! - **A delivered child result** ([`interpret_pending`]): a *result
//!   message* (§2.6, carrying `terminal_ref:`) the step-boundary drain
//!   ([`super::drain`]) left in the inbox for the interpreter. Its
//!   lifecycle event is named by the returning child's **role**, derived
//!   from the child's dispatch commit subject — the single authoritative
//!   home ([`crate::prompt::role`], no sidecar): `compactor` →
//!   `compactor_return` → the one merge (§2.6); anything else →
//!   `worker_return` → `deliver_result` (transfer + delivery commit).
//! - **A checkpoint flush** ([`run_flush`]): at a step boundary the
//!   executor reads the `compaction:` clock ([`compactor::checkpoint`]);
//!   when a checkpoint is due it runs the `worker_flush` bindings —
//!   `dispatch(compactor)`, forking a compactor off the tip C (§2.6).
//!
//! Circumstance is derived from disk, never dispatched to (§6): the hop
//! reads the inbox and the checkpoint clock, so crash-replay re-derives
//! the identical match. An event with no binding falls back to its §2.6
//! baseline (a worker result delivers, a compactor result merges) — the
//! general path with empty inputs, not a special case.

use super::{drain, transcript, transfer};
use crate::config::{Action, Event, Workflow};
use crate::prompt::{ChildDispatchRequest, Deps, Error, child_dispatch, compactor, inbox, role};
use std::path::{Path, PathBuf};

/// A pending result message (§2.6) awaiting interpretation: the returning
/// child's id (the deposit `<sender>`), its terminal ref, and the inbox
/// file path (moved on delivery, removed on merge).
struct ChildResult {
    child_id: String,
    terminal_ref: String,
    path: PathBuf,
}

/// Whether `agent_id`'s inbox holds any result message (§2.6) — the cheap
/// disk query the hop uses to decide whether to resolve the workflow at
/// all (a no-op hop has none and resolves nothing, §6 lazy resolution).
pub(super) fn has_pending_result(workspace: &Path, agent_id: &str) -> Result<bool, Error> {
    let dir = inbox::inbox_dir(workspace, agent_id);
    for msg in drain::pending(&dir)? {
        if transfer::terminal_ref_of(&read_body(&msg.path)?).is_some() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Interpret every pending result message in `agent_id`'s inbox (§6
/// delivered-child-result circumstance), under the executor lock the hop
/// already holds and against the materialized `worktree`. Each message's
/// event is derived from the returning child's role and its bound actions
/// run in declared order; a consumed message leaves the inbox, a gate-held
/// one stays for a later hop.
pub(super) fn interpret_pending(
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    workflow: &Workflow,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let dir = inbox::inbox_dir(workspace, agent_id);
    for msg in drain::pending(&dir)? {
        let Some(terminal_ref) = transfer::terminal_ref_of(&read_body(&msg.path)?) else {
            continue; // an ordinary steering message — the drain owns it
        };
        let cr = ChildResult {
            child_id: msg.sender,
            terminal_ref,
            path: msg.path,
        };
        let event = child_event(worktree, &cr, deps.git)?;
        for action in child_actions(workflow, event) {
            execute_child(&action, event, agent_id, worktree, &cr, deps)?;
        }
    }
    Ok(())
}

/// Read a deposited message body, mapping I/O to [`Error::Io`].
fn read_body(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(Error::Io)
}

/// Name the lifecycle event of a returning child by its role (§6), derived
/// from the child's dispatch commit subject reachable at its terminal ref
/// (the single authoritative home, [`role`]). A role that reads as `None`
/// (a malformed subject) or anything but `compactor` is a `worker_return`
/// — the deliver-and-transfer baseline (§2.6).
fn child_event(worktree: &Path, cr: &ChildResult, git: &dyn crate::template::GitRunner) -> Result<Event, Error> {
    let derived = role::derive(worktree, &cr.terminal_ref, &cr.child_id, git)?;
    Ok(match derived.as_deref() {
        Some(compactor::COMPACTOR_ROLE) => Event::CompactorReturn,
        _ => Event::WorkerReturn,
    })
}

/// The actions bound to a child-result `event`, or its §2.6 baseline
/// default when unbound: a compactor result merges, every other result
/// delivers. The default lives in the capability, not the config
/// (severability, `docs/PRINCIPLES.md`) — an experiment overrides it by
/// binding the event.
fn child_actions(workflow: &Workflow, event: Event) -> Vec<Action> {
    let bound = workflow.actions_for(event);
    if !bound.is_empty() {
        return bound;
    }
    match event {
        Event::CompactorReturn => vec![Action::CompactionMerge],
        _ => vec![Action::DeliverResult],
    }
}

/// Execute one action at a delivered-child-result event. Ball-1 covers
/// `deliver_result` and `compaction_merge`; other closed-set actions in
/// this context are declined loudly (their executors are tracked
/// follow-ons, `docs/PRINCIPLES.md` Decline illegal operations).
fn execute_child(
    action: &Action,
    event: Event,
    agent_id: &str,
    worktree: &Path,
    cr: &ChildResult,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    match action {
        Action::DeliverResult => deliver_result(worktree, agent_id, cr, deps.git),
        Action::CompactionMerge => compaction_merge(worktree, cr, deps.git),
        other => Err(Error::ActionUnsupported {
            action: format!("{other:?}"),
            event: event.as_str(),
        }),
    }
}

/// `deliver_result` (§2.6): apply the child's work-product transfer as one
/// commit, then move its result message into the transcript as the
/// delivery commit. The message leaves the inbox by the rename inside
/// [`transcript::deliver_message`] — the once-only latch is the ordinary
/// transcript record (§6), not a second fact.
fn deliver_result(
    worktree: &Path,
    agent_id: &str,
    cr: &ChildResult,
    git: &dyn crate::template::GitRunner,
) -> Result<(), Error> {
    transfer::apply(worktree, &cr.child_id, &cr.terminal_ref, git)?;
    transcript::deliver_message(worktree, agent_id, &cr.child_id, &cr.path, git)
}

/// `compaction_merge` (§2.6, the one merge): land the returning compactor
/// branch `--no-ff` into this branch, then consume the trigger message.
/// The merge commit is the record (§5.5 rebuild point) — the compactor's
/// result never enters the parent transcript, so the inbox file is removed
/// rather than delivered.
fn compaction_merge(
    worktree: &Path,
    cr: &ChildResult,
    git: &dyn crate::template::GitRunner,
) -> Result<(), Error> {
    compactor::merge(worktree, &cr.child_id, git)?;
    std::fs::remove_file(&cr.path).map_err(Error::Io)
}

/// Run the `worker_flush` checkpoint at a step boundary (§2.7, §6): if the
/// `compaction:` clock is due for the branch at `worktree`, run the
/// event's bound actions (default: `dispatch(compactor)`), forking a
/// compactor off the tip C. A branch with no `compaction:` block is never
/// due, so this is a no-op — the general path with empty inputs.
pub(super) fn run_flush(
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    workflow: &Workflow,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let state = compactor::state(worktree, deps.clock.now_unix(), false, deps.git)?;
    if !compactor::due(workflow.compaction.as_ref(), &state) {
        return Ok(());
    }
    for action in flush_actions(workflow) {
        execute_flush(&action, workspace, agent_id, worktree, deps)?;
    }
    Ok(())
}

/// The `worker_flush` actions, or the §2.7 baseline default when unbound:
/// dispatch a compactor. Overridable by binding the event.
fn flush_actions(workflow: &Workflow) -> Vec<Action> {
    let bound = workflow.actions_for(Event::WorkerFlush);
    if bound.is_empty() {
        vec![Action::Dispatch {
            role: compactor::COMPACTOR_ROLE.to_string(),
            with: None,
            mode: None,
        }]
    } else {
        bound
    }
}

/// Execute one `worker_flush` action. The compactor dispatch is the only
/// shipped flush action; another closed-set action here is declined.
fn execute_flush(
    action: &Action,
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    match action {
        Action::Dispatch { role, .. } if role == compactor::COMPACTOR_ROLE => {
            dispatch_compactor(workspace, agent_id, worktree, deps)
        }
        other => Err(Error::ActionUnsupported {
            action: format!("{other:?}"),
            event: Event::WorkerFlush.as_str(),
        }),
    }
}

/// Fork a compactor child off `agent_id`'s tip C and start it through the
/// front door (§2.5, §2.7) — an ordinary child dispatch with the compactor
/// soul and boilerplate goal; its return lands the compaction merge on a
/// later hop (`compactor_return`, above).
fn dispatch_compactor(
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    let goal = compactor::compactor_goal(agent_id);
    let req = ChildDispatchRequest {
        repo: workspace,
        parent_branch: agent_id,
        parent_worktree: worktree,
        role: compactor::COMPACTOR_ROLE,
        goal: &goal,
    };
    child_dispatch::run(&req, deps.git, deps.clock, deps.id_gen, deps.launcher)?;
    Ok(())
}

#[cfg(test)]
mod tests;
