//! CLI handler for `lernie dispatch <role>` (ARCH §3.4) — the per-role
//! argument-shape rules and the hand-off into the compactor / worker
//! backends. Lives in the lib (not the bin) so the bin stays a thin
//! shim under the repo's 300-line cap and the wiring is unit-testable —
//! the same discipline as `stop::cli_run` and `inbox::cli_run`.

use super::{CompactorRequest, Error, WorkerRequest};
use crate::prompt::{NanoIdGen, SystemClock, compactor, worker};
use crate::template::RealGit;
use std::path::Path;

/// Role name for the v0.3 terminal compactor (§2.7); `--goal` rejected.
const ROLE_COMPACTOR: &str = "compactor";
/// Role name for the v0.4 worker subagent (§2.5); `--goal` required.
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
/// uniform non-zero exit.
pub fn run(
    role: &str,
    repo: &Path,
    branch: &str,
    goal: Option<&str>,
) -> Result<(), DispatchCliError> {
    match role {
        ROLE_COMPACTOR => run_compactor(repo, branch, goal),
        ROLE_WORKER => run_worker(repo, branch, goal),
        other => Err(DispatchCliError::UnknownRole(other.to_owned())),
    }
}

fn run_compactor(repo: &Path, branch: &str, goal: Option<&str>) -> Result<(), DispatchCliError> {
    if goal.is_some() {
        return Err(DispatchCliError::GoalForbidden(ROLE_COMPACTOR));
    }
    crate::workspace::require(repo).map_err(crate::prompt::Error::from)?;
    let worktree = crate::workspace::agent_worktree(repo, branch);
    let req = CompactorRequest {
        repo,
        parent_conv_id: branch,
        parent_worktree: &worktree,
    };
    Ok(compactor::run(
        &req,
        &RealGit::new(),
        &SystemClock,
        &NanoIdGen,
    )?)
}

fn run_worker(
    repo: &Path,
    parent_branch: &str,
    goal: Option<&str>,
) -> Result<(), DispatchCliError> {
    let goal = goal.ok_or(DispatchCliError::GoalRequired(ROLE_WORKER))?;
    crate::workspace::require(repo).map_err(crate::prompt::Error::from)?;
    let parent_worktree = crate::workspace::agent_worktree(repo, parent_branch);
    let req = WorkerRequest {
        repo,
        parent_branch,
        parent_worktree: &parent_worktree,
        goal,
    };
    // Print the spawned branch so the `dispatch` built-in captures it as
    // the `tool_result` handle (ARCH §3.3) — stdout carries one product.
    let sub_branch = worker::run(&req, &RealGit::new(), &SystemClock, &NanoIdGen)?;
    println!("{sub_branch}");
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

    #[test]
    fn compactor_dispatch_succeeds_against_a_real_repo() {
        let (_holder, repo) = scaffolded_repo_with_parent("20260101-p1");
        run("compactor", &repo, "20260101-p1", None).unwrap();
        // The compaction merge landed the summary on the parent branch.
        assert!(
            crate::workspace::agent_worktree(&repo, "20260101-p1")
                .join("summary/001.md")
                .exists()
        );
    }

    #[test]
    fn worker_dispatch_succeeds_and_spawns_a_sub_branch() {
        let (_holder, repo) = scaffolded_repo_with_parent("20260101-p1");
        run("worker", &repo, "20260101-p1", Some("do the thing")).unwrap();
        // Exactly one sub-agent worktree appeared under agents/ with
        // the parent's id prefix (hyphenated descent, §2.3).
        let subs = std::fs::read_dir(repo.join(crate::workspace::AGENTS_DIR))
            .unwrap()
            .flatten()
            .filter(|e| {
                let n = e.file_name().to_string_lossy().into_owned();
                n.starts_with("20260101-p1-")
            })
            .count();
        assert_eq!(subs, 1);
    }

    #[test]
    fn unknown_role_is_refused_with_its_name() {
        let err = run("no-such-role", Path::new("/tmp"), "b1", None).unwrap_err();
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
