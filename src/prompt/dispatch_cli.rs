//! CLI handler for `lernie dispatch <role>` (ARCH §3.4) — the front
//! door's role-validity pre-flight, the per-role `--goal` rule, and the
//! hand-off into the child-dispatch primitive. Lives in the lib (not the
//! bin) so the bin stays a thin shim under the repo's 300-line cap and
//! the wiring is unit-testable — the same discipline as `stop::cli_run`
//! and `inbox::cli_run`.
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
use std::path::Path;

/// Role name for the compactor child (§2.7): the one role whose goal is
/// procedure-generated, so `--goal` is rejected. This names the §2.7
/// compaction procedure, not a dispatch-validity allow-list.
const ROLE_COMPACTOR: &str = COMPACTOR_ROLE;

/// Dispatch CLI failures, joined with [`Error`] under one `Display` for a
/// uniform `lernie dispatch <role>:` failure line.
#[derive(Debug)]
pub enum DispatchCliError {
    /// The role is not dispatchable against the calling branch's
    /// governing config commit (not in `providers.yaml`, or its soul is
    /// missing) — the open-set membership failure (§4.3), naming the
    /// config commit consulted.
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
/// Workspace validity was already established by the pre-flight validation
/// (governing config resolution), so no separate `require` guard remains.
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
mod tests {
    use super::*;
    use crate::workspace::fixture;
    use tempfile::TempDir;

    /// A real scaffolded workspace (the `lernie new` core) with a parent
    /// agent branch + worktree — the state `lernie dispatch` is invoked
    /// against in production (§3.4). The default config lists `worker` and
    /// `compactor` with their souls, so both validate off this parent.
    fn scaffolded_repo_with_parent(parent: &str) -> (TempDir, std::path::PathBuf) {
        let (holder, repo) = fixture::workspace();
        fixture::spawn_root(&repo, parent);
        (holder, repo)
    }

    /// A [`Launcher`] that swallows launches — the fork + front-door
    /// deposit is under test, not the real `lernie advance` spawn.
    struct NoopLauncher;
    impl Launcher for NoopLauncher {
        fn launch(&self, _workspace: &Path, _agent_id: &str) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Count sub-agent worktrees forked under `parent`'s id prefix.
    fn sub_count(repo: &Path, parent: &str) -> usize {
        std::fs::read_dir(repo.join(crate::workspace::AGENTS_DIR))
            .unwrap()
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with(&format!("{parent}-"))
            })
            .count()
    }

    #[test]
    fn compactor_dispatch_forks_an_ordinary_compactor_child() {
        // §2.7: the compactor is an ordinary child dispatch — a branch
        // off the dispatching tip with the compactor soul pinned and a
        // boilerplate goal deposited, run by the front door.
        let (_holder, repo) = scaffolded_repo_with_parent("20260101-p1");
        run_with(ROLE_COMPACTOR, &repo, "20260101-p1", None, &NoopLauncher).unwrap();
        assert_eq!(sub_count(&repo, "20260101-p1"), 1);
    }

    #[test]
    fn worker_dispatch_succeeds_and_spawns_a_sub_branch() {
        let (_holder, repo) = scaffolded_repo_with_parent("20260101-p1");
        run_with(
            "worker",
            &repo,
            "20260101-p1",
            Some("do the thing"),
            &NoopLauncher,
        )
        .unwrap();
        assert_eq!(sub_count(&repo, "20260101-p1"), 1);
    }

    #[test]
    fn any_config_role_dispatches_open_set() {
        // The v0.7 criterion through the front door: a third role the
        // config defines (a verifier — zero code) is dispatchable exactly
        // like the template roles. No name list gates it.
        let (_holder, repo) = fixture::workspace();
        let yaml = "roles:\n  worker:\n    provider: anthropic\n    model: sonnet\n  \
                    verifier:\n    provider: anthropic\n    model: sonnet\n";
        fixture::amend_config(
            &repo,
            &[("providers.yaml", yaml), ("souls/verifier.md", "v\n")],
        );
        fixture::spawn_root(&repo, "p9");
        run_with("verifier", &repo, "p9", Some("judge it"), &NoopLauncher).unwrap();
        assert_eq!(sub_count(&repo, "p9"), 1);
    }

    #[test]
    fn undefined_role_is_a_config_validation_failure() {
        let (_holder, repo) = scaffolded_repo_with_parent("p1");
        let err = run_with("no-such-role", &repo, "p1", Some("g"), &NoopLauncher).unwrap_err();
        assert!(matches!(err, DispatchCliError::InvalidRole(_)), "{err}");
        assert!(
            err.to_string()
                .contains("role \"no-such-role\" is not defined in"),
            "{err}"
        );
    }

    #[test]
    fn worker_requires_a_goal() {
        // Through the public `run` (the AdvanceLauncher wiring): validation
        // passes, then the missing `--goal` is refused before any fork.
        let (_holder, repo) = scaffolded_repo_with_parent("p1");
        let err = run("worker", &repo, "p1", None, Path::new("true")).unwrap_err();
        assert_eq!(err.to_string(), "--goal is required for role \"worker\"");
    }

    #[test]
    fn compactor_rejects_a_goal() {
        let (_holder, repo) = scaffolded_repo_with_parent("p1");
        let err = run(ROLE_COMPACTOR, &repo, "p1", Some("g"), Path::new("true")).unwrap_err();
        assert_eq!(
            err.to_string(),
            "--goal is not accepted for role \"compactor\""
        );
    }

    #[test]
    fn inner_errors_render_through_the_shared_display() {
        // Validation passes, but the fork itself fails: with the parent's
        // worktree removed, `git worktree add` (run in it) errors, flowing
        // through `From<Error>` and the shared `Display`.
        let (_holder, repo) = scaffolded_repo_with_parent("p1");
        std::fs::remove_dir_all(repo.join(crate::workspace::AGENTS_DIR).join("p1")).unwrap();
        let err = run_with("worker", &repo, "p1", Some("g"), &NoopLauncher).unwrap_err();
        assert!(matches!(err, DispatchCliError::Inner(_)), "{err}");
        assert!(!err.to_string().is_empty());
    }
}
