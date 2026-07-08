//! Recording stubs for `prompt::run`'s injected dependencies.
//!
//! Lives alongside [`super::fixtures`] but split out so the latter
//! stays under the repo's per-file line cap.

use crate::prompt::{AdapterRunner, BRAZEN_PIN, Dispatcher, Sleeper};
use crate::template::GitRunner;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Canned [`AdapterRunner`] reply. `Ok` carries raw stdout bytes
/// (with `\n` terminators) that the stub replays into the per-line
/// callback. `Err` short-circuits before any callback fires.
pub(super) enum AdapterReply {
    Ok(Vec<u8>),
    Err(io::Error),
}

/// One adapter invocation: (binary, argv, stdin).
pub(super) type AdapterCall = (OsString, Vec<String>, Vec<u8>);

/// The bytes `bz --version` prints under the pin the harness expects
/// (the load-time version guard, §4.4).
pub(super) fn version_line() -> Vec<u8> {
    format!("bz {BRAZEN_PIN}\n").into_bytes()
}

/// FIFO-replying [`AdapterRunner`] with a recording log. Each scripted
/// reply's bytes are split on `\n` and replayed through the callback
/// (the §4.4 wire shape: one event per line). Under the default
/// (no `adapter:` override) resolution the harness first runs the
/// version guard (`bz --version`), so a `run`-level script leads with
/// [`version_line`]; adapter-override tests skip that.
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

    /// Version guard reply then one model-call stream (the default
    /// no-override happy path).
    pub(super) fn happy(model_stream: &[u8]) -> Self {
        Self::scripted([
            AdapterReply::Ok(version_line()),
            AdapterReply::Ok(model_stream.to_vec()),
        ])
    }

    pub(super) fn reply_ok(bytes: &[u8]) -> AdapterReply {
        AdapterReply::Ok(bytes.to_vec())
    }
    pub(super) fn reply_err(kind: io::ErrorKind, msg: &str) -> AdapterReply {
        AdapterReply::Err(io::Error::new(kind, msg.to_string()))
    }
}

impl AdapterRunner for StubAdapter {
    fn run(
        &self,
        binary: &OsString,
        args: &[&str],
        stdin_bytes: &[u8],
        on_line: &mut dyn FnMut(&[u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        self.observed.borrow_mut().push((
            binary.clone(),
            args.iter().map(|s| (*s).to_owned()).collect(),
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

/// No-op [`Sleeper`]: the retry loop's backoff sleeps are elided in
/// tests (the retry *logic* does not depend on wall time). Records the
/// requested durations so a test can assert a backoff was scheduled.
#[derive(Default)]
pub(super) struct StubSleeper {
    pub(super) slept: RefCell<Vec<Duration>>,
}

impl Sleeper for StubSleeper {
    fn sleep(&self, dur: Duration) {
        self.slept.borrow_mut().push(dur);
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

/// One observed [`Dispatcher::dispatch`] invocation: `(role, repo,
/// branch, goal)`.
pub(super) type DispatchCall = (String, PathBuf, String, Option<String>);

/// Recording [`Dispatcher`] for the dispatch handoff.
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
