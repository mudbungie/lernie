//! CLI handler for `lernie dispatch <role>` (ARCH §3.4) — the per-role
//! argument-shape rules and the hand-off into the compactor / worker
//! backends. Lives in the lib (not the bin) so the bin stays a thin
//! shim under the repo's 300-line cap and the wiring is unit-testable —
//! the same discipline as `stop::cli_run` and `inbox::cli_run`.

use super::{ChildDispatchRequest, Error};
use crate::prompt::compactor::{COMPACTOR_ROLE, compactor_goal};
use crate::prompt::inbox::{AdvanceLauncher, Launcher};
use crate::prompt::{NanoIdGen, SystemClock, child_dispatch};
use crate::template::RealGit;
use std::path::Path;

/// Role name for the compactor child (§2.7). An ordinary child dispatch
/// like the worker, but with a boilerplate goal, so `--goal` is rejected.
const ROLE_COMPACTOR: &str = COMPACTOR_ROLE;
/// Role name for the worker subagent (§2.5); `--goal` required.
const ROLE_WORKER: &str = "worker";

/// Dispatch CLI argument-shape errors, joined with [`Error`] under one
/// `Display` for a uniform `lernie dispatch <role>:` failure line.
#[derive(Debug)]
pub enum DispatchCliError {
    UnknownRole(String),
    GoalRequired(&'static str),
    GoalForbidden(&'static str),
    Inner(Error),
}

impl std::fmt::Display for DispatchCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownRole(r) => write!(f, "unknown role {r:?}"),
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
/// (ARCH §3.4). Per-role `--goal` rules surface as `Err` for the bin's
/// uniform non-zero exit. Both roles are ordinary child dispatches
/// ([`child_dispatch`], §2.5, §2.7); they differ only in the pinned soul
/// (`souls/<role>.md`) and in where the goal comes from — a worker carries
/// a per-call `--goal`, a compactor a boilerplate goal (§2.7).
pub fn run(
    role: &str,
    repo: &Path,
    branch: &str,
    goal: Option<&str>,
) -> Result<(), DispatchCliError> {
    // The production launcher detach-spawns `lernie advance` (§2.11); its
    // construction is pure (resolves `current_exe`, no spawn), so the
    // spawn-free wiring is covered here and the launch decision is tested
    // through [`run_with`] against an injected launcher.
    let launcher = AdvanceLauncher::current().map_err(crate::prompt::Error::from)?;
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
    // Resolve the per-role goal (§2.7): a worker requires `--goal`; a
    // compactor rejects it and uses the boilerplate goal instead.
    let goal_text = match role {
        ROLE_WORKER => goal.ok_or(DispatchCliError::GoalRequired(ROLE_WORKER))?.to_owned(),
        ROLE_COMPACTOR if goal.is_some() => {
            return Err(DispatchCliError::GoalForbidden(ROLE_COMPACTOR));
        }
        ROLE_COMPACTOR => compactor_goal(parent_branch),
        other => return Err(DispatchCliError::UnknownRole(other.to_owned())),
    };
    dispatch_child(repo, parent_branch, role, &goal_text, launcher)
}

/// Fork `role`'s child off `parent_branch` and start it through the front
/// door (§2.5), printing the child id so the `dispatch` built-in captures
/// it as the `tool_result` address (§3.3 — stdout carries one product).
fn dispatch_child(
    repo: &Path,
    parent_branch: &str,
    role: &str,
    goal: &str,
    launcher: &dyn Launcher,
) -> Result<(), DispatchCliError> {
    crate::workspace::require(repo).map_err(crate::prompt::Error::from)?;
    let parent_worktree = crate::workspace::agent_worktree(repo, parent_branch);
    let req = ChildDispatchRequest {
        repo,
        parent_branch,
        parent_worktree: &parent_worktree,
        role,
        goal,
    };
    let child = child_dispatch::run(&req, &RealGit::new(), &SystemClock, &NanoIdGen, launcher)?;
    println!("{child}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// A real scaffolded workspace (the `lernie new` core) with a
    /// parent agent branch + worktree — the state `lernie dispatch` is
    /// invoked against in production (§3.4).
    fn scaffolded_repo_with_parent(parent: &str) -> (TempDir, std::path::PathBuf) {
        let (holder, repo) = crate::workspace::fixture::workspace();
        crate::workspace::fixture::spawn_root(&repo, parent);
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
        // boilerplate goal deposited, run by the front door. No terminal
        // stub, no synchronous summary.
        let (_holder, repo) = scaffolded_repo_with_parent("20260101-p1");
        run_with(ROLE_COMPACTOR, &repo, "20260101-p1", None, &NoopLauncher).unwrap();
        assert_eq!(sub_count(&repo, "20260101-p1"), 1);
    }

    #[test]
    fn worker_dispatch_succeeds_and_spawns_a_sub_branch() {
        let (_holder, repo) = scaffolded_repo_with_parent("20260101-p1");
        run_with(ROLE_WORKER, &repo, "20260101-p1", Some("do the thing"), &NoopLauncher).unwrap();
        assert_eq!(sub_count(&repo, "20260101-p1"), 1);
    }

    #[test]
    fn unknown_role_is_refused_with_its_name() {
        let err =
            run_with("no-such-role", Path::new("/tmp"), "b1", None, &NoopLauncher).unwrap_err();
        assert!(matches!(err, DispatchCliError::UnknownRole(_)));
        assert_eq!(err.to_string(), "unknown role \"no-such-role\"");
    }

    #[test]
    fn worker_requires_a_goal() {
        let err = run(ROLE_WORKER, Path::new("/tmp"), "b1", None).unwrap_err();
        assert_eq!(err.to_string(), "--goal is required for role \"worker\"");
    }

    #[test]
    fn compactor_rejects_a_goal() {
        let err = run(ROLE_COMPACTOR, Path::new("/tmp"), "b1", Some("g")).unwrap_err();
        assert_eq!(
            err.to_string(),
            "--goal is not accepted for role \"compactor\""
        );
    }

    #[test]
    fn inner_errors_render_through_the_shared_display() {
        // A worker dispatch against a nonexistent repo fails in the
        // backend; the error flows through `From<Error>` and `Display`.
        let err = run(
            ROLE_WORKER,
            Path::new("/no/such/repo"),
            "b1",
            Some("do the thing"),
        )
        .unwrap_err();
        assert!(matches!(err, DispatchCliError::Inner(_)));
        assert!(!err.to_string().is_empty());
    }
}
