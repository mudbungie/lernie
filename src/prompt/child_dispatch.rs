//! Child dispatch — fork plus front door, never a spawn (ARCH §2.5).
//!
//! A tool-call **dispatch** (§2.5) starts a child agent with a goal. The
//! primitive is writes and one deposit, with no process supervision
//! anywhere in it. [`run`] does three things, inline and synchronously:
//!
//! 1. **fork** the child branch off the parent's tip and land the
//!    dispatch commit (§2.3 step 2) — `goal.md` + `soul.md` pinned, the
//!    config's control files removed from the child's tree (§2.2). This
//!    is [`super::subagent::spawn_subagent_branch`], shared with the
//!    compactor (§2.7).
//! 2. **deposit** the dispatch message into the new agent's inbox
//!    through the front door (§2.11): `deposit` then `probe_and_launch`,
//!    exactly what `lernie message` does. The probe finds the fresh
//!    child quiescent and launches its driver — `lernie advance` (§6),
//!    the ordinary driver every agent runs under. There is no
//!    child-specific loop and no worker path; the step loop never
//!    branches on parent/child.
//! 3. **return** the child's id — its address (§2.11) — to the caller,
//!    which the `dispatch` built-in re-emits as the `tool_result`.
//!
//! The child reports back with no executor logic keyed on being a child:
//! at its terminal event `advance` deposits its result message into the
//! **return address**, which for the shipped default is derived from the
//! child's id (`inbox::parent_of` — the dispatcher's address, §2.6). A
//! root records no dispatch and has no such address, so it sends nothing
//! and answers the user instead (§2.4). Return totality (§2.3 step 5) is
//! thereby a property of the dispatch primitive — a dispatch cannot fork
//! without an inbox to deposit into — rather than of loop code.
//!
//! The goal is one input with two projections, both written at dispatch:
//! `goal.md` (pinned standing context, §2.8) and the deposited dispatch
//! message (the on-ramp the child's step-1 drain delivers). They carry
//! the same text by construction and neither is ever rewritten, so no
//! second fact can drift (`docs/PRINCIPLES.md` Single source of truth) —
//! the same shape the root on-ramp uses (`dispatch::run_exchange`).

use super::clock::{Clock, IdGen};
use super::subagent::{SpawnRequest, spawn_subagent_branch};
use super::{Error, SOULS_DIR};
use crate::prompt::inbox::{self, Launcher};
use crate::template::GitRunner;
use crate::workspace;
use std::path::{Path, PathBuf};

/// Inputs to a child dispatch. Built the same way by the dispatch
/// built-in and the `lernie dispatch` CLI regardless of the target role.
pub struct ChildDispatchRequest<'a> {
    /// Workspace repository root. Used for the child's worktree path
    /// (sibling under `agents/`, §2.2), for soul resolution
    /// (`souls/worker.md` in the governing config commit), and as the
    /// deposit target's workspace.
    pub repo: &'a Path,
    /// Dispatching branch name — the parent's full hyphenated descent
    /// (§2.3). The child forks off this branch's tip, its id is
    /// `<parent>-<sub-id>`, and it is the deposit's sender (§2.11
    /// provenance) and the derived return address (§2.6).
    pub parent_branch: &'a str,
    /// The dispatching branch's worktree — where `git worktree add` runs
    /// (any access point onto the one workspace repository, §2.2).
    pub parent_worktree: &'a Path,
    /// The child's role (§2.5, §4.3): selects the pinned soul
    /// (`souls/<role>.md` in the governing config commit) and labels the
    /// dispatch commit. `worker` for an ordinary child, `compactor` for a
    /// compaction dispatch (§2.7) — parent/child is provenance, the role
    /// is what the child *is*.
    pub role: &'a str,
    /// The goal / dispatch message. Written verbatim to the child's
    /// `goal.md` and deposited as its first inbox message.
    pub goal: &'a str,
}

/// Fork a child agent off `req.parent_branch`'s tip and start it through
/// the front door. Returns the child's id (`<parent>-<sub-id>`) — its
/// branch name and its address (§2.3, §2.11). `launcher` is injected so
/// the post-deposit driver launch is testable without spawning a real
/// `lernie advance`; production passes [`inbox::AdvanceLauncher`].
pub fn run(
    req: &ChildDispatchRequest<'_>,
    git: &dyn GitRunner,
    clock: &dyn Clock,
    id_gen: &dyn IdGen,
    launcher: &dyn Launcher,
) -> Result<String, Error> {
    let sub_id = format!("{}-{}", clock.now_compact(), id_gen.short());
    // Hyphenated descent (§2.3): the child's id and worktree share the
    // `<parent>-<sub-id>` name; the `agents/` ref prefix is applied at
    // the git boundary by `spawn_subagent_branch`.
    let sub_branch = format!("{}-{sub_id}", req.parent_branch);
    let sub_worktree = workspace::agent_worktree(req.repo, &sub_branch);

    // Soul resolution (§2.2 / §4.3): read `souls/worker.md` from the
    // dispatching branch's governing config commit, verbatim — the
    // immutable commit is the load-bearing artifact, never a worktree
    // file. A child inherits the parent's config through ancestry.
    let commit =
        workspace::governing_config(req.repo, req.parent_branch, git).map_err(|source| {
            Error::Git {
                op: "governing config",
                source,
            }
        })?;
    let soul_rel = format!("{SOULS_DIR}/{}.md", req.role);
    let soul = workspace::show_control(req.repo, &commit, &soul_rel, git).map_err(|source| {
        Error::ControlRead {
            path: PathBuf::from(format!("{commit}:{soul_rel}")),
            source,
        }
    })?;

    let commit_subject = format!("dispatch: {} [{sub_branch}]", req.role);
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

    // Front door (§2.11): deposit the dispatch message from the parent,
    // then probe-and-launch. The fresh child is quiescent, so the probe
    // launches `lernie advance` — its ordinary driver. This is the whole
    // of "starting" a child: a deposit and the deposit's own launch.
    inbox::deposit(req.repo, &sub_branch, req.parent_branch, req.goal, clock)?;
    inbox::probe_and_launch(req.repo, &sub_branch, launcher).map_err(|source| {
        Error::ExecutorLock {
            path: inbox::inbox_dir(req.repo, &sub_branch),
            source,
        }
    })?;

    Ok(sub_branch)
}

#[cfg(test)]
mod tests;
