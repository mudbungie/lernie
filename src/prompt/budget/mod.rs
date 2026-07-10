//! Per-conversation budget enforcement (ARCH §6 "Budgets (v0.7)").
//!
//! `workflow.yaml` declares `budgets: {max_total_tokens, max_wall_seconds,
//! max_depth}` (all optional; omitted → unbounded). The harness checks
//! them at every model-call boundary, *before* invoking the adapter
//! (`crate::prompt::dispatch::run_exchange`). Spend, wall, and depth are
//! all derived from disk at check time by [`derive`] — no running counter
//! is stored (PRINCIPLES "Single source of truth").
//!
//! **One live whole-tree check — no inheritance.** A budget is a
//! per-conversation-tree ceiling, and `steps/` is one shared tree at the
//! conv-repo root, written live by every conversation (root and every
//! subagent) and never merged (ARCH §2.2/§2.3/§2.6). So any driver — root
//! or subagent — derives the *whole tree's* live spend against the root
//! id ([`root_of`]) and checks it against the single frozen `workflow.yaml`
//! limit. Nothing is handed down at dispatch: the child reads the same
//! total the parent would, so there is no snapshot to freeze and no
//! parent-minus-child to double-count. Tokens and wall derive over the
//! whole tree; `max_depth` is positional and derives from the driver's own
//! branch name. (An optional per-subtree cap — a future `--token-cap`-style
//! knob checked against a subtree's own spend — is not built here.)
//!
//! **Exhaustion is an ordinary terminal state.** On exhaustion the
//! harness ceases the branch's step loop and writes
//! `refs/lernie/budget-exhausted/<branch>` ([`mark_exhausted`]) — the
//! same git-native marking pattern as the §2.6-step-6 conflicted ref,
//! read by `await(handle)` to surface `{status: budget_exhausted}`. No
//! new event type, no `response.json` marker — a ref plus a stop,
//! classified like any other terminal state (ARCH §6).
//!
//! **`max_depth` and the root (flagged, ARCH §6).** §6 does not spell out
//! the depth boundary. This module reads `max_depth` as the deepest
//! *allowed* dispatch depth: a conversation is exhausted iff
//! `depth(branch) > max_depth`. The root is depth 0, so it is never
//! depth-exhausted for any non-negative `max_depth`; a subagent
//! `max_depth + 1` levels below the root exhausts on its first model
//! call. See `docs/ARCHITECTURE.md` §6.

pub mod derive;
#[cfg(test)]
mod tests;

use crate::config::Budgets;
use crate::template::GitRunner;
use std::path::Path;

/// Git-native marker ref for an exhausted conversation
/// (`refs/lernie/budget-exhausted/<branch>`, ARCH §6 — mirrors the
/// §2.6-step-6 conflicted ref). The single home of the prefix; the
/// `await` built-in reads it to surface `{status: budget_exhausted}`.
pub const BUDGET_EXHAUSTED_REF_PREFIX: &str = "refs/lernie/budget-exhausted/";

/// Which declared limit a conversation crossed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Tokens,
    Wall,
    Depth,
}

impl std::fmt::Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Axis::Tokens => "max_total_tokens",
            Axis::Wall => "max_wall_seconds",
            Axis::Depth => "max_depth",
        })
    }
}

/// The crossed limit and the derived actual that crossed it. Carried for
/// the operator-facing diagnostic only — the terminal state is the ref,
/// not this value (ARCH §6 "an ordinary terminal state").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exhausted {
    pub axis: Axis,
    pub limit: u64,
    pub actual: u64,
}

impl std::fmt::Display for Exhausted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} exhausted ({}/{})",
            self.axis, self.actual, self.limit
        )
    }
}

/// Evaluate the single frozen `budgets` against spend/wall/depth derived
/// live from disk. Tokens and wall are whole-tree consumables — derived
/// over [`root_of`]`(branch)` (the branch plus its entire descent, ARCH
/// §6) and exhausted at `actual >= limit`, so the driver stops *before* it
/// overspends. Depth is positional — derived from `branch` itself and
/// exhausted at `actual > limit` (`max_depth` is the deepest allowed
/// depth; the root at depth 0 is never depth-exhausted). Returns the first
/// crossed axis, or `None` when every declared limit still has headroom;
/// an unbounded axis (`None` limit) never triggers.
pub fn check(repo: &Path, branch: &str, budgets: &Budgets) -> Option<Exhausted> {
    if let Some(limit) = budgets.max_total_tokens {
        let actual = derive::spend(repo, root_of(branch));
        if actual >= limit {
            return Some(Exhausted {
                axis: Axis::Tokens,
                limit,
                actual,
            });
        }
    }
    if let Some(limit) = budgets.max_wall_seconds {
        let actual = derive::wall_seconds(repo, root_of(branch));
        if actual >= limit {
            return Some(Exhausted {
                axis: Axis::Wall,
                limit,
                actual,
            });
        }
    }
    if let Some(limit) = budgets.max_depth {
        let actual = u64::from(derive::depth(branch));
        if actual > u64::from(limit) {
            return Some(Exhausted {
                axis: Axis::Depth,
                limit: u64::from(limit),
                actual,
            });
        }
    }
    None
}

/// The root conversation id of a branch: its first two hyphen-delimited
/// tokens (`<ts>-<short>`, ARCH §2.2). Every dispatch appends
/// `-<ts>-<short>` (hyphenated descent), so the root is the prefix before
/// the second hyphen. Whole-tree spend/wall derive against this, since
/// [`derive`] sums a branch plus its entire descent — the root's descent
/// *is* the whole tree. A bare root id (at most one hyphen) is its own root.
fn root_of(branch: &str) -> &str {
    match branch.match_indices('-').nth(1) {
        Some((idx, _)) => &branch[..idx],
        None => branch,
    }
}

/// Write the budget-exhausted marker ref for `branch` at its tip
/// (`git update-ref refs/lernie/budget-exhausted/<branch> <branch>`),
/// run inside `worktree` where `.git` resolves. State lives in git, not
/// a sidecar file (PRINCIPLES SSOT) — the same pattern as the §2.6
/// conflicted ref.
pub fn mark_exhausted(worktree: &Path, branch: &str, git: &dyn GitRunner) -> std::io::Result<()> {
    let ref_name = format!("{BUDGET_EXHAUSTED_REF_PREFIX}{branch}");
    git.run(worktree, &["update-ref", ref_name.as_str(), branch])
}
