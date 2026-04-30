//! `await` built-in (ARCH §2.5, §3.3 — v0.4 Phase 3).
//!
//! Stdin is the `tool_use.input` block as JSON: `{ "handle": <string> }`.
//! The conversation context (which conv-repo, which calling branch)
//! arrives via the `LERNIE_CONV_REPO` and `LERNIE_CONV_BRANCH` env vars
//! the executor sets per ARCH §3.3 — same convention as `dispatch`.
//!
//! Blocks until the named subagent reaches a terminal state and emits
//! one of three `tool_result` payloads:
//!
//! - `{"status":"merged","summary":"<text>"}` — the subagent's branch
//!   is reachable from the calling branch (`merge-base(handle,parent) ==
//!   HEAD(handle)`); `summary` is the latest `summary/<NNN>.md` on the
//!   subagent's tip (the terminal compactor's output, §2.7).
//! - `{"status":"stopped"}` — surfaces both on-disk stop signatures
//!   (ARCH §2.9, §3.5): the latest step's `response.json` ended in a
//!   §4.4 `error` event, OR the file has no terminal event line and
//!   no process holds its fd open (kill-mid-stream — kernel closed
//!   the harness's fds on exit). The kill case reuses the
//!   [`PgidFinder`] /proc-fd scan that backs `lernie stop`'s pid
//!   discovery (ARCH line 267 — same source of truth as the §3.5
//!   `in_flight` classification).
//! - `{"status":"conflicted"}` — the merge protocol wrote a
//!   `refs/lernie/conflicted/<handle>` ref on rebase failure (§2.6
//!   step 6).
//!
//! Single source of truth: every state read is via git refs or the
//! conversation-repo's filesystem (`docs/PRINCIPLES.md`). No sidecar
//! handle index, no in-memory state.

use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

use crate::prompt::stop::PgidFinder;
use crate::template::{GitRunner, ROOT_WORKTREE};

mod state;
#[cfg(test)]
mod tests;

pub use state::{CONFLICTED_REF_PREFIX, RESPONSE_FILE, STEPS_DIR, SUMMARY_DIR};

/// How long the loop waits between [`state::check`] polls when the
/// subagent is still in flight. Production cadence; tests inject a
/// no-op [`Sleeper`] so the value is never exercised during `cargo
/// test`.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Wire shape of the input. `serde(deny_unknown_fields)` so a
/// malformed `tool_use.input` surfaces as [`Error::InvalidJson`]
/// rather than silently dropping fields.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    handle: String,
}

/// Wire shape of the output. The `tag = "status"` discriminator places
/// the variant name in the `status` field — matching SKILL.md's three
/// shapes — and `summary` only appears on the merged variant.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "lowercase")]
enum Output<'a> {
    #[serde(rename = "merged")]
    Merged { summary: &'a str },
    #[serde(rename = "stopped")]
    Stopped,
    #[serde(rename = "conflicted")]
    Conflicted,
}

/// Every way [`run`] can fail. Per ARCH §3.3, stderr is concatenated
/// after stdout into `tool_result.content` when exit is non-zero, so
/// the model sees the failure verbatim.
#[derive(Debug, Error)]
pub enum Error {
    /// Stdin handed back bytes that did not parse as the documented
    /// `{handle}` shape — wrong type, missing field, extra field.
    #[error("invalid input JSON: {0}")]
    InvalidJson(#[source] serde_json::Error),
    /// The harness's stdin pipe failed mid-read.
    #[error("read input from stdin: {0}")]
    StdinRead(#[source] io::Error),
    /// Required env var (`LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH`)
    /// not set. Production callers always set these; the variant
    /// exists so a hand-invoked `lernie tool await` outside a step
    /// gets a clear message.
    #[error("missing env var {0:?} (set by the harness per ARCH §3.3)")]
    MissingEnv(&'static str),
    /// `handle` is not a descendant of the calling branch (per ARCH
    /// §2.3, a sub-branch's name is `<parent>-<sub-id>`). Awaiting an
    /// unrelated branch is rejected to keep the dispatch/await pair
    /// scoped to this conversation.
    #[error("handle {handle:?} is not a subagent of {parent:?}")]
    NotADescendant { handle: String, parent: String },
    /// A `git` invocation failed — wrapped with the operation name so
    /// the model gets a precise hint. Mirrors `prompt::Error::Git`'s
    /// shape.
    #[error("git {op}: {source}")]
    Git {
        op: &'static str,
        #[source]
        source: io::Error,
    },
    /// Subagent merged into the parent but no `summary/<NNN>.md` is
    /// readable on its tip. Indicates a compactor regression: the
    /// terminal compactor was supposed to write one (§2.7).
    #[error("subagent merged but no summary/<NNN>.md is on its branch")]
    MergedWithoutSummary,
    /// Writing the JSON output to stdout failed.
    #[error("write to stdout: {0}")]
    Write(#[source] io::Error),
}

/// Trait for env-var lookup. Production reads `std::env::var`; tests
/// inject a fixed map so the conv-repo / conv-branch values are not
/// dependent on global process state. Mirrors the dispatch tool's
/// stub-friendly env-lookup pattern.
pub trait EnvLookup {
    fn get(&self, key: &str) -> Option<OsString>;
}

/// Production [`EnvLookup`] — reads the live process environment.
pub struct ProcessEnv;

impl EnvLookup for ProcessEnv {
    fn get(&self, key: &str) -> Option<OsString> {
        std::env::var_os(key)
    }
}

/// Trait for the in-flight wait between polls. Production wires
/// [`ThreadSleeper`]; tests inject a no-op so [`run`] never actually
/// sleeps during `cargo test` (and so loop coverage is deterministic).
pub trait Sleeper {
    fn sleep(&self, dur: Duration);
}

/// Production [`Sleeper`] — `std::thread::sleep`.
pub struct ThreadSleeper;
impl Sleeper for ThreadSleeper {
    fn sleep(&self, dur: Duration) {
        std::thread::sleep(dur);
    }
}

/// Pure entry point: parse stdin, validate, poll for terminal state,
/// write the JSON outcome to `stdout`. The `lernie tool await` shim
/// wires this to the live process's stdio plus [`ProcessEnv`] +
/// [`ThreadSleeper`] + a real [`GitRunner`] + a real [`PgidFinder`]
/// (`crate::prompt::stop::ProcFsFinder` against `/proc`).
pub fn run<R: Read, W: Write>(
    stdin: &mut R,
    stdout: &mut W,
    env: &dyn EnvLookup,
    git: &dyn GitRunner,
    writer_finder: &dyn PgidFinder,
    sleeper: &dyn Sleeper,
) -> Result<(), Error> {
    let mut buf = Vec::new();
    stdin.read_to_end(&mut buf).map_err(Error::StdinRead)?;
    let input: Input = serde_json::from_slice(&buf).map_err(Error::InvalidJson)?;

    let repo_os = require_env(env, super::super::ENV_CONV_REPO)?;
    let parent_os = require_env(env, super::super::ENV_CONV_BRANCH)?;
    let repo = PathBuf::from(repo_os);
    let parent = parent_os
        .into_string()
        .map_err(|_| Error::MissingEnv(super::super::ENV_CONV_BRANCH))?;

    validate_descent(&input.handle, &parent)?;
    let git_dir = repo.join(ROOT_WORKTREE);

    #[rustfmt::skip]
    let terminal = poll_until_terminal(&repo, &git_dir, &parent, &input.handle, git, writer_finder, sleeper)?;
    write_payload(stdout, terminal.as_output())
}

/// Poll loop: returns the first non-`InFlight` state. Each spin
/// hands the [`Sleeper`] one [`POLL_INTERVAL`]; tests inject a
/// stateful sleeper that mutates the conv-repo so a later poll
/// resolves without burning real wallclock time.
fn poll_until_terminal(
    repo: &std::path::Path,
    git_dir: &std::path::Path,
    parent: &str,
    handle: &str,
    git: &dyn GitRunner,
    writer_finder: &dyn PgidFinder,
    sleeper: &dyn Sleeper,
) -> Result<state::Terminal, Error> {
    loop {
        match state::check(repo, git_dir, parent, handle, git, writer_finder)? {
            state::State::Merged(s) => return Ok(state::Terminal::Merged(s)),
            state::State::Stopped => return Ok(state::Terminal::Stopped),
            state::State::Conflicted => return Ok(state::Terminal::Conflicted),
            state::State::InFlight => sleeper.sleep(POLL_INTERVAL),
        }
    }
}

fn require_env(env: &dyn EnvLookup, key: &'static str) -> Result<OsString, Error> {
    env.get(key).ok_or(Error::MissingEnv(key))
}

/// `handle` must be a hyphenated descent off `parent` (ARCH §2.3 — a
/// subagent of `p1` is `p1-<sub>`). Anything else means the model
/// passed a foreign branch, which we reject.
fn validate_descent(handle: &str, parent: &str) -> Result<(), Error> {
    let prefix = format!("{parent}-");
    if !handle.starts_with(&prefix) || handle.len() <= prefix.len() {
        return Err(Error::NotADescendant {
            handle: handle.to_string(),
            parent: parent.to_string(),
        });
    }
    Ok(())
}

fn write_payload<W: Write>(stdout: &mut W, payload: Output<'_>) -> Result<(), Error> {
    let bytes = serde_json::to_vec(&payload).expect("Output is always serializable");
    stdout.write_all(&bytes).map_err(Error::Write)
}
