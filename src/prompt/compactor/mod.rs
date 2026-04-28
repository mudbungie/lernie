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
//! The stub writes a placeholder one-line summary identifying the
//! dispatching conversation. v0.3.1 dropped its previous read of
//! `response.json` to honor §2.3's diagnostic-only contract — no
//! harness code path may read `request.json` or `response.json` at
//! runtime. The real model-driven summary lands in v0.4+ where
//! input is delivered through the dispatch contract.

pub mod tools;

use super::merge::rebase_and_merge;
use super::subagent::{SpawnRequest, spawn_subagent_branch};
use super::{Clock, Error, IdGen};
use crate::template::GitRunner;
use std::path::Path;
use tools::write_summary;

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
    /// Path to the dispatching branch's worktree. The compactor
    /// spawns its own branch off this worktree's `.git` (the
    /// conv-repo's git dir lives in `root/`, §2.2) and the final
    /// `--no-ff` merge runs here, advancing the parent branch's ref
    /// to the merge commit.
    pub parent_worktree: &'a Path,
}

/// Run the terminal compactor against `req`. The stub writes a
/// single-line placeholder summary for the dispatching branch and
/// merges the result back into it; the caller is responsible for
/// merging the dispatching branch onto its parent (§2.6).
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

    // Stub summary: just identifies the parent conversation. Reading
    // `response.json` would violate §2.3's diagnostic-only contract;
    // the real summary in v0.4+ comes through the dispatch contract.
    let summary = build_summary(req.parent_conv_id);

    // Dispatch (§2.5 / §2.7): branch + worktree + goal.md + dispatch
    // commit. The v0.3 compactor stub omits soul.md because it has no
    // model call; the shared helper accepts `soul_text: None` for that
    // case. v0.4+ wires a real compactor agent that fills it in.
    let goal_text = compactor_goal(req.parent_conv_id);
    let commit_subject = format!("compaction: dispatch [{}]", req.parent_conv_id);
    spawn_subagent_branch(
        &SpawnRequest {
            parent_worktree: req.parent_worktree,
            parent_branch: req.parent_conv_id,
            sub_branch: &cmp_branch,
            sub_worktree: &cmp_worktree,
            goal_text: &goal_text,
            soul_text: None,
            commit_subject: &commit_subject,
        },
        git,
    )?;

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

/// Stub summary body. Identifies the dispatching conversation
/// without reading any of its diagnostic step records (§2.3 — no
/// runtime read of `request.json` / `response.json`).
fn build_summary(parent_conv_id: &str) -> String {
    format!("conversation {parent_conv_id}: terminal compaction\n")
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
