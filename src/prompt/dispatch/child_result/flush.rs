//! The checkpoint-flush seam of the §6 binding interpreter.
//!
//! A **checkpoint flush** ([`run_flush`]): a due `compaction:` clock at a
//! step boundary runs `worker_flush` → `dispatch(compactor)`. A branch with
//! no `compaction:` block is never due, so the whole seam is a no-op — the
//! general path with empty inputs.

use crate::config::{Action, Event, Workflow};
use crate::prompt::{ChildDispatchRequest, Deps, Error, child_dispatch, compactor};
use std::path::Path;

/// Run the `worker_flush` checkpoint at a step boundary (§2.7, §6): if the
/// `compaction:` clock is due for the branch at `worktree`, run the
/// event's bound actions (default: `dispatch(compactor)`), forking a
/// compactor off the tip C. A branch with no `compaction:` block is never
/// due, so this is a no-op — the general path with empty inputs.
pub(in crate::prompt::dispatch) fn run_flush(
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    workflow: &Workflow,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    // No `compaction:` clock → never due; skip the git-derived state (§2.7).
    if workflow.compaction.is_none() {
        return Ok(());
    }
    let state = compactor::state(worktree, agent_id, deps.clock.now_unix(), false, deps.git)?;
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
/// front door (§2.5, §2.7); its return lands the compaction merge on a
/// later hop (`compactor_return`).
fn dispatch_compactor(
    workspace: &Path,
    agent_id: &str,
    worktree: &Path,
    deps: &Deps<'_>,
) -> Result<(), Error> {
    // The boilerplate goal quotes the dispatching branch's own goal
    // (§2.7), read from the worktree we are forking off.
    let goal = compactor::compactor_goal(worktree, agent_id)?;
    let req = ChildDispatchRequest {
        repo: workspace,
        parent_branch: agent_id,
        parent_worktree: worktree,
        role: compactor::COMPACTOR_ROLE,
        goal: &goal,
        name: None,
        fork_point: None,
    };
    child_dispatch::run_procedure(&req, deps.git, deps.clock, deps.id_gen, deps.launcher)
}
