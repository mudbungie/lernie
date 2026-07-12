//! The work-product transfer at delivery (ARCH §2.6).
//!
//! A **result message** (§2.6) carries a `terminal_ref:` — the sha of the
//! child's branch tip at return. When such a message is delivered, the
//! parent's executor applies the child's work-product diff as one commit
//! *immediately before* the delivery commit (§2.6): the diff from the
//! child's fork point (its dispatch commit's parent, derived from
//! ancestry as `merge-base(parent, terminal_ref)`) to the terminal sha,
//! **filtered to work products** — the child's branch-scoped context
//! paths (`goal.md`, `soul.md`, `messages/**`, `summary/**`, `skills/**`)
//! are excluded, because a child's context must never contaminate its
//! parent's tree (§2.1, §2.6).
//!
//! Conflict-free is by construction: sibling write paths are disjoint and
//! children edit work products from the parent's own fork point (§2.5),
//! so the apply is clean unless the write-path guarantee was violated — a
//! harness defect, not a normal arm. A diff that fails to apply is
//! **declined loudly** (§2.6): the result message still delivers (the
//! terminal ref preserves every byte on the child's branch), and the
//! failure is marked git-natively at `refs/lernie/conflicted/<agent-id>`
//! for operator attention — the same marking pattern as budget
//! exhaustion (§6). The decline is not an error the delivery propagates;
//! it is a recorded outcome.

use crate::prompt::Error;
use crate::template::GitRunner;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Ref namespace for the declined-transfer mark (§2.6). The merge
/// protocol that first wrote this ref is deleted; the namespace survives
/// as the decline signal (§2.6 "the ref namespace itself survives").
const CONFLICTED_REF_PREFIX: &str = "refs/lernie/conflicted/";

/// Branch-scoped context paths excluded from the transfer (§2.6) — a
/// child's context, which must never reach its parent's tree. Expressed
/// as git exclude pathspecs so `git diff` filters them out in one pass.
const CONTEXT_EXCLUDES: &[&str] = &[
    ":(exclude)goal.md",
    ":(exclude)soul.md",
    ":(exclude)messages",
    ":(exclude)summary",
    ":(exclude)skills",
];

/// Parse `terminal_ref:` out of a deposited message's frontmatter, or
/// `None` when the message carries none (an ordinary steering message,
/// not a result message — §2.6, §2.11). Only the leading frontmatter
/// block (between the first two `---` lines) is scanned, so a body line
/// that happens to read `terminal_ref: …` is never mistaken for the
/// field.
pub(super) fn terminal_ref_of(body: &str) -> Option<String> {
    let mut lines = body.lines();
    if lines.next() != Some("---") {
        return None;
    }
    for line in lines {
        if line == "---" {
            return None;
        }
        if let Some(value) = line.strip_prefix("terminal_ref:") {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Apply the work-product transfer for a result message from `child_id`
/// naming `terminal_ref`, onto the parent branch checked out at
/// `worktree` (its branch is `parent_branch`). Lands the filtered diff as
/// one commit (§2.6); a clean empty diff commits nothing (the general
/// path with empty inputs), and an apply failure is declined by marking
/// `refs/lernie/conflicted/<child_id>` and returning `Ok` — the delivery
/// still proceeds.
pub(super) fn apply(
    worktree: &Path,
    parent_branch: &str,
    child_id: &str,
    terminal_ref: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let fork = git
        .run_capture(worktree, &["merge-base", parent_branch, terminal_ref])
        .map_err(|source| Error::Git {
            op: "transfer merge-base",
            source,
        })?;

    let patch = patch_path(child_id);
    let patch_str = patch.to_string_lossy().into_owned();
    let output_arg = format!("--output={patch_str}");
    let mut diff_args = vec!["diff", &fork, terminal_ref, &output_arg, "--"];
    diff_args.extend_from_slice(CONTEXT_EXCLUDES);
    git.run(worktree, &diff_args).map_err(|source| Error::Git {
        op: "transfer diff",
        source,
    })?;

    // A patch that is absent or zero-length means the child changed no
    // work products — nothing to transfer, so no commit (§2.6 empty diff).
    let empty = std::fs::metadata(&patch)
        .map(|m| m.len() == 0)
        .unwrap_or(true);
    if empty {
        let _ = std::fs::remove_file(&patch);
        return Ok(());
    }

    // Conflict-free by construction (§2.6); an apply failure is the
    // harness-defect signal, declined loudly rather than propagated.
    let outcome = match git.run(worktree, &["apply", "--index", &patch_str]) {
        Ok(()) => commit_transfer(worktree, child_id, git),
        Err(_) => decline(worktree, child_id, terminal_ref, git),
    };
    let _ = std::fs::remove_file(&patch);
    outcome
}

/// A unique patch path outside every worktree (so it is never swept into
/// a commit), keyed by `child_id` and a nanosecond stamp.
fn patch_path(child_id: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("lernie-transfer-{child_id}-{nanos}.patch"))
}

/// Commit the applied (already index-staged) work-product diff as one
/// commit on the parent branch (§2.6 "lands as one commit").
fn commit_transfer(worktree: &Path, child_id: &str, git: &dyn GitRunner) -> Result<(), Error> {
    let msg = format!("work-product transfer [{child_id}]");
    git.run(worktree, &["commit", "-m", msg.as_str()])
        .map_err(|source| Error::Git {
            op: "transfer commit",
            source,
        })
}

/// Decline the transfer (§2.6): mark `refs/lernie/conflicted/<child_id>`
/// at the child's terminal sha (preserving every byte of its work) for
/// operator attention. The result message still delivers.
fn decline(
    worktree: &Path,
    child_id: &str,
    terminal_ref: &str,
    git: &dyn GitRunner,
) -> Result<(), Error> {
    let conflicted_ref = format!("{CONFLICTED_REF_PREFIX}{child_id}");
    git.run(
        worktree,
        &["update-ref", conflicted_ref.as_str(), terminal_ref],
    )
    .map_err(|source| Error::Git {
        op: "transfer decline update-ref",
        source,
    })
}

#[cfg(test)]
mod tests;
