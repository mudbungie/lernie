//! Checkpoint trigger evaluation (ARCH §2.6, §2.7, §6).
//!
//! Compaction runs at **checkpoints** during a branch's execution. The
//! triggers are declared in the governing config's `workflow.yaml`
//! `compaction:` block (§6) — `every_n_commits`, `every_t_seconds`, or
//! the agent-elected `on_flush` — and are read **at the step boundary by
//! the executor**, which already holds the loaded workflow config (§6 hop
//! step 4). A branch with no configured trigger never compacts (§2.7).
//!
//! This module is the evaluation, kept **minimal and binding-shaped** so
//! it slots into the workflow-binding interpreter (§6) rather than
//! standing as a parallel path: [`due`] is a pure predicate over the
//! config and a [`CheckpointState`] the executor derives from disk, and
//! [`state`] is that derivation. When the interpreter evaluates the
//! `compaction:` block at a boundary and the `worker_flush` event, it
//! computes the same state and asks the same predicate; today's boundary
//! hook calls them directly.
//!
//! The **checkpoint commit `C`** is the branch tip at the boundary where
//! [`due`] fires — the commit the dispatched compactor forks off (§2.6).
//! "Since the last checkpoint" is derived from git, never stored
//! (`docs/PRINCIPLES.md` Single source of truth).
//!
//! # Two invariants on eligibility
//!
//! **The clock starts at the branch's own founding commit.** A branch is
//! forked off its parent's tip and inherits the parent's whole history
//! (§2.3 *Fork and inheritance*), so "commits on this branch" can never
//! mean "commits reachable from HEAD": a seconds-old child would read its
//! parent's hundred commits as its own and be instantly due. The one
//! commit that founds a branch — and the only one naming it — is its
//! **dispatch commit**, `dispatch: <role> [<agent-id>]` for a child and
//! `step 001: dispatch [<agent-id>]` for a root
//! ([`crate::prompt::role`], [`crate::prompt::dispatch::step_commit`]).
//! Both end in `[<agent-id>]`, so one anchored pattern founds every
//! branch and the root is not a special case — it is the general path
//! ([`origin`]). A branch's checkpoint reference is therefore the newest
//! of {its dispatch commit, its last compaction merge}, and the root
//! commit only when neither exists.
//!
//! **A compactor is never compaction-eligible.** A compactor *is* the
//! compaction, not a subject of one (§2.7): compacting it would fork a
//! compactor off a compactor, whose own transcript is the compaction it
//! was dispatched to perform. The role is derived from the same founding
//! commit ([`crate::prompt::role::derive`] — the single authoritative
//! home for an agent's role), so the exclusion costs no new state.
//!
//! Either invariant alone stops the runaway cascade of bl-a9eb (yog
//! bl-ebbd); both are stated because they are different facts.

use super::Error;
use crate::config::{CompactionConfig, CompactionTrigger};
use crate::prompt::role;
use crate::template::GitRunner;
use std::path::Path;

/// Subject prefix of a compaction-merge commit ([`super::merge`]). The
/// most recent such commit marks the last checkpoint; commits after it are
/// what a fresh `every_n_commits`/`every_t_seconds` trigger measures from.
pub(super) const MERGE_SUBJECT_PREFIX: &str = "compaction merge [";

/// Branch state a checkpoint trigger is evaluated against (§6), derived
/// from disk by [`state`]. Every field is a live derivation, never a
/// stored counter (`docs/PRINCIPLES.md` Single source of truth).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointState {
    /// Commits on the branch since the last compaction merge (or branch
    /// root). Drives `every_n_commits`.
    pub commits_since_checkpoint: u32,
    /// Wall-clock seconds since the last checkpoint commit's timestamp
    /// (or the branch root's). Drives `every_t_seconds`. Saturating at 0,
    /// so a checkpoint dated in the future never reads as negative time.
    pub seconds_since_checkpoint: u64,
    /// The agent elected a flush this boundary (§2.7 "the `flush` action
    /// the agent may call"). Drives `on_flush`; the flush is the
    /// agent-elected trigger, distinct from the config-clock triggers.
    pub flush_requested: bool,
    /// This branch is itself a compactor — its role, derived from its own
    /// dispatch commit ([`crate::prompt::role::derive`]), is
    /// [`super::COMPACTOR_ROLE`]. A compactor is not a member of the
    /// compaction-eligible set at any commit count, elapsed time, or
    /// elected flush (module docs, §2.7).
    pub is_compactor: bool,
}

/// Whether a checkpoint is due this boundary (§2.6, §2.7) — the one home
/// of compaction eligibility. `None` config — no configured trigger —
/// never compacts (§2.7), and **a compactor is never eligible** whatever
/// the config says (module docs: it is the compaction, not a subject of
/// one). Otherwise the trigger kind selects the predicate; a `None`/`0`
/// `n` (guarded out at config load, §6) is never due, so a malformed
/// config fails closed rather than compacting every step.
pub fn due(cfg: Option<&CompactionConfig>, state: &CheckpointState) -> bool {
    if state.is_compactor {
        return false;
    }
    let Some(cfg) = cfg else {
        return false;
    };
    let threshold = |v: u64| {
        cfg.intermediate
            .n
            .is_some_and(|n| n > 0 && v >= u64::from(n))
    };
    match cfg.intermediate.trigger {
        CompactionTrigger::EveryNCommits => threshold(u64::from(state.commits_since_checkpoint)),
        CompactionTrigger::EveryTSeconds => threshold(state.seconds_since_checkpoint),
        CompactionTrigger::OnFlush => state.flush_requested,
    }
}

/// Derive [`CheckpointState`] for the agent `agent_id`, whose branch is
/// checked out at `worktree` (§6). `now_unix` is the current wall-clock in
/// Unix seconds, supplied by the caller so this stays a pure derivation
/// over its inputs (§6 binding-shaped); `flush_requested` is the
/// agent-elected input. The commit count and the checkpoint timestamp both
/// measure from [`origin`] — the branch's own founding commit or its last
/// compaction merge, whichever is newer — so an inherited history is never
/// counted as this branch's own (module docs).
pub fn state(
    worktree: &Path,
    agent_id: &str,
    now_unix: u64,
    flush_requested: bool,
    git: &dyn GitRunner,
) -> Result<CheckpointState, Error> {
    let last = origin(worktree, agent_id, git)?;
    Ok(CheckpointState {
        commits_since_checkpoint: commits_since(worktree, last.as_deref(), git)?,
        seconds_since_checkpoint: now_unix.saturating_sub(checkpoint_time(worktree, &last, git)?),
        flush_requested,
        is_compactor: role::derive(worktree, "HEAD", agent_id, git)?.as_deref()
            == Some(super::COMPACTOR_ROLE),
    })
}

/// Count commits on `HEAD` after `last` (exclusive), or the whole branch
/// when `last` is `None`.
fn commits_since(worktree: &Path, last: Option<&str>, git: &dyn GitRunner) -> Result<u32, Error> {
    let range = match last {
        Some(sha) => format!("{sha}..HEAD"),
        None => "HEAD".to_string(),
    };
    let out = git
        .run_capture(worktree, &["rev-list", "--count", &range])
        .map_err(|source| Error::Git {
            op: "checkpoint rev-list count",
            source,
        })?;
    Ok(out.trim().parse::<u32>().unwrap_or(0))
}

/// Committer Unix timestamp of the reference commit: the branch's
/// [`origin`] when one exists, else the branch's root commit — the point
/// elapsed time is measured from.
fn checkpoint_time(
    worktree: &Path,
    last: &Option<String>,
    git: &dyn GitRunner,
) -> Result<u64, Error> {
    let reference = match last {
        Some(sha) => sha.clone(),
        None => root_commit(worktree, git)?,
    };
    let out = git
        .run_capture(worktree, &["log", "-n", "1", "--format=%ct", &reference])
        .map_err(|source| Error::Git {
            op: "checkpoint commit time",
            source,
        })?;
    Ok(out.trim().parse::<u64>().unwrap_or(0))
}

/// The branch's root commit (its first commit, no parent). `--max-parents=0`
/// lists roots; the last line is the eldest.
fn root_commit(worktree: &Path, git: &dyn GitRunner) -> Result<String, Error> {
    let out = git
        .run_capture(worktree, &["rev-list", "--max-parents=0", "HEAD"])
        .map_err(|source| Error::Git {
            op: "checkpoint root rev-list",
            source,
        })?;
    Ok(out.lines().last().unwrap_or("").trim().to_string())
}

/// The sha the branch's checkpoint clock measures from: the newest commit
/// on `HEAD` that is either **this branch's own founding commit** (its
/// dispatch commit, whose subject ends `[<agent-id>]` for a child and a
/// root alike) or a **compaction merge**. `git log -n1` walks newest-first
/// and stops at the first match, and multiple `--grep` patterns are OR'd,
/// so one query answers "where does this branch's own clock start".
///
/// `None` — neither commit reachable — falls back to the branch root
/// ([`checkpoint_time`], [`commits_since`]). That is the general path with
/// empty inputs, not a bootstrap special case: a tree with no dispatch
/// commit at all has nothing else to measure from.
fn origin(worktree: &Path, agent_id: &str, git: &dyn GitRunner) -> Result<Option<String>, Error> {
    let founding = format!(r"\[{agent_id}\]$");
    let merged = format!("^{}", regex_escape_brackets(MERGE_SUBJECT_PREFIX));
    let out = git
        .run_capture(
            worktree,
            &[
                "log",
                "-n",
                "1",
                "--format=%H",
                "-E",
                "--grep",
                founding.as_str(),
                "--grep",
                merged.as_str(),
            ],
        )
        .map_err(|source| Error::Git {
            op: "checkpoint log grep",
            source,
        })?;
    let sha = out.trim();
    Ok((!sha.is_empty()).then(|| sha.to_string()))
}

/// Escape the one regex metacharacter a commit-subject *prefix* constant
/// can carry (`[`), so a literal prefix reads as a literal under `git log
/// -E`. Keeping both `--grep` patterns in one regex dialect is what lets
/// the two questions [`origin`] asks collapse into one git call.
fn regex_escape_brackets(literal: &str) -> String {
    literal.replace('[', r"\[")
}

#[cfg(test)]
mod tests;
