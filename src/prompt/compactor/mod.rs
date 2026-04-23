//! Terminal-compaction dispatch (ARCH §2.7, v0.2 scope per §12).
//!
//! The v1 compactor is a subagent dispatched off a branch's tip with
//! a goal instructing it to summarize the branch's work. Its toolset
//! is in [`tools`]: `write_summary` and `mark_for_deletion`. v0.2
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
//! `exchange <id>: <assistant-response text>`, and commits it on the
//! compactor branch.

pub mod tools;

use super::merge::rebase_and_merge;
use super::step::{RESPONSE_FILE, StepResponse, step_dir_rel};
use super::{AGENT_DIR, Clock, Error, IdGen};
use crate::template::GitRunner;
use std::path::Path;
use tools::write_summary;

/// Prefix for invocation branches (ARCH §2.3). Exchange branches use
/// `ex/`; every other branch class (compactor, verifier, future
/// subagents) uses this. The compactor is the only caller in v0.2.
const INVOCATION_BRANCH_PREFIX: &str = "inv";
/// Subdirectory inside `.lernie/worktrees/` for invocation branch
/// worktrees. Mirrors the branch-name prefix so branch name and
/// on-disk path line up without a rewrite step.
const WORKTREES_DIR: &str = ".lernie/worktrees";
/// v0.2 has exactly one step per exchange (§12). The compactor reads
/// that one step's response to build its summary.
const TERMINAL_STEP_SEQ: u32 = 1;

/// Shape of the terminal-compaction dispatch. Kept as a struct so the
/// CLI entry point (`lernie dispatch compactor`) and the in-process
/// caller build the same request the same way — §3.4 calls out that
/// the two paths share a command surface, not just code.
pub struct CompactorRequest<'a> {
    /// Repo root. Used for `git worktree add` / `worktree remove` and
    /// for resolving the dispatching branch's worktree path.
    pub repo: &'a Path,
    /// The dispatching branch's name (e.g. `ex/<ts>-<short-id>`). The
    /// compactor spawns off this branch's tip and merges back into it.
    pub parent_branch: &'a str,
    /// Path to the dispatching branch's worktree. The stub reads
    /// `response.json` from this worktree (which is where it was just
    /// committed). The final `--no-ff` merge runs here too, so the
    /// parent branch's ref advances to the merge commit.
    pub parent_worktree: &'a Path,
    /// The exchange id — the part of the branch name after `ex/`.
    /// Used to locate `exchanges/<id>/steps/001/response.json` on the
    /// parent's tree.
    pub exchange_id: &'a str,
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
    let cmp_branch = format!("{INVOCATION_BRANCH_PREFIX}/{}/{cmp_id}", req.exchange_id);
    let cmp_worktree_rel = format!(
        "{WORKTREES_DIR}/{INVOCATION_BRANCH_PREFIX}/{}/{cmp_id}",
        req.exchange_id
    );
    let cmp_worktree = req.repo.join(cmp_worktree_rel);

    // Read the terminal response off the parent worktree (where it
    // was just committed). The compactor's input is the parent tip's
    // tree; in v0.3+ that input will be delivered through the
    // dispatch contract rather than a direct filesystem read.
    let summary = build_summary(req.parent_worktree, req.exchange_id)?;

    spawn_compactor_branch(req.repo, &cmp_worktree, &cmp_branch, req.parent_branch, git)?;

    // Dispatch commit (§2.10): goal.md lands before any model call.
    // v0.2 has no model call, but the shape is the load-bearing
    // part — v0.3+ inherits the same dispatch surface for the real
    // compactor agent.
    write_goal(&cmp_worktree, req.parent_branch)?;
    commit_goal(&cmp_worktree, req.exchange_id, git)?;

    let summary_rel = write_summary(&cmp_worktree, &summary)?;
    commit_summary(&cmp_worktree, &summary_rel, req.exchange_id, git)?;

    rebase_and_merge(
        req.repo,
        req.parent_branch,
        req.parent_worktree,
        &cmp_worktree,
        &cmp_branch,
        git,
    )?;

    Ok(())
}

/// Boilerplate goal handed to the compactor at dispatch time. The
/// branch name interpolates so the compactor knows which branch it is
/// summarizing without a separate context handoff. v0.2 stub does not
/// read the file (no model call); the text is here so v0.3+ inherits
/// the dispatch shape unchanged.
pub(crate) fn compactor_goal(parent_branch: &str) -> String {
    format!(
        "You are the terminal compactor for branch `{parent_branch}`.\n\
         \n\
         Read the branch's work and produce a signal-preserving summary using the\n\
         `write_summary` tool. The harness writes it to the next\n\
         `.agent/compactions/<NNN>.md` on this branch.\n\
         \n\
         Use `mark_for_deletion` to nominate superseded files (e.g. raw step dirs)\n\
         for removal. Do not nominate the previous summary; the harness deletes it\n\
         automatically before merging back.\n\
         \n\
         Decide relevance against the parent branch's goal at `.agent/goal.md`.\n"
    )
}

/// Write `.agent/goal.md` to the compactor's worktree. Mirrors the
/// exchange-side `write_snapshot`'s goal write (§2.8) — every
/// non-root branch carries a goal.
fn write_goal(cmp_worktree: &Path, parent_branch: &str) -> Result<(), Error> {
    let agent_dir = cmp_worktree.join(AGENT_DIR);
    std::fs::create_dir_all(&agent_dir)?;
    std::fs::write(agent_dir.join("goal.md"), compactor_goal(parent_branch))?;
    Ok(())
}

/// `git add` the goal then `git commit` the dispatch snapshot. Names
/// the exchange in the message so history reads `compaction: dispatch
/// [ex <id>]` analogously to the exchange's snapshot commit.
fn commit_goal(cmp_worktree: &Path, exchange_id: &str, git: &dyn GitRunner) -> Result<(), Error> {
    let goal_rel = format!("{AGENT_DIR}/goal.md");
    git.run(cmp_worktree, &["add", goal_rel.as_str()])
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("compaction: dispatch [ex {exchange_id}]");
    git.run(cmp_worktree, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}

/// Build the stub summary body from the dispatching branch's terminal
/// response.
fn build_summary(parent_worktree: &Path, exchange_id: &str) -> Result<String, Error> {
    let step_rel = step_dir_rel(exchange_id, TERMINAL_STEP_SEQ);
    let response_path = parent_worktree.join(step_rel).join(RESPONSE_FILE);
    let bytes = std::fs::read(&response_path)?;
    let response: StepResponse = serde_json::from_slice(&bytes).map_err(Error::AdapterJson)?;
    Ok(format!(
        "exchange {exchange_id}: {}\n",
        response.assistant_response
    ))
}

fn spawn_compactor_branch(
    repo: &Path,
    cmp_worktree: &Path,
    cmp_branch: &str,
    parent_branch: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let wt_str = cmp_worktree.to_string_lossy().to_string();
    git.run(
        repo,
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
    exchange_id: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    git.run(cmp_worktree, &["add", summary_rel])
        .map_err(|source| Error::Git { op: "add", source })?;
    let msg = format!("compaction: terminal summary [ex {exchange_id}]");
    git.run(cmp_worktree, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "commit",
            source,
        })
}

#[cfg(test)]
mod tests;
