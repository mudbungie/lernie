//! Per-conversation budget enforcement (ARCH §6 "Budgets (v0.7)").
//!
//! `workflow.yaml` declares `budgets: {max_total_tokens, max_wall_seconds,
//! max_depth}` (all optional; omitted → unbounded). The harness checks
//! them at every model-call boundary, *before* invoking the adapter
//! (`crate::prompt::dispatch::run_exchange`). Spend, wall, and depth are
//! all derived from disk at check time by [`derive`] — no running counter
//! is stored (PRINCIPLES "Single source of truth").
//!
//! **Clamped inheritance.** A dispatch hands a child the minimum of the
//! parent's remaining budget and the child's own declaration
//! ([`remaining`] + [`clamp`], ARCH §6). Because `workflow.yaml` is one
//! frozen copy per conversation tree, "child declaration" and "parent
//! declaration" are the same limits, so the clamp resolves to the
//! parent's remaining headroom — a hand-off that cannot be re-derived
//! once the child starts spending (the parent's subtree spend then
//! includes the child's). The root conversation has no parent, so its
//! effective budget *is* its declaration, evaluated over the whole tree
//! (branch + descent).
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

/// Evaluate `effective` (a conversation's clamped limits) against
/// spend/wall/depth derived from disk for `branch` and its descent.
/// Returns the first crossed axis, or `None` when every declared limit
/// still has headroom. An unbounded axis (`None` limit) never triggers.
///
/// Tokens and wall are consumables — exhausted at `actual >= limit`, so
/// the conversation stops *before* it overspends. Depth is positional —
/// `max_depth` is the deepest allowed depth, so it exhausts at
/// `actual > limit` (module doc: the root interaction).
pub fn check(repo: &Path, branch: &str, effective: &Budgets) -> Option<Exhausted> {
    if let Some(limit) = effective.max_total_tokens {
        let actual = derive::spend(repo, branch);
        if actual >= limit {
            return Some(Exhausted {
                axis: Axis::Tokens,
                limit,
                actual,
            });
        }
    }
    if let Some(limit) = effective.max_wall_seconds {
        let actual = derive::wall_seconds(repo, branch);
        if actual >= limit {
            return Some(Exhausted {
                axis: Axis::Wall,
                limit,
                actual,
            });
        }
    }
    if let Some(limit) = effective.max_depth {
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

/// The parent's leftover budget, for clamping a child's declaration
/// against (ARCH §6 clamped inheritance). Tokens and wall deplete by the
/// parent's derived spend (saturating, so a parent at/over its cap hands
/// zero, never an underflow); depth is a shared absolute ceiling and
/// passes through unchanged.
pub fn remaining(repo: &Path, parent_branch: &str, parent: &Budgets) -> Budgets {
    Budgets {
        max_total_tokens: parent
            .max_total_tokens
            .map(|m| m.saturating_sub(derive::spend(repo, parent_branch))),
        max_wall_seconds: parent
            .max_wall_seconds
            .map(|m| m.saturating_sub(derive::wall_seconds(repo, parent_branch))),
        max_depth: parent.max_depth,
    }
}

/// Clamp a child's own declaration to the parent's remaining budget:
/// `min` per axis (ARCH §6 "hands the child min(parent remaining, child
/// declaration)"). An axis bounded on only one side takes that side's
/// limit; unbounded on both stays unbounded.
pub fn clamp(parent_remaining: &Budgets, child_declared: &Budgets) -> Budgets {
    Budgets {
        max_total_tokens: min_opt(
            parent_remaining.max_total_tokens,
            child_declared.max_total_tokens,
        ),
        max_wall_seconds: min_opt(
            parent_remaining.max_wall_seconds,
            child_declared.max_wall_seconds,
        ),
        max_depth: min_opt(parent_remaining.max_depth, child_declared.max_depth),
    }
}

fn min_opt<T: Ord>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, b) => b,
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
