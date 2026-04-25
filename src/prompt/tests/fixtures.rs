//! Shared stubs and fixtures for `prompt::tests::*`.

pub(super) use super::tool_stub::StubToolExecutor;

use crate::prompt::{AdapterRunner, Clock, Deps, Dispatcher, IdGen};
use crate::template::GitRunner;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Monotonic `iso-N`/`ct-N` [`Clock`].
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

/// Canned [`AdapterRunner`] reply.
pub(super) enum AdapterReply {
    Ok(Vec<u8>),
    Err(io::Error),
}

/// One adapter invocation: (binary, argv, envs, stdin).
pub(super) type AdapterCall = (OsString, Vec<String>, Vec<(String, String)>, Vec<u8>);

/// Canonical `describe` JSON. Tests varying the shape build inline.
pub(super) const STUB_DESCRIBE_JSON: &str = r#"{
    "name":"anthropic","schema_version":2,
    "capabilities":["tool_use_native"],
    "models":["claude-sonnet-4-7"],
    "auth_env":["ANTHROPIC_API_KEY"],
    "endpoint_env":["LERNIE_PROVIDER_ANTHROPIC_ENDPOINT"]
}"#;

/// FIFO-replying [`AdapterRunner`] with a recording log.
pub(super) struct StubAdapter {
    replies: RefCell<VecDeque<AdapterReply>>,
    pub(super) observed: RefCell<Vec<AdapterCall>>,
}

impl StubAdapter {
    pub(super) fn scripted<I>(replies: I) -> Self
    where
        I: IntoIterator<Item = AdapterReply>,
    {
        Self {
            replies: RefCell::new(replies.into_iter().collect()),
            observed: RefCell::new(Vec::new()),
        }
    }

    /// `describe` ok then `complete_bytes` on the next call.
    pub(super) fn happy(complete_bytes: &[u8]) -> Self {
        Self::scripted([
            AdapterReply::Ok(STUB_DESCRIBE_JSON.as_bytes().to_vec()),
            AdapterReply::Ok(complete_bytes.to_vec()),
        ])
    }

    /// One-shot error reply (fires on the `describe` call).
    pub(super) fn failing(kind: io::ErrorKind, msg: &str) -> Self {
        Self::scripted([AdapterReply::Err(io::Error::new(kind, msg.to_string()))])
    }

    pub(super) fn reply_ok(bytes: &[u8]) -> AdapterReply {
        AdapterReply::Ok(bytes.to_vec())
    }
    pub(super) fn reply_err(kind: io::ErrorKind, msg: &str) -> AdapterReply {
        AdapterReply::Err(io::Error::new(kind, msg.to_string()))
    }

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

/// Recording [`GitRunner`] with optional `fail_at` index.
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
    fn run_capture(&self, dest: &Path, args: &[&str]) -> io::Result<String> {
        // Empty return: alignment rm produces no staged delta in stub
        // tests, so neither conditional checkout nor alignment commit fires.
        self.run(dest, args)?;
        Ok(String::new())
    }
}

/// Global `<harness-root>/providers.yaml` (ARCH §4.1).
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

/// Per-repo `<conv-repo>/providers.yaml` (ARCH §4.3).
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

/// Lay out a v0.3 conv repo (§2.2): per-repo `providers.yaml` and
/// optional `souls/worker.md`.
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
/// `providers.yaml`.
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
    tool_executor: &'a StubToolExecutor,
    harness_root: &'a Path,
) -> Deps<'a> {
    Deps {
        adapter,
        git,
        clock,
        id_gen: id,
        dispatcher,
        tool_executor,
        harness_root,
    }
}

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

pub(super) fn unreachable_adapter() -> StubAdapter {
    StubAdapter::scripted([])
}

/// Drive [`crate::prompt::run`] with default stubs.
pub(super) fn run_with_stubs(
    repo: &Path,
    msg: &str,
    adapter: &StubAdapter,
    git: &StubGit,
) -> Result<String, crate::prompt::Error> {
    let harness = scaffold_harness_root();
    let (clock, id, dispatcher) = (FixedClock::default(), FixedIdGen, StubDispatcher::ok());
    let tool_executor = StubToolExecutor::ok();
    crate::prompt::run(
        repo,
        msg,
        &valid_deps(
            adapter,
            git,
            &clock,
            &id,
            &dispatcher,
            &tool_executor,
            harness.path(),
        ),
    )
}

/// Conv worktree path for the standard fixtures (FixedClock=ct-1,
/// FixedIdGen=deadbeef → `<repo>/ct-1-deadbeef/`, §2.2).
pub(super) fn worktree_path(repo: &Path) -> PathBuf {
    repo.join("ct-1-deadbeef")
}
