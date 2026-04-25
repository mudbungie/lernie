//! Terminal-compaction dispatch (ARCH §2.7, v0.3 scope per §12).
//!
//! The v1 compactor is a subagent dispatched off a branch's tip with
//! a goal instructing it to summarize the branch's work. Its toolset
//! is in [`tools`]: `write_summary` and `mark_for_deletion`. v0.3
//! ships a stub that uses neither a model call nor real deletion
//! marking — what it demonstrates is the **dispatch shape**: the
//! compactor runs on its own branch off the dispatching branch's tip
//! and merges back through the normal merge protocol (§2.6). That
//! shape is the load-bearing part, because the generalized dispatch
//! primitive in v0.4 reuses it verbatim; this is the "One obvious
//! path" principle instantiated (see `docs/PRINCIPLES.md`).
//!
//! The stub reads the terminal [`StepResponse`] off the dispatching
//! branch's tree, writes a one-line summary of the form
//! `conversation <id>: <assistant-response text>`, and commits it on
//! the compactor branch.

pub mod tools;

use super::merge::rebase_and_merge;
use super::step::{RESPONSE_FILE, StepResponse, step_dir_rel};
use super::{Clock, Error, IdGen};
use crate::template::GitRunner;
use std::path::Path;
use tools::write_summary;

/// v0.3 has exactly one step per root conversation (§12). The
/// compactor reads that one step's response to build its summary.
const TERMINAL_STEP_SEQ: u32 = 1;

/// Shape of the terminal-compaction dispatch. Kept as a struct so the
/// CLI entry point (`lernie dispatch compactor`) and the in-process
/// caller build the same request the same way — §3.4 calls out that
/// the two paths share a command surface, not just code.
pub struct CompactorRequest<'a> {
    /// Repo root. Used for resolving the compactor's worktree path
    /// (a sibling of `root/`, §2.2).
    pub repo: &'a Path,
    /// The dispatching conversation's id, which is also its branch
    /// name (ARCH §2.3 — bare hyphenated descent, no prefix). The
    /// compactor spawns off this branch's tip and merges back into it.
    pub parent_conv_id: &'a str,
    /// Path to the dispatching branch's worktree. The stub reads
    /// `response.json` from this worktree (which is where it was just
    /// committed). The final `--no-ff` merge runs here too, so the
    /// parent branch's ref advances to the merge commit.
    pub parent_worktree: &'a Path,
}

/// Run the terminal compactor against `req`. The stub writes a
/// single-line summary for the dispatching branch and merges the
/// result back into it; the caller is responsible for merging the
/// dispatching branch onto its parent (§2.6).
pub fn run(
    req: &CompactorRequest<'_>,
    git: &dyn GitRunner,
    clock: &dyn Clock,
    id_gen: &dyn IdGen,
) -> Result<(), Error> {
    let cmp_id = format!("{}-{}", clock.now_compact(), id_gen.short());
    // Hyphenated descent (ARCH §2.2): the compactor's branch and
    // worktree both live under `<parent>-<cmp>`. Branch name and
    // directory name are intentionally identical so on-disk layout
    // and ref namespace are isomorphic.
    let cmp_branch = format!("{}-{cmp_id}", req.parent_conv_id);
    let cmp_worktree = req.repo.join(&cmp_branch);

    // Read the terminal response off the parent worktree (where it
    // was just committed). The compactor's input is the parent tip's
    // tree; in v0.4+ that input will be delivered through the
    // dispatch contract rather than a direct filesystem read.
    let summary = build_summary(req.parent_worktree, req.parent_conv_id)?;

    spawn_compactor_branch(
        req.parent_worktree,
        &cmp_worktree,
        &cmp_branch,
        req.parent_conv_id,
        git,
    )?;

    // Dispatch commit (§2.10): goal.md lands before any model call.
    // v0.3 has no model call, but the shape is the load-bearing
    // part — v0.4+ inherits the same dispatch surface for the real
    // compactor agent.
    write_goal(&cmp_worktree, req.parent_conv_id)?;
    commit_goal(&cmp_worktree, req.parent_conv_id, git)?;

    let summary_rel = write_summary(&cmp_worktree, &summary)?;
    commit_summary(&cmp_worktree, &summary_rel, req.parent_conv_id, git)?;

    // `repo` arg of `rebase_and_merge` is just the cwd for the
    // `worktree remove` step. The conv-repo root itself is not a git
    // checkout in v0.3 (the `.git` lives inside `root/`, ARCH §2.2),
    // so we use the parent worktree — which shares the same .git
    // dir — as the cwd. Either linked worktree would do; `parent_worktree`
    // is the one already in scope.
    rebase_and_merge(
        req.parent_worktree,
        req.parent_conv_id,
        req.parent_worktree,
        &cmp_worktree,
        &cmp_branch,
        git,
    )?;

    Ok(())
}

/// Boilerplate goal handed to the compactor at dispatch time. The
/// branch name interpolates so the compactor knows which branch it is
/// summarizing without a separate context handoff. v0.3 stub does not
/// read the file (no model call); the text is here so v0.4+ inherits
/// the dispatch shape unchanged.
pub(crate) fn compactor_goal(parent_branch: &str) -> String {
    format!(
        "You are the terminal compactor for branch `{parent_branch}`.\n\
         \n\
         Read the branch's work and produce a signal-preserving summary using the\n\
         `write_summary` tool. The harness writes it to the next\n\
         `summary/<NNN>.md` on this branch.\n\
         \n\
         Use `mark_for_deletion` to nominate superseded files (e.g. raw step dirs)\n\
         for removal. Do not nominate the previous summary; the harness deletes it\n\
         automatically before merging back.\n\
         \n\
         Decide relevance against the parent branch's goal at `goal.md`.\n"
    )
}

/// Write `goal.md` to the compactor's worktree. Mirrors the
/// dispatch-side `write_snapshot`'s goal write (§2.8) — every
/// non-root branch carries a goal at the worktree root.
fn write_goal(cmp_worktree: &Path, parent_branch: &str) -> Result<(), Error> {
    std::fs::create_dir_all(cmp_worktree)?;
    std::fs::write(cmp_worktree.join("goal.md"), compactor_goal(parent_branch))?;
    Ok(())
}

/// `git add` the goal then `git commit` the dispatch snapshot. Names
/// the conversation in the message so history reads `compaction:
/// dispatch [<conv-id>]` analogously to the snapshot commit.
fn commit_goal(
    cmp_worktree: &Path,
    parent_conv_id: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    git.run(cmp_worktree, &["add", "goal.md"])
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("compaction: dispatch [{parent_conv_id}]");
    git.run(cmp_worktree, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}

/// Build the stub summary body from the dispatching branch's terminal
/// response.
fn build_summary(parent_worktree: &Path, parent_conv_id: &str) -> Result<String, Error> {
    let step_rel = step_dir_rel(parent_conv_id, TERMINAL_STEP_SEQ);
    let response_path = parent_worktree.join(step_rel).join(RESPONSE_FILE);
    let bytes = std::fs::read(&response_path)?;
    let response: StepResponse = serde_json::from_slice(&bytes).map_err(Error::AdapterJson)?;
    Ok(format!(
        "conversation {parent_conv_id}: {}\n",
        response.assistant_response
    ))
}

/// `git worktree add -b <cmp_branch> <cmp_worktree> <parent_branch>`,
/// run inside the parent worktree (which has access to the same `.git`
/// dir as `root/`, §2.2). Spawning from the parent worktree means we
/// do not need to know where `root/` is — the compactor stays scoped
/// to its dispatching branch.
fn spawn_compactor_branch(
    parent_worktree: &Path,
    cmp_worktree: &Path,
    cmp_branch: &str,
    parent_branch: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let wt_str = cmp_worktree.to_string_lossy().to_string();
    git.run(
        parent_worktree,
        &[
            "worktree",
            "add",
            "-b",
            cmp_branch,
            wt_str.as_str(),
            parent_branch,
        ],
    )
    .map_err(|source| Error::Git {
        op: "worktree add",
        source,
    })
}

fn commit_summary(
    cmp_worktree: &Path,
    summary_rel: &str,
    parent_conv_id: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    git.run(cmp_worktree, &["add", summary_rel])
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("compaction: terminal summary [{parent_conv_id}]");
    git.run(cmp_worktree, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}

#[cfg(test)]
mod tests;
