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
//!     1. The latest step's `response.json` ended in a §4.4 `error`
//!        event (clean failure — provider error or compactor abort).
//!     2. The latest step's `response.json` has no `message_stop` and
//!        no `error` line, AND no writer process holds it open
//!        (kill-mid-stream — harness died with the fd open, kernel
//!        closed it on exit). Detected by reusing the §2.9 / ARCH
//!        line-267 [`PgidFinder`] /proc-fd scan that backs `lernie
//!        stop`'s pid discovery — same source of truth that the §3.5
//!        in_flight classification reads.
//! - [`State::InFlight`]: none of the above; caller polls again.

use std::fs;
use std::path::{Path, PathBuf};

use super::Error;
use crate::prompt::stop::PgidFinder;
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
        // §4.4 `error` line: the adapter or harness recorded a typed
        // failure before exiting cleanly. Stopped on the spot.
        Examined::Error => Ok(State::Stopped),
        // §4.4 `message_stop`: model finished a step cleanly. Harness
        // still has terminal compaction + merge-back to do, so the
        // chain may still progress — keep polling.
        Examined::MessageStop => Ok(State::InFlight),
        // No file to scan — either no step dir yet, or the latest
        // step has not produced a `response.json`. Nothing to
        // disambiguate against /proc; keep polling.
        Examined::Absent => Ok(State::InFlight),
        // File on disk with bytes but no terminal line — disambiguate
        // mid-stream from kill-mid-stream via the §3.5 writer probe.
        Examined::NonTerminal(path) => {
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

/// Outcome of one `response.json` examination — drives the [`check`]
/// match arms. Mirrors the UI's `git_tree::state` classifier shape
/// (ARCH §3.5, §7.1) but bundles the path with the kill-mid-stream
/// variant so the writer-probe arm is total: there is no
/// `NonTerminal`-without-path state in the type.
#[derive(Debug)]
enum Examined {
    /// No latest step yet, or the step has no `response.json` on
    /// disk. Nothing to probe.
    Absent,
    /// Last completed line is a §4.4 `message_stop` — model step
    /// finished cleanly; harness may still be advancing.
    MessageStop,
    /// Last completed line is a §4.4 `error` — typed failure recorded
    /// on disk. Always Stopped.
    Error,
    /// File exists with bytes but no terminal `message_stop`/`error`
    /// line — disambiguate mid-stream from kill-mid-stream by probing
    /// the bundled path against [`PgidFinder`].
    NonTerminal(PathBuf),
}

/// Locate the latest step's `response.json` and classify its
/// terminal-line state. Bundles the path with `NonTerminal` so the
/// caller can hand it to the [`PgidFinder`] without re-resolving the
/// path (single source of truth — one path computation per poll).
fn examine_latest_response(repo: &Path, handle: &str) -> Result<Examined, Error> {
    let conv_steps = repo.join(STEPS_DIR).join(handle);
    let Some(latest) = latest_step_dir(&conv_steps) else {
        return Ok(Examined::Absent);
    };
    let path = latest.join(RESPONSE_FILE);
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Examined::Absent),
        Err(source) => {
            return Err(Error::Git {
                op: "read response.json",
                source,
            });
        }
    };
    Ok(classify_last_line(&bytes, path))
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

/// Last fully-terminated JSONL line classification. Mirrors the UI's
/// `git_tree::state::has_terminal_event` mid-write tolerance — a
/// trailing partial line (no `\n` yet) is dropped; only the most
/// recent completed line is examined. Malformed JSON or unknown
/// event types both fall through to `NonTerminal` so the kill-mid-
/// stream probe can run; a writer crashing while emitting garbage is
/// still a kill. Consumes `path` so the `NonTerminal` variant carries
/// it forward to the writer probe.
fn classify_last_line(bytes: &[u8], path: PathBuf) -> Examined {
    let terminated = match bytes.iter().rposition(|&b| b == b'\n') {
        Some(idx) => &bytes[..=idx],
        None => return Examined::NonTerminal(path),
    };
    let Some(line) = terminated
        .split(|&b| b == b'\n')
        .rfind(|line| !line.is_empty())
    else {
        return Examined::NonTerminal(path);
    };
    let Ok(value): Result<serde_json::Value, _> = serde_json::from_slice(line) else {
        return Examined::NonTerminal(path);
    };
    match value.get("type").and_then(|v| v.as_str()) {
        Some("error") => Examined::Error,
        Some("message_stop") => Examined::MessageStop,
        _ => Examined::NonTerminal(path),
    }
}
