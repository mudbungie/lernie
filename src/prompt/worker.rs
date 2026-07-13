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
//! `souls/worker.md` read from the **governing config commit** of the
//! dispatching branch (ARCH §2.2, §4.3 — no per-role path override),
//! derived from ancestry, never from a worktree file. The goal is
//! supplied per call by the dispatcher; in v0.4 it arrives via the
//! `--goal <text>` flag of `lernie dispatch worker`.

use super::clock::{Clock, IdGen};
use super::subagent::{SpawnRequest, spawn_subagent_branch};
use super::{Error, SOULS_DIR};
use crate::template::GitRunner;
use crate::workspace;
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
    // Hyphenated descent (ARCH §2.2): the worker's id and worktree
    // share the same `<parent>-<sub-id>` name; the branch ref is
    // `agents/<id>` (§2.3), applied at the git boundary.
    let sub_branch = format!("{}-{sub_id}", req.parent_branch);
    let sub_worktree = workspace::agent_worktree(req.repo, &sub_branch);

    // Soul resolution per ARCH §2.2 / §4.3 — read `souls/worker.md`
    // from the dispatching branch's governing config commit, verbatim
    // (no template interpolation): the immutable commit is the
    // load-bearing artifact, never a worktree file.
    let commit =
        workspace::governing_config(req.repo, req.parent_branch, git).map_err(|source| {
            Error::Git {
                op: "governing config",
                source,
            }
        })?;
    let soul_rel = format!("{SOULS_DIR}/{WORKER_SOUL_FILE}");
    let soul = workspace::show_control(req.repo, &commit, &soul_rel, git).map_err(|source| {
        Error::ControlRead {
            path: std::path::PathBuf::from(format!("{commit}:{soul_rel}")),
            source,
        }
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
    use crate::workspace::fixture;

    #[test]
    fn run_writes_goal_and_soul_from_the_governing_config_and_returns_sub_branch() {
        // Real git end to end: the worker's soul is read from the
        // parent's governing config commit (§2.2), the branch is
        // `agents/<parent>-<sub-id>`, and the dispatch commit removed
        // the control files from the child's tree.
        let (_h, ws) = fixture::workspace();
        fixture::amend_config(&ws, &[("souls/worker.md", "worker soul body\n")]);
        let parent_wt = fixture::spawn_root(&ws, "20260101-p1");
        let req = WorkerRequest {
            repo: &ws,
            parent_branch: "20260101-p1",
            parent_worktree: &parent_wt,
            goal: "do the thing\n",
        };
        let g = crate::template::RealGit::new();
        let sub_branch = run(
            &req,
            &g,
            &crate::prompt::SystemClock,
            &crate::prompt::NanoIdGen,
        )
        .unwrap();
        assert!(sub_branch.starts_with("20260101-p1-"), "{sub_branch}");

        let sub_wt = workspace::agent_worktree(&ws, &sub_branch);
        assert_eq!(
            std::fs::read_to_string(sub_wt.join("goal.md")).unwrap(),
            "do the thing\n"
        );
        // `show_control` rides `run_capture`, which trims surrounding
        // whitespace — the soul body lands trimmed.
        assert_eq!(
            std::fs::read_to_string(sub_wt.join("soul.md")).unwrap(),
            "worker soul body"
        );
        // The ref namespace holds the child (§2.3)…
        let ids = workspace::agent_ids(&ws, &g).unwrap();
        assert!(ids.contains(&sub_branch), "{ids:?}");
        // …and the child's tree carries no control files (§2.2): the
        // parent's dispatch already removed them, and the child's rm is
        // the total no-op arm.
        assert!(!sub_wt.join("providers.yaml").exists());
        assert!(!sub_wt.join("souls").exists());
    }

    #[test]
    fn run_surfaces_missing_soul_as_control_read() {
        // The default config commit carries souls/worker.md; a config
        // amended to *remove* it makes the governing-config read fail
        // loudly before any spawn side effect.
        let (_h, ws) = fixture::workspace();
        let parent_wt = fixture::spawn_root(&ws, "20260101-p1");
        // Rewrite the worker soul out of the config lineage the parent
        // governs under is impossible (the fork froze it, §2.2) — so
        // exercise the arm with a parent forked off a config that never
        // had one: amend first, then fork.
        let (_h2, ws2) = fixture::workspace();
        let g = crate::template::RealGit::new();
        // Author a config commit without souls/ by removing it.
        let author = ws2.join(".strip");
        let author_str = author.to_string_lossy().to_string();
        g.run(
            &workspace::repo_git(&ws2),
            &["worktree", "add", author_str.as_str(), "config/default"],
        )
        .unwrap();
        g.run(&author, &["rm", "-r", "-q", "souls"]).unwrap();
        g.run(&author, &["commit", "-m", "config: no souls"])
            .unwrap();
        g.run(
            &workspace::repo_git(&ws2),
            &["worktree", "remove", "--force", author_str.as_str()],
        )
        .unwrap();
        let parent_wt2 = fixture::spawn_root(&ws2, "20260101-p2");
        let req = WorkerRequest {
            repo: &ws2,
            parent_branch: "20260101-p2",
            parent_worktree: &parent_wt2,
            goal: "g",
        };
        let err = run(
            &req,
            &g,
            &crate::prompt::SystemClock,
            &crate::prompt::NanoIdGen,
        )
        .unwrap_err();
        assert!(matches!(err, Error::ControlRead { .. }), "got {err:?}");
        // No child branch was spawned before the failure.
        assert!(workspace::agent_ids(&ws2, &g).unwrap().len() == 1);
        let _ = (parent_wt,);
    }

    #[test]
    fn run_surfaces_governing_config_failure_as_git() {
        // A parent branch with no config ancestor (constructed orphan)
        // fails the ancestry derivation loudly (§2.2).
        let (_h, ws) = fixture::workspace();
        let g = crate::template::RealGit::new();
        let wt = workspace::agent_worktree(&ws, "20260101-x1");
        let wt_str = wt.to_string_lossy().to_string();
        g.run(
            &workspace::repo_git(&ws),
            &[
                "worktree",
                "add",
                "--orphan",
                "-b",
                "agents/20260101-x1",
                wt_str.as_str(),
            ],
        )
        .unwrap();
        std::fs::write(wt.join("goal.md"), "g").unwrap();
        g.run(&wt, &["add", "goal.md"]).unwrap();
        g.run(&wt, &["commit", "-m", "orphan"]).unwrap();
        let req = WorkerRequest {
            repo: &ws,
            parent_branch: "20260101-x1",
            parent_worktree: &wt,
            goal: "g",
        };
        let err = run(
            &req,
            &g,
            &crate::prompt::SystemClock,
            &crate::prompt::NanoIdGen,
        )
        .unwrap_err();
        assert!(
            matches!(
                err,
                Error::Git {
                    op: "governing config",
                    ..
                }
            ),
            "got {err:?}"
        );
    }

    #[test]
    fn run_surfaces_worktree_add_failure() {
        // Spawning the same sub-id twice makes the second `worktree add`
        // fail (the ref already exists) — surfaced as the spawn error,
        // which doubles as the structural id-uniqueness guarantee.
        struct FixedClock;
        impl Clock for FixedClock {
            fn now_iso8601(&self) -> String {
                unreachable!("worker::run never reads the iso clock")
            }
            fn now_compact(&self) -> String {
                "ct9".into()
            }
        }
        struct FixedIdGen;
        impl IdGen for FixedIdGen {
            fn short(&self) -> String {
                "feedface".into()
            }
        }
        let (_h, ws) = fixture::workspace();
        let parent_wt = fixture::spawn_root(&ws, "20260101-p1");
        let g = crate::template::RealGit::new();
        let req = WorkerRequest {
            repo: &ws,
            parent_branch: "20260101-p1",
            parent_worktree: &parent_wt,
            goal: "g",
        };
        run(&req, &g, &FixedClock, &FixedIdGen).unwrap();
        let err = run(&req, &g, &FixedClock, &FixedIdGen).unwrap_err();
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
