//! Compaction (ARCH §2.6, §2.7) — the model-driven compactor, its
//! checkpoint triggers, and the compaction merge.
//!
//! A **compactor** is an *ordinary child agent* (§2.7): it is dispatched
//! like any other child ([`super::child_dispatch`] with the `compactor`
//! role), runs its own step loop with a real model call, and returns by
//! depositing a result message (§2.6). Nothing about it is privileged
//! except two things this module owns:
//!
//! - its **toolset** ([`tools`]) — the fixed pair `write_summary` /
//!   `mark_for_deletion`, built into the primitive and available to the
//!   compactor role alone, making "deletion-only" structural (§2.7);
//! - how its output **lands** ([`merge`]) — the compaction merge, the one
//!   merge left in the system now that merge-back is gone (§2.6).
//!
//! [`checkpoint`] is the trigger evaluation: the executor reads it at a
//! step boundary and, when a checkpoint is due, dispatches a compactor off
//! the branch tip (the checkpoint commit `C`, §2.6). When the compactor
//! returns with a *final-response* epitaph, the executor lands the merge;
//! any other epitaph lands **no merge** and the branch continues
//! uncompacted (§2.7). Both the boundary trigger read and the return-time
//! merge are the same step-boundary seam the workflow-binding interpreter
//! drives (§6) — [`checkpoint::due`] and [`merge::merge`] are the
//! binding-shaped procedures it invokes.
//!
//! **There is no terminal-compaction stage** (§2.7): a child's result
//! message carries its own terminal response (§2.6), not a compactor
//! product, and with merge-back gone there is no merge payload to slim
//! before returning. The v0.3 terminal compactor — a stub dispatched at
//! every final response — is deleted; compaction now happens only at
//! configured checkpoints during a branch's life.

pub mod checkpoint;
pub mod merge;
pub mod tools;

pub use checkpoint::{CheckpointState, due, state};
pub use merge::{MergeOutcome, merge};

use super::Error;

/// Role name of the compactor child (ARCH §2.7). Its soul is
/// `souls/compactor.md` in the governing config commit, and its toolset is
/// the built-in [`tools`] pair — not a `providers.yaml` `tools:` list.
pub const COMPACTOR_ROLE: &str = "compactor";

/// Boilerplate goal handed to a compactor at dispatch (ARCH §2.7). The
/// dispatching branch name interpolates so the compactor knows which
/// branch it is compacting; its inherited worktree (forked off the
/// checkpoint commit) carries that branch's transcript, summaries, and
/// work products — the worktree invariant *is* its view (§2.7, §5.1).
pub fn compactor_goal(parent_branch: &str) -> String {
    format!(
        "You are the compactor for branch `{parent_branch}`.\n\
         \n\
         Read the branch's transcript, prior summaries under `summary/`, and\n\
         work products, and produce a signal-preserving, minimal view of the\n\
         branch's history using the `write_summary` tool. The harness writes it\n\
         to the next `summary/<NNN>.md` on this branch.\n\
         \n\
         Use `mark_for_deletion` to nominate superseded files — stale transcript\n\
         entries under `messages/`, a prior `summary/` you are replacing, spent\n\
         `skills/` bodies. Your toolset is deletion-only: you can remove and\n\
         summarize, never rewrite, so the worst case is lost information, never\n\
         corrupted information. A work product the live branch has rewritten\n\
         since you forked is kept regardless of your nomination (live-branch-wins).\n\
         \n\
         Decide relevance against the branch's goal at `goal.md`.\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compactor_goal_names_the_branch_and_both_tools() {
        let g = compactor_goal("20260101-p1");
        assert!(g.contains("`20260101-p1`"), "{g}");
        assert!(g.contains("write_summary"));
        assert!(g.contains("mark_for_deletion"));
        assert!(g.contains("goal.md"));
        assert!(g.contains("summary/<NNN>.md"));
        assert!(g.contains("live-branch-wins"));
    }

    #[test]
    fn compactor_role_is_the_soul_key() {
        assert_eq!(COMPACTOR_ROLE, "compactor");
    }
}
