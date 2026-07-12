//! Worker subagent dispatch (ARCH §2.5, v0.4 Phase 1).
//!
//! A *worker* is a subagent dispatched by a tool call from a parent
//! conversation, with a per-call goal. v0.4 Phase 1 lands the dispatch
//! shape only: spawn the branch + worktree, write `goal.md` and
//! `soul.md`, commit the dispatch snapshot, exit. The subagent's own
//! step loop, the `dispatch` tool that puts it in front of a parent
//! agent, and the result-message return path (an inbox deposit on
//! termination, §2.11) slot in on top of this commit.
//!
//! Soul resolution mirrors the root-conversation path (`prompt::run`):
//! `<conv-repo>/souls/worker.md` (ARCH §4.3 — no per-role path
//! override). The goal is supplied per call by the dispatcher; in v0.4
//! it arrives via the `--goal <text>` flag of `lernie dispatch worker`.

use super::clock::{Clock, IdGen};
use super::subagent::{SpawnRequest, spawn_subagent_branch};
use super::{Error, SOULS_DIR};
use crate::template::GitRunner;
use std::path::Path;

/// Inputs to a worker dispatch. Mirrors [`super::CompactorRequest`] in
/// shape so future v0.4 callers (the `dispatch` tool, Phase 2) build
/// the same request the same way regardless of which role they target.
pub struct WorkerRequest<'a> {
    /// Conversation-repo root. Used for the worker's worktree path
    /// (sibling of `root/`, §2.2) and for soul resolution
    /// (`<repo>/souls/worker.md`).
    pub repo: &'a Path,
    /// Dispatching branch name — bare hyphenated descent of the
    /// parent's conv-id chain. The worker branch is spawned off this
    /// branch's tip and the worker conv-id is `<parent>-<sub-id>`
    /// (§2.2 / §2.3).
    pub parent_branch: &'a str,
    /// Path to the dispatching branch's worktree. The worker spawns
    /// off this worktree's `.git` (the conv-repo's git dir lives in
    /// `root/`, §2.2).
    pub parent_worktree: &'a Path,
    /// Per-call goal text — written verbatim to the worker's
    /// `goal.md` and committed as the dispatch commit's tree.
    pub goal: &'a str,
}

/// Soul filename for the worker role. Pinned by ARCH §4.3 (soul =
/// `<conv-repo>/souls/<role>.md` — no per-role path override).
const WORKER_SOUL_FILE: &str = "worker.md";
/// Role name as the CLI / dispatcher sees it. Single source for the
/// CLI subcommand string, the dispatch commit subject, and the soul
/// filename's stem.
pub(crate) const WORKER_ROLE: &str = "worker";

/// Spawn a worker subagent off `req.parent_branch`'s tip. Phase 1
/// stops at the dispatch commit — no model call, no tool loop, no
/// merge-back. Returns the new branch name (the worker's full
/// conv-id, `<parent>-<sub-id>`) so callers can locate the dispatch
/// commit without a separate lookup.
pub fn run(
    req: &WorkerRequest<'_>,
    git: &dyn GitRunner,
    clock: &dyn Clock,
    id_gen: &dyn IdGen,
) -> Result<String, Error> {
    let sub_id = format!("{}-{}", clock.now_compact(), id_gen.short());
    // Hyphenated descent (ARCH §2.2): the worker's branch and worktree
    // share the same `<parent>-<sub-id>` name.
    let sub_branch = format!("{}-{sub_id}", req.parent_branch);
    let sub_worktree = req.repo.join(&sub_branch);

    // Soul resolution per ARCH §4.3 — no path override. Read the file
    // verbatim (no template interpolation) so the on-disk soul is the
    // load-bearing artifact.
    let soul_path = req.repo.join(SOULS_DIR).join(WORKER_SOUL_FILE);
    let soul = std::fs::read_to_string(&soul_path).map_err(|source| Error::SoulRead {
        path: soul_path.clone(),
        source,
    })?;

    let commit_subject = format!("dispatch: {WORKER_ROLE} [{sub_branch}]");
    spawn_subagent_branch(
        &SpawnRequest {
            parent_worktree: req.parent_worktree,
            parent_branch: req.parent_branch,
            sub_branch: &sub_branch,
            sub_worktree: &sub_worktree,
            goal_text: req.goal,
            soul_text: Some(&soul),
            commit_subject: &commit_subject,
        },
        git,
    )?;

    Ok(sub_branch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::io;
    use std::path::PathBuf;

    struct FixedClock;
    impl Clock for FixedClock {
        fn now_iso8601(&self) -> String {
            unreachable!("worker::run never reads the iso clock")
        }
        fn now_compact(&self) -> String {
            "ct-9".into()
        }
    }
    struct FixedIdGen;
    impl IdGen for FixedIdGen {
        fn short(&self) -> String {
            "feedface".into()
        }
    }

    #[derive(Default)]
    struct StubGit {
        runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
        fail_at: Option<usize>,
    }
    impl StubGit {
        fn ok() -> Self {
            Self::default()
        }
        fn failing_at(idx: usize) -> Self {
            Self {
                fail_at: Some(idx),
                ..Self::default()
            }
        }
    }
    impl GitRunner for StubGit {
        fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
            let mut runs = self.runs.borrow_mut();
            let idx = runs.len();
            runs.push((
                dest.to_path_buf(),
                args.iter().map(|s| (*s).to_owned()).collect(),
            ));
            if self.fail_at == Some(idx) {
                Err(io::Error::other(format!("stub git fail at {idx}")))
            } else {
                Ok(())
            }
        }
        fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
            self.run(dest, args)?;
            Ok(String::new())
        }
    }

    fn tmpdir() -> tempfile::TempDir {
        tempfile::TempDir::new().unwrap()
    }

    fn layout(parent_branch: &str, soul: Option<&str>) -> (tempfile::TempDir, PathBuf) {
        let repo = tmpdir();
        let parent_wt = repo.path().join(parent_branch);
        std::fs::create_dir_all(&parent_wt).unwrap();
        if let Some(text) = soul {
            let souls = repo.path().join(SOULS_DIR);
            std::fs::create_dir_all(&souls).unwrap();
            std::fs::write(souls.join(WORKER_SOUL_FILE), text).unwrap();
        }
        (repo, parent_wt)
    }

    #[test]
    fn run_writes_goal_and_soul_and_returns_sub_branch() {
        let (repo, parent_wt) = layout("p1", Some("worker soul body\n"));
        let req = WorkerRequest {
            repo: repo.path(),
            parent_branch: "p1",
            parent_worktree: &parent_wt,
            goal: "do the thing\n",
        };
        let git = StubGit::ok();
        let sub_branch = run(&req, &git, &FixedClock, &FixedIdGen).unwrap();

        assert_eq!(sub_branch, "p1-ct-9-feedface");
        let sub_wt = repo.path().join(&sub_branch);

        let runs = git.runs.borrow();
        // 0: worktree add (in parent worktree)
        assert_eq!(runs[0].0, parent_wt);
        assert_eq!(runs[0].1[..4], ["worktree", "add", "-b", &sub_branch]);
        assert_eq!(runs[0].1[4], sub_wt.to_string_lossy().to_string());
        assert_eq!(runs[0].1[5], "p1");
        // 1: add goal.md soul.md (in sub worktree)
        assert_eq!(runs[1].0, sub_wt);
        assert_eq!(runs[1].1, vec!["add", "goal.md", "soul.md"]);
        // 2: commit dispatch (in sub worktree)
        assert_eq!(runs[2].0, sub_wt);
        assert_eq!(runs[2].1[0], "commit");
        assert_eq!(runs[2].1[2], format!("dispatch: worker [{sub_branch}]"));

        // Files written verbatim from input.
        assert_eq!(
            std::fs::read_to_string(sub_wt.join("goal.md")).unwrap(),
            "do the thing\n"
        );
        assert_eq!(
            std::fs::read_to_string(sub_wt.join("soul.md")).unwrap(),
            "worker soul body\n"
        );
    }

    #[test]
    fn run_surfaces_missing_soul_as_soulread() {
        // No souls/worker.md on disk — the helper bails before any
        // git command runs (the spawn helper is the side-effecting
        // part, and we want the failure to land before mutating the
        // parent worktree's state).
        let (repo, parent_wt) = layout("p1", None);
        let req = WorkerRequest {
            repo: repo.path(),
            parent_branch: "p1",
            parent_worktree: &parent_wt,
            goal: "g",
        };
        let git = StubGit::ok();
        let err = run(&req, &git, &FixedClock, &FixedIdGen).unwrap_err();
        match err {
            Error::SoulRead { path, .. } => {
                assert!(path.ends_with("souls/worker.md"), "got {}", path.display());
            }
            other => panic!("expected SoulRead, got {other:?}"),
        }
        assert!(git.runs.borrow().is_empty(), "no git ops before soul read");
    }

    #[test]
    fn run_surfaces_worktree_add_failure() {
        let (repo, parent_wt) = layout("p1", Some("soul"));
        let req = WorkerRequest {
            repo: repo.path(),
            parent_branch: "p1",
            parent_worktree: &parent_wt,
            goal: "g",
        };
        let git = StubGit::failing_at(0);
        let err = run(&req, &git, &FixedClock, &FixedIdGen).unwrap_err();
        assert!(
            matches!(
                err,
                Error::Git {
                    op: "worktree add",
                    ..
                }
            ),
            "got {err:?}"
        );
    }
}
