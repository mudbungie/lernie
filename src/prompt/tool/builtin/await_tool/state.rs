//! Terminal-state derivation for the `await` built-in.
//!
//! Single source of truth (`docs/PRINCIPLES.md`): the four return
//! variants are derived by reading git refs and the conv-repo's
//! filesystem at poll time. No sidecar handle index, no in-memory
//! state.
//!
//! - [`State::Merged`]: the subagent's tip is reachable from the
//!   parent (`merge-base(handle,parent) == HEAD(handle)`) — ARCH §2.6
//!   merge-back has landed. The terminal compacted summary is read
//!   from the latest `summary/<NNN>.md` on the subagent's tip.
//! - [`State::Conflicted`]: the merge protocol wrote
//!   `refs/lernie/conflicted/<handle>` on rebase failure (ARCH §2.6
//!   step 6 — harness defect surface).
//! - [`State::Stopped`]: two on-disk signatures, both surfaced through
//!   the same variant (ARCH §2.9 — kill, crash, and explicit stop are
//!   indistinguishable on disk):
//!     1. The latest step's `response.json` last segment carries an
//!        `Error` ([`Outcome::Failed`] — provider error, retry budget
//!        exhausted, or compactor abort).
//!     2. The latest step's `response.json` has no trailing terminal
//!        line ([`Outcome::NoTerminal`]) AND no writer process holds it
//!        open (kill-mid-stream — harness died with the fd open, kernel
//!        closed it on exit). Detected by reusing the §2.9 / ARCH
//!        line-267 [`PgidFinder`] /proc-fd scan that backs `lernie
//!        stop`'s pid discovery — same source of truth that the §3.5
//!        in_flight classification reads.
//! - [`State::InFlight`]: none of the above; caller polls again.
//!
//! The `response.json` framing is read through
//! [`crate::provider::segment`], the single seam that accepts both the
//! legacy v0.3 and brazen `v=1` vocabularies (§4.4) — so `await`
//! survives the v0.6 writer swap without a change here.

use std::fs;
use std::path::{Path, PathBuf};

use super::Error;
use crate::prompt::stop::PgidFinder;
use crate::provider::segment::{self, Outcome};
use crate::template::GitRunner;

/// Subdir of the conv-repo holding step records (ARCH §2.2 / §2.3).
pub const STEPS_DIR: &str = "steps";
/// Per-step JSONL of §4.4 stream events (ARCH §2.3, §4.4).
pub const RESPONSE_FILE: &str = "response.json";
/// Branch-relative directory holding compaction summaries (ARCH §2.7).
pub const SUMMARY_DIR: &str = "summary";
/// Ref-namespace prefix for the merge protocol's conflicted marker
/// (ARCH §2.6 step 6). The full ref is
/// `refs/lernie/conflicted/<sub-branch>` and points at the sub-branch's
/// pre-rebase tip.
pub const CONFLICTED_REF_PREFIX: &str = "refs/lernie/conflicted/";

/// Result of one [`check`] poll. The non-terminal variant
/// ([`Self::InFlight`]) tells [`super::run`] to sleep and retry.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum State {
    Merged(String),
    Stopped,
    Conflicted,
    InFlight,
}

/// Terminal subset of [`State`] returned by the poll loop. Splitting
/// the type so the post-loop code is total-match (no `InFlight` arm
/// to handle) keeps the run path linear and tarpaulin-trackable.
#[derive(Debug)]
pub(super) enum Terminal {
    Merged(String),
    Stopped,
    Conflicted,
}

impl Terminal {
    pub(super) fn as_output(&self) -> super::Output<'_> {
        match self {
            Self::Merged(s) => super::Output::Merged {
                summary: s.as_str(),
            },
            Self::Stopped => super::Output::Stopped,
            Self::Conflicted => super::Output::Conflicted,
        }
    }
}

/// One poll: probe git + the conv-repo's filesystem for any of the
/// three terminal signatures. Order matters only insofar as
/// `Conflicted` is checked before `Merged` — a conflict is the merge
/// protocol's surfaced failure, and once written it stays there until
/// the operator clears it.
pub(super) fn check(
    repo: &Path,
    git_dir: &Path,
    parent: &str,
    handle: &str,
    git: &dyn GitRunner,
    writer_finder: &dyn PgidFinder,
) -> Result<State, Error> {
    if conflicted_ref_exists(git_dir, handle, git)? {
        return Ok(State::Conflicted);
    }
    if is_merged(git_dir, parent, handle, git)? {
        let summary = latest_summary(git_dir, handle, git)?;
        return Ok(State::Merged(summary));
    }
    match examine_latest_response(repo, handle)? {
        // Last segment carries an `Error` (§4.4 *failed*): the adapter
        // or harness recorded a typed failure. Stopped on the spot.
        Some((Outcome::Failed, _)) => Ok(State::Stopped),
        // Clean terminal (§4.4 *complete*): the model finished a step.
        // The harness still has terminal compaction + merge-back to do,
        // so the chain may still progress — keep polling.
        Some((Outcome::Complete, _)) => Ok(State::InFlight),
        // No file to scan — either no step dir yet, or the latest step
        // has not produced a `response.json`. Nothing to disambiguate
        // against /proc; keep polling.
        None => Ok(State::InFlight),
        // Bytes on disk but no trailing terminal (§4.4 *no terminal*) —
        // disambiguate mid-stream from kill-mid-stream via the §3.5
        // writer probe.
        Some((Outcome::NoTerminal, path)) => {
            if writer_finder
                .find_writer_pgid(&path)
                .map_err(|source| Error::Git {
                    op: "scan /proc for response.json writer",
                    source,
                })?
                .is_none()
            {
                Ok(State::Stopped)
            } else {
                Ok(State::InFlight)
            }
        }
    }
}

/// `git for-each-ref` succeeds with empty stdout when the ref does
/// not exist and prints the ref line when it does — so a non-empty
/// capture is the existence test.
fn conflicted_ref_exists(git_dir: &Path, handle: &str, git: &dyn GitRunner) -> Result<bool, Error> {
    let ref_name = format!("{CONFLICTED_REF_PREFIX}{handle}");
    let out = git
        .run_capture(git_dir, &["for-each-ref", &ref_name])
        .map_err(|source| Error::Git {
            op: "for-each-ref",
            source,
        })?;
    Ok(!out.trim().is_empty())
}

/// Merged check: `merge-base(handle, parent) == rev-parse(handle)`.
/// `git merge-base --is-ancestor` would be terser, but its non-zero
/// "not an ancestor" exit looks identical to a real failure under the
/// `run/run_capture` shape — the equality check stays inside ordinary
/// success-only output and reads as the ref query it actually is.
fn is_merged(
    git_dir: &Path,
    parent: &str,
    handle: &str,
    git: &dyn GitRunner,
) -> Result<bool, Error> {
    let handle_tip = git
        .run_capture(git_dir, &["rev-parse", handle])
        .map_err(|source| Error::Git {
            op: "rev-parse handle",
            source,
        })?;
    let base = git
        .run_capture(git_dir, &["merge-base", handle, parent])
        .map_err(|source| Error::Git {
            op: "merge-base",
            source,
        })?;
    Ok(handle_tip.trim() == base.trim() && !handle_tip.trim().is_empty())
}

/// Read the highest-numbered `summary/<NNN>.md` from `handle`'s tip.
/// `git ls-tree` lists tree entries; we filter to `summary/*.md`,
/// pick the lexicographically greatest (zero-padded NNN sorts
/// numerically), and `git show` its blob.
fn latest_summary(git_dir: &Path, handle: &str, git: &dyn GitRunner) -> Result<String, Error> {
    let listing = git
        .run_capture(
            git_dir,
            &["ls-tree", "-r", "--name-only", handle, "--", SUMMARY_DIR],
        )
        .map_err(|source| Error::Git {
            op: "ls-tree summary",
            source,
        })?;
    let path = listing
        .lines()
        .filter(|p| p.starts_with(&format!("{SUMMARY_DIR}/")) && p.ends_with(".md"))
        .max_by(|a, b| a.cmp(b))
        .ok_or(Error::MergedWithoutSummary)?;
    let spec = format!("{handle}:{path}");
    git.run_capture(git_dir, &["show", &spec])
        .map_err(|source| Error::Git {
            op: "show summary",
            source,
        })
}

/// Locate the latest step's `response.json` and classify its last
/// attempt segment via [`crate::provider::segment`] (both vocabularies,
/// §4.4). `None` means "no file to scan" (no step dir yet, or no
/// `response.json` written) — the *absent* case. `Some((outcome,
/// path))` bundles the classification with the path so the caller can
/// hand a [`Outcome::NoTerminal`] straight to the [`PgidFinder`] without
/// recomputing it (single source of truth — one path per poll).
fn examine_latest_response(repo: &Path, handle: &str) -> Result<Option<(Outcome, PathBuf)>, Error> {
    let conv_steps = repo.join(STEPS_DIR).join(handle);
    let Some(latest) = latest_step_dir(&conv_steps) else {
        return Ok(None);
    };
    let path = latest.join(RESPONSE_FILE);
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(Error::Git {
                op: "read response.json",
                source,
            });
        }
    };
    Ok(Some((segment::classify(&bytes), path)))
}

/// Walk `conv_steps` for numeric subdirs and return the highest. The
/// step-seq width is fixed at 3 (ARCH §2.3), so lexicographic
/// comparison sorts numerically.
fn latest_step_dir(conv_steps: &Path) -> Option<std::path::PathBuf> {
    let mut entries: Vec<_> = fs::read_dir(conv_steps).ok()?.flatten().collect();
    entries.retain(|e| {
        e.file_name()
            .to_str()
            .map(|s| s.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false)
    });
    entries.sort_by_key(|e| e.file_name());
    entries.last().map(|e| e.path())
}
