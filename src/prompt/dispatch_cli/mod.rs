//! CLI handler for `lernie dispatch <role>` (ARCH §3.4) — the shared id
//! guard, the front door's role-validity pre-flight, the per-role
//! `--goal` rule, and the hand-off into the child-dispatch primitive.
//! Lives in the lib (not the bin) so the bin stays a thin shim under the
//! repo's 300-line cap and the wiring is unit-testable — the same
//! discipline as `stop::cli_run` and `inbox::cli_run`.
//!
//! **The id guard is the same rule at every verb taking an agent id from
//! outside** — `message`, `advance`, `stop`, `dispatch`, `bundle`
//! (README). `dispatch` runs it through the same two shared functions the
//! others do: [`crate::workspace::require`] for the workspace layout
//! (§2.2) and [`crate::workspace::require_agent`] for the dispatching
//! parent (§2.3), both ahead of any governing-config derivation.
//!
//! **The role set is open (§4.3).** This CLI enumerates no role names:
//! validity is the single-home config check ([`crate::prompt::role::validate`])
//! — a role is dispatchable iff the governing config commit lists it and
//! carries its soul — run *before* the fork so a rejected role leaves no
//! branch debris. Exactly one role is special-cased, and only for the
//! `--goal` rule: the compactor's goal is procedure-generated (§2.7), so
//! it is the one role that rejects `--goal`. The closed vocabulary
//! `worker`/`compactor`/`verifier` belongs to the §6 workflow
//! interpreter, never to dispatch validity (§4.3 severability line).

use super::{ChildDispatchRequest, Error};
use crate::prompt::compactor::{COMPACTOR_ROLE, compactor_goal};
use crate::prompt::inbox::{AdvanceLauncher, Launcher};
use crate::prompt::role;
use crate::prompt::{NanoIdGen, SystemClock, child_dispatch};
use crate::template::RealGit;
use crate::workspace;
use std::path::Path;

/// Role name for the compactor child (§2.7): the one role whose goal is
/// procedure-generated, so `--goal` is rejected. This names the §2.7
/// compaction procedure, not a dispatch-validity allow-list.
const ROLE_COMPACTOR: &str = COMPACTOR_ROLE;

/// Dispatch CLI failures, joined with [`Error`] under one `Display` for a
/// uniform `lernie dispatch <role>:` failure line.
#[derive(Debug)]
pub enum DispatchCliError {
    /// The workspace-layout guard declined the path — the shared
    /// [`crate::workspace::require`] voice every id-taking verb uses.
    Layout(workspace::LayoutError),
    /// The dispatching parent has no `agents/*` ref — the shared
    /// [`crate::workspace::require_agent`] voice (§2.3).
    UnknownParent(workspace::UnknownAgent),
    /// The role is not dispatchable against the calling branch's
    /// governing config commit (not in `providers.yaml`, or its soul is
    /// missing) — the open-set membership failure (§4.3).
    InvalidRole(role::validate::Invalid),
    /// `--goal` omitted for a role that requires one (every role but the
    /// compactor).
    GoalRequired(String),
    /// `--goal` supplied for the compactor, whose goal is procedure-
    /// generated (§2.7).
    GoalForbidden(&'static str),
    Inner(Error),
}

impl std::fmt::Display for DispatchCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Layout(e) => write!(f, "{e}"),
            Self::UnknownParent(e) => write!(f, "{e}"),
            Self::InvalidRole(inv) => write!(f, "{inv}"),
            Self::GoalRequired(r) => write!(f, "--goal is required for role {r:?}"),
            Self::GoalForbidden(r) => write!(f, "--goal is not accepted for role {r:?}"),
            Self::Inner(e) => write!(f, "{e}"),
        }
    }
}

impl From<Error> for DispatchCliError {
    fn from(value: Error) -> Self {
        Self::Inner(value)
    }
}

/// Run `lernie dispatch <role> <repo> <branch> [--goal <text>]`
/// (ARCH §3.4). Role-validity and per-role `--goal` violations surface as
/// `Err` for the bin's uniform non-zero exit. Any valid role is dispatched
/// as an ordinary child ([`child_dispatch`], §2.5); roles differ only in
/// the pinned soul (`souls/<role>.md`) and in where the goal comes from —
/// a per-call `--goal` for every role but the compactor, whose goal is
/// the §2.7 boilerplate.
pub fn run(
    role: &str,
    repo: &Path,
    branch: &str,
    goal: Option<&str>,
    driver_target: &Path,
) -> Result<(), DispatchCliError> {
    // The production launcher detach-spawns `lernie advance` (§2.11) at
    // `driver_target` — the running-binary path the exec binding injects
    // (`cmd::Fx::driver_target`, §3.4); the library resolves none itself.
    // The launch decision is tested through [`run_with`] against an
    // injected launcher.
    let launcher = AdvanceLauncher::with_exe(driver_target.to_path_buf());
    run_with(role, repo, branch, goal, &launcher)
}

/// [`run`] with the driver launcher injected — the same
/// launcher-as-parameter discipline as `inbox::probe_and_launch`, so the
/// fork + front-door deposit is exercisable without spawning a real
/// `lernie advance`.
fn run_with(
    role: &str,
    repo: &Path,
    parent_branch: &str,
    goal: Option<&str>,
    launcher: &dyn Launcher,
) -> Result<(), DispatchCliError> {
    // The shared id guard, ahead of everything (§2.2, §2.3): the
    // workspace layout, then the dispatching parent's existence. It is
    // the same sequence `message`, `advance`, `stop` and `bundle` run,
    // through the same two functions — so a missing workspace or a
    // mistyped parent is declined in the product's voice here too,
    // instead of surfacing as a raw git failure from the governing-config
    // derivation below (bl-c89b).
    workspace::require(repo).map_err(DispatchCliError::Layout)?;
    workspace::require_agent(
        repo,
        parent_branch,
        "a child forks off an existing parent (ARCH §2.5)",
        &RealGit::new(),
    )
    .map_err(DispatchCliError::UnknownParent)?;

    // Open-set validity precedes the fork (§4.3): a role absent from the
    // governing config commit (unlisted, or missing its soul) is refused
    // before any branch is created, so a rejected role leaves no debris.
    // One home for the check (`role::validate`), never a name list here.
    role::validate::validate(repo, parent_branch, role, &RealGit::new())
        .map_err(DispatchCliError::InvalidRole)?;

    // Resolve the per-role goal (§2.7): every role carries a per-call
    // `--goal` except the compactor, which rejects it and uses the
    // boilerplate goal the compaction procedure owns instead.
    let goal_text = if role == ROLE_COMPACTOR {
        if goal.is_some() {
            return Err(DispatchCliError::GoalForbidden(ROLE_COMPACTOR));
        }
        compactor_goal(parent_branch)
    } else {
        goal.ok_or_else(|| DispatchCliError::GoalRequired(role.to_owned()))?
            .to_owned()
    };
    dispatch_child(repo, parent_branch, role, &goal_text, launcher)
}

/// Fork `role`'s child off `parent_branch` and start it through the front
/// door (§2.5), printing the child id so the `dispatch` built-in captures
/// it as the `tool_result` address (§3.3 — stdout carries one product).
/// The workspace and the parent were established by [`run_with`]'s shared
/// guard, so nothing is re-checked here.
fn dispatch_child(
    repo: &Path,
    parent_branch: &str,
    role: &str,
    goal: &str,
    launcher: &dyn Launcher,
) -> Result<(), DispatchCliError> {
    let parent_worktree = crate::workspace::agent_worktree(repo, parent_branch);
    let req = ChildDispatchRequest {
        repo,
        parent_branch,
        parent_worktree: &parent_worktree,
        role,
        goal,
        fork_point: None,
    };
    let child = child_dispatch::run(&req, &RealGit::new(), &SystemClock, &NanoIdGen, launcher)?;
    println!("{child}");
    Ok(())
}

#[cfg(test)]
mod tests;
