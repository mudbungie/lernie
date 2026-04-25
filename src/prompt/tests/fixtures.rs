//! Shared stubs and fixtures for `prompt::tests::*`.

use crate::prompt::{AdapterRunner, Clock, Deps, Dispatcher, IdGen};
use crate::template::GitRunner;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Deterministic [`Clock`] — returns `iso-N`/`ct-N` counters so each
/// `started_at` / `ended_at` / ts is distinct and observable.
#[derive(Default)]
pub(super) struct FixedClock {
    iso_calls: RefCell<u32>,
    compact_calls: RefCell<u32>,
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

/// Scripted [`AdapterRunner`] reply: canned stdout bytes or an I/O
/// error.
pub(super) enum AdapterReply {
    Ok(Vec<u8>),
    Err(io::Error),
}

/// Snapshot of one adapter invocation: (binary, argv, envs, stdin).
pub(super) type AdapterCall = (OsString, Vec<String>, Vec<(String, String)>, Vec<u8>);

/// Canonical `describe` JSON. Tests varying the shape build inline.
pub(super) const STUB_DESCRIBE_JSON: &str = r#"{
    "name":"anthropic","schema_version":2,
    "capabilities":["tool_use_native"],
    "models":["claude-sonnet-4-7"],
    "auth_env":["ANTHROPIC_API_KEY"],
    "endpoint_env":["LERNIE_PROVIDER_ANTHROPIC_ENDPOINT"]
}"#;

/// Scripted [`AdapterRunner`] — replies pop from a FIFO queue (so a
/// test can script `describe` then `complete` independently). All
/// invocations are recorded.
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

    /// Successful `describe` then `complete_bytes` on the next call.
    pub(super) fn happy(complete_bytes: &[u8]) -> Self {
        Self::scripted([
            AdapterReply::Ok(STUB_DESCRIBE_JSON.as_bytes().to_vec()),
            AdapterReply::Ok(complete_bytes.to_vec()),
        ])
    }

    /// Single-call error reply — fires on the `describe` call.
    pub(super) fn failing(kind: io::ErrorKind, msg: &str) -> Self {
        Self::scripted([AdapterReply::Err(io::Error::new(kind, msg.to_string()))])
    }

    pub(super) fn reply_ok(bytes: &[u8]) -> AdapterReply {
        AdapterReply::Ok(bytes.to_vec())
    }
    pub(super) fn reply_err(kind: io::ErrorKind, msg: &str) -> AdapterReply {
        AdapterReply::Err(io::Error::new(kind, msg.to_string()))
    }

    /// Most recent invocation — for tests that only inspect `complete`.
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

/// Scripted [`GitRunner`] — records (dest, args) so tests can tell
/// which dir the command ran in. Optional fail_at index for error
/// paths.
#[derive(Default)]
pub(super) struct StubGit {
    pub(super) runs: RefCell<Vec<(PathBuf, Vec<String>)>>,
    fail_at: Option<usize>,
}

impl StubGit {
    pub(super) fn ok() -> Self {
        Self::default()
    }
    pub(super) fn failing_at(idx: usize) -> Self {
        Self {
            fail_at: Some(idx),
            ..Self::default()
        }
    }
}

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
    fn run_capture(&self, _dest: &Path, _args: &[&str]) -> io::Result<String> {
        // Single-source-of-truth: branch queries go through `git
        // branch --list`, not harness-side captures.
        unreachable!("StubGit::run_capture is not used by `prompt::dispatch`")
    }
}

// --- Fixtures ---------------------------------------------------------

/// Global `<harness-root>/providers.yaml` — endpoints, auth, model
/// capabilities (ARCH §4.1). Per the v0.3 layout, the per-repo file
/// only carries the role assignments.
pub(super) const VALID_GLOBAL_PROVIDERS_YAML: &str = r#"
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

/// Per-repo `<conv-repo>/providers.yaml` — `roles:` section only
/// (ARCH §4.3). Endpoint and model capabilities live in the global
/// file.
pub(super) const VALID_PER_REPO_PROVIDERS_YAML: &str = r#"
roles:
  worker:
    provider: anthropic
    model: claude-sonnet-4-7
"#;

pub(super) const HAPPY_RESPONSE_JSON: &str = r#"{
    "id":"msg_01","model":"claude-sonnet-4-7","stop_reason":"end_turn",
    "content":[{"type":"text","text":"hi there"}],
    "usage":{"input_tokens":3,"output_tokens":2}
}"#;

/// Lay out a v0.3 conversation repo (ARCH §2.2): per-repo
/// `providers.yaml` at the conv-repo root, optional `souls/worker.md`
/// for the worker role's system prompt (§4.3). Returns the holding
/// TempDir; `tmp.path()` is the conv-repo root passed to `prompt::run`.
pub(super) fn scaffold_repo(per_repo_yaml: &str, worker_soul: Option<&str>) -> TempDir {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("providers.yaml"), per_repo_yaml).unwrap();
    if let Some(body) = worker_soul {
        let souls = tmp.path().join("souls");
        std::fs::create_dir_all(&souls).unwrap();
        std::fs::write(souls.join("worker.md"), body).unwrap();
    }
    tmp
}

/// Lay out a temp harness root (ARCH §2.2) with a global
/// `providers.yaml`. `prompt::run` reads it via `Deps::harness_root`;
/// production passes [`crate::harness_root::resolve`]'s output.
pub(super) fn scaffold_harness_root() -> TempDir {
    let tmp = TempDir::new().unwrap();
    std::fs::write(
        tmp.path().join("providers.yaml"),
        VALID_GLOBAL_PROVIDERS_YAML,
    )
    .unwrap();
    tmp
}

pub(super) fn valid_deps<'a>(
    adapter: &'a StubAdapter,
    git: &'a StubGit,
    clock: &'a FixedClock,
    id: &'a FixedIdGen,
    dispatcher: &'a StubDispatcher,
    harness_root: &'a Path,
) -> Deps<'a> {
    Deps {
        adapter,
        git,
        clock,
        id_gen: id,
        dispatcher,
        harness_root,
    }
}

/// Recording [`Dispatcher`] — captures every dispatch call as
/// `(repo, branch)` so prompt-level tests can assert the compactor
/// was dispatched without paying for a subprocess. `fail` is the
/// optional error returned after recording.
#[derive(Default)]
pub(super) struct StubDispatcher {
    pub(super) calls: RefCell<Vec<(PathBuf, String)>>,
    fail: Option<io::Error>,
}

impl StubDispatcher {
    pub(super) fn ok() -> Self {
        Self::default()
    }
    pub(super) fn failing(kind: io::ErrorKind, msg: &str) -> Self {
        Self {
            fail: Some(io::Error::new(kind, msg.to_string())),
            ..Self::default()
        }
    }
}

impl Dispatcher for StubDispatcher {
    fn dispatch_compactor(&self, repo: &Path, branch: &str) -> io::Result<()> {
        let entry = (repo.to_path_buf(), branch.to_owned());
        self.calls.borrow_mut().push(entry);
        match &self.fail {
            None => Ok(()),
            Some(e) => Err(io::Error::new(e.kind(), e.to_string())),
        }
    }
}

/// An adapter the test does not expect to reach.
pub(super) fn unreachable_adapter() -> StubAdapter {
    StubAdapter::scripted([])
}

/// Drive [`crate::prompt::run`] with default stubs for clock, id,
/// dispatcher, and a fresh harness root scaffolded with the canonical
/// global providers.yaml. Tests that need a non-ok dispatcher or a
/// custom harness root build [`Deps`] inline instead.
pub(super) fn run_with_stubs(
    repo: &Path,
    msg: &str,
    adapter: &StubAdapter,
    git: &StubGit,
) -> Result<String, crate::prompt::Error> {
    let harness = scaffold_harness_root();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    crate::prompt::run(
        repo,
        msg,
        &valid_deps(adapter, git, &clock, &id, &dispatcher, harness.path()),
    )
}

/// Deterministic worktree path for the standard fixtures — FixedClock
/// returns `ct-1` on the first `now_compact` call, FixedIdGen always
/// returns `deadbeef`. Worktree dir = bare conv-id, sibling of
/// `root/` (ARCH §2.2). Tests pre-populate paths under this directory
/// to force I/O failures.
pub(super) fn worktree_path(repo: &Path) -> PathBuf {
    repo.join("ct-1-deadbeef")
}
