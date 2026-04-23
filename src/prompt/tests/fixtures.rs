//! Shared stubs and fixtures for `prompt::tests::*`.

use crate::prompt::{AdapterRunner, Clock, Deps, IdGen};
use crate::template::GitRunner;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Deterministic [`Clock`] — counts how many times each method was
/// called and returns a formatted counter, so `started_at` /
/// `ended_at` / ts are all distinct and observable.
pub(super) struct FixedClock {
    iso_calls: RefCell<u32>,
    compact_calls: RefCell<u32>,
}

impl FixedClock {
    pub(super) fn new() -> Self {
        Self {
            iso_calls: RefCell::new(0),
            compact_calls: RefCell::new(0),
        }
    }
}

impl Clock for FixedClock {
    fn now_iso8601(&self) -> String {
        *self.iso_calls.borrow_mut() += 1;
        format!("iso-{}", self.iso_calls.borrow())
    }
    fn now_compact(&self) -> String {
        *self.compact_calls.borrow_mut() += 1;
        format!("ct-{}", self.compact_calls.borrow())
    }
}

pub(super) struct FixedIdGen;
impl IdGen for FixedIdGen {
    fn short(&self) -> String {
        "deadbeef".into()
    }
}

/// Scripted [`AdapterRunner`] reply — either canned stdout bytes or an
/// I/O error.
pub(super) enum AdapterReply {
    Ok(Vec<u8>),
    Err(io::Error),
}

/// Snapshot of a single adapter invocation captured by [`StubAdapter`]:
/// binary name, argv, environment overrides, and stdin bytes.
pub(super) type AdapterCall = (OsString, Vec<String>, Vec<(String, String)>, Vec<u8>);

/// Canonical `describe` JSON the harness expects from the Anthropic
/// adapter. Tests that vary the shape (e.g. missing `endpoint_env`)
/// build their own bytes inline.
pub(super) const STUB_DESCRIBE_JSON: &str = r#"{
    "name":"anthropic","schema_version":2,
    "capabilities":["tool_use_native"],
    "models":["claude-sonnet-4-7"],
    "auth_env":["ANTHROPIC_API_KEY"],
    "endpoint_env":["LERNIE_PROVIDER_ANTHROPIC_ENDPOINT"]
}"#;

/// Scripted [`AdapterRunner`] — replies are taken from a FIFO queue
/// so a single test can script `describe` then `complete`
/// independently. All invocations are recorded for later assertion.
pub(super) struct StubAdapter {
    replies: RefCell<VecDeque<AdapterReply>>,
    pub(super) observed: RefCell<Vec<AdapterCall>>,
}

impl StubAdapter {
    /// Queue an explicit sequence of replies.
    pub(super) fn scripted<I>(replies: I) -> Self
    where
        I: IntoIterator<Item = AdapterReply>,
    {
        Self {
            replies: RefCell::new(replies.into_iter().collect()),
            observed: RefCell::new(Vec::new()),
        }
    }

    /// Common case: a successful `describe` followed by
    /// `complete_bytes` from the next adapter call.
    pub(super) fn happy(complete_bytes: &[u8]) -> Self {
        Self::scripted([
            AdapterReply::Ok(STUB_DESCRIBE_JSON.as_bytes().to_vec()),
            AdapterReply::Ok(complete_bytes.to_vec()),
        ])
    }

    /// Single-call error reply — fires on the first adapter invocation
    /// (which is `describe`). Used for the spawn-failure error path.
    pub(super) fn failing(kind: io::ErrorKind, msg: &str) -> Self {
        Self::scripted([AdapterReply::Err(io::Error::new(kind, msg.to_string()))])
    }

    /// Helpers for building reply variants without exposing the enum.
    pub(super) fn reply_ok(bytes: &[u8]) -> AdapterReply {
        AdapterReply::Ok(bytes.to_vec())
    }
    pub(super) fn reply_err(kind: io::ErrorKind, msg: &str) -> AdapterReply {
        AdapterReply::Err(io::Error::new(kind, msg.to_string()))
    }

    /// The most recent invocation — convenient for tests that only
    /// care about the `complete` call (the second one, when describe
    /// succeeded).
    pub(super) fn last(&self) -> AdapterCall {
        self.observed.borrow().last().cloned().expect("no calls")
    }
}

impl AdapterRunner for StubAdapter {
    fn run(
        &self,
        binary: &OsString,
        args: &[&str],
        envs: &[(&str, &str)],
        stdin_bytes: &[u8],
    ) -> io::Result<Vec<u8>> {
        self.observed.borrow_mut().push((
            binary.clone(),
            args.iter().map(|s| (*s).to_owned()).collect(),
            envs.iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            stdin_bytes.to_vec(),
        ));
        match self.replies.borrow_mut().pop_front() {
            Some(AdapterReply::Ok(b)) => Ok(b),
            Some(AdapterReply::Err(e)) => Err(e),
            None => panic!("StubAdapter::run called more times than scripted"),
        }
    }
}

/// Scripted [`GitRunner`] — records both the `-C` destination and the
/// args so tests can distinguish commands run in the repo root
/// (`worktree add`) from commands run in the branch worktree (`add`,
/// `commit`). Can be configured to fail at a specific call index.
pub(super) struct StubGit {
    pub(super) runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
    fail_at: Option<usize>,
}

impl StubGit {
    pub(super) fn ok() -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: None,
        }
    }
    pub(super) fn failing_at(idx: usize) -> Self {
        Self {
            runs: RefCell::new(Vec::new()),
            fail_at: Some(idx),
        }
    }
}

/// Canned stdout the stub returns for any `run_capture`; a single
/// reply slot is enough for v0.2 since the only capture is
/// `git rev-parse main`.
pub(super) const STUB_BASE_SHA: &str = "base-sha-cafe";

impl GitRunner for StubGit {
    fn run(&self, dest: &Path, args: &[&str]) -> io::Result<()> {
        let mut runs = self.runs.borrow_mut();
        let idx = runs.len();
        runs.push((
            dest.to_path_buf(),
            args.iter().map(|s| (*s).to_owned()).collect(),
        ));
        if self.fail_at == Some(idx) {
            Err(io::Error::other(format!("stub git fail at {idx}")))
        } else {
            Ok(())
        }
    }
    fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
        self.run(dest, args)?;
        Ok(STUB_BASE_SHA.into())
    }
}

// --- Fixtures ---------------------------------------------------------

pub(super) const VALID_PROVIDERS_YAML: &str = r#"
providers:
  anthropic:
    endpoint: https://api.anthropic.com
    auth:
      type: api_key
      env: ANTHROPIC_API_KEY
models:
  claude-sonnet-4-7:
    provider: anthropic
    model_id: claude-sonnet-4-7
    capabilities: [tool_use_native]
    context_window: 200000
"#;

pub(super) const VALID_AGENTS_YAML: &str = r#"
agents:
  worker:
    model: claude-sonnet-4-7
    system_prompt: prompts/worker.md
"#;

pub(super) const HAPPY_RESPONSE_JSON: &str = r#"{
    "id":"msg_01","model":"claude-sonnet-4-7","stop_reason":"end_turn",
    "content":[{"type":"text","text":"hi there"}],
    "usage":{"input_tokens":3,"output_tokens":2}
}"#;

pub(super) fn scaffold_repo(
    providers_yaml: &str,
    agents_yaml: &str,
    worker_prompt: Option<&str>,
) -> TempDir {
    let tmp = TempDir::new().unwrap();
    let agent = tmp.path().join(".agent");
    let prompts = agent.join("system").join("prompts");
    std::fs::create_dir_all(&prompts).unwrap();
    std::fs::write(agent.join("providers.yaml"), providers_yaml).unwrap();
    std::fs::write(agent.join("agents.yaml"), agents_yaml).unwrap();
    if let Some(body) = worker_prompt {
        std::fs::write(prompts.join("worker.md"), body).unwrap();
    }
    tmp
}

pub(super) fn valid_deps<'a>(
    adapter: &'a StubAdapter,
    git: &'a StubGit,
    clock: &'a FixedClock,
    id: &'a FixedIdGen,
) -> Deps<'a> {
    Deps {
        adapter,
        git,
        clock,
        id_gen: id,
    }
}

/// An adapter the test does not expect to reach — scripted with no
/// replies, so calling it panics and the failure is loud.
pub(super) fn unreachable_adapter() -> StubAdapter {
    StubAdapter::scripted([])
}

/// Deterministic worktree path for the standard fixtures — FixedClock
/// returns `ct-1` on the first `now_compact` call, FixedIdGen always
/// returns `deadbeef`. Tests pre-populate paths under this directory
/// to force I/O failures.
pub(super) fn worktree_path(repo: &Path) -> PathBuf {
    repo.join(".lernie/worktrees/ex/ct-1-deadbeef")
}
