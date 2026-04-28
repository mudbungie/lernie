//! Recording stubs for `prompt::run`'s injected dependencies.
//!
//! Lives alongside [`super::fixtures`] but split out so the latter
//! stays under the repo's per-file line cap.

use crate::prompt::{AdapterRunner, Dispatcher};
use crate::template::GitRunner;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

/// Canned [`AdapterRunner`] reply. `Ok` carries raw stdout bytes
/// (with `\n` terminators) that the stub replays into the per-line
/// callback. `Err` short-circuits before any callback fires.
pub(super) enum AdapterReply {
    Ok(Vec<u8>),
    Err(io::Error),
}

/// One adapter invocation: (binary, argv, envs, stdin).
pub(super) type AdapterCall = (OsString, Vec<String>, Vec<(String, String)>, Vec<u8>);

/// Canonical `describe` JSON. Tests varying the shape build inline.
pub(super) const STUB_DESCRIBE_JSON: &str = r#"{
    "name":"anthropic","schema_version":2,
    "capabilities":["tool_use_native","streaming"],
    "models":["claude-sonnet-4-7"],
    "auth_env":["ANTHROPIC_API_KEY"],
    "endpoint_env":["LERNIE_PROVIDER_ANTHROPIC_ENDPOINT"]
}"#;

/// FIFO-replying [`AdapterRunner`] with a recording log. Each scripted
/// reply's bytes are split on `\n` and replayed through the runner's
/// callback (the §4.4 wire shape: one event per line).
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

    /// `describe` ok then `complete_bytes` (JSONL) on the next call.
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
        on_line: &mut dyn FnMut(&[u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        self.observed.borrow_mut().push((
            binary.clone(),
            args.iter().map(|s| (*s).to_owned()).collect(),
            envs.iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
            stdin_bytes.to_vec(),
        ));
        let bytes = match self.replies.borrow_mut().pop_front() {
            Some(AdapterReply::Ok(b)) => b,
            Some(AdapterReply::Err(e)) => return Err(e),
            None => panic!("StubAdapter::run called more times than scripted"),
        };
        for line in bytes.split(|b| *b == b'\n') {
            if line.is_empty() {
                continue;
            }
            on_line(line)?;
        }
        Ok(())
    }
}

pub(super) fn unreachable_adapter() -> StubAdapter {
    StubAdapter::scripted([])
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

/// One observed [`Dispatcher::dispatch`] invocation: `(role, repo,
/// branch, goal)`. Aliased so the `RefCell` field type stays under
/// clippy's complexity ceiling.
pub(super) type DispatchCall = (String, PathBuf, String, Option<String>);

/// Recording [`Dispatcher`] for the dispatch handoff. Captures both
/// the compactor handoff (no goal, role=`compactor`) and the worker
/// handoff (`--goal …`, role=`worker`) without needing two stubs.
#[derive(Default)]
pub(super) struct StubDispatcher {
    pub(super) calls: RefCell<Vec<DispatchCall>>,
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
    fn dispatch(
        &self,
        role: &str,
        repo: &Path,
        branch: &str,
        goal: Option<&str>,
    ) -> io::Result<()> {
        let entry = (
            role.to_owned(),
            repo.to_path_buf(),
            branch.to_owned(),
            goal.map(str::to_owned),
        );
        self.calls.borrow_mut().push(entry);
        match &self.fail {
            None => Ok(()),
            Some(e) => Err(io::Error::new(e.kind(), e.to_string())),
        }
    }
}
