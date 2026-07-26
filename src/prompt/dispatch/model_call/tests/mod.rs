//! Unit tests for the harness-owned retry driver (ARCH §2.10, §4.4).
//!
//! This module is the scaffolding — stubs, stream builders, and the
//! `drive` harness; the cases live beside it.

mod cases;
mod stderr;

use super::*;
use crate::config::workflow::{Backoff, RetryConfig};
use crate::prompt::Error;
use crate::provider::segment::{self, Outcome};
use brazen::{CanonicalError, ContentKind, Delta, ErrorKind, Event, FinishReason, Role};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::Path;

/// FIFO stub adapter: each `run` pops one scripted reply and replays its
/// NDJSON bytes line-by-line, recording the stdin it saw. `stderrs` is
/// the parallel stderr script — an exhausted queue yields the ordinary
/// empty capture.
struct StubAdapter {
    replies: RefCell<VecDeque<io::Result<Vec<u8>>>>,
    stderrs: RefCell<VecDeque<Vec<u8>>>,
    stdins: RefCell<Vec<Vec<u8>>>,
}

impl StubAdapter {
    fn new(replies: Vec<io::Result<Vec<u8>>>) -> Self {
        Self {
            replies: RefCell::new(replies.into_iter().collect()),
            stderrs: RefCell::new(VecDeque::new()),
            stdins: RefCell::new(Vec::new()),
        }
    }

    /// Same, with a per-attempt stderr capture scripted alongside.
    fn with_stderr(replies: Vec<io::Result<Vec<u8>>>, stderrs: Vec<Vec<u8>>) -> Self {
        let stub = Self::new(replies);
        *stub.stderrs.borrow_mut() = stderrs.into_iter().collect();
        stub
    }
}

impl AdapterRunner for StubAdapter {
    fn run(
        &self,
        _binary: &OsString,
        _args: &[&str],
        stdin_bytes: &[u8],
        on_line: &mut dyn FnMut(&[u8]) -> io::Result<()>,
    ) -> io::Result<Vec<u8>> {
        self.stdins.borrow_mut().push(stdin_bytes.to_vec());
        let stderr = self.stderrs.borrow_mut().pop_front().unwrap_or_default();
        match self
            .replies
            .borrow_mut()
            .pop_front()
            .expect("scripted reply")
        {
            Ok(bytes) => {
                for l in bytes.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
                    on_line(l)?;
                }
                Ok(stderr)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Default)]
struct RecSleeper(RefCell<Vec<Duration>>);
impl Sleeper for RecSleeper {
    fn sleep(&self, dur: Duration) {
        self.0.borrow_mut().push(dur);
    }
}

fn line(e: &Event) -> Vec<u8> {
    let mut v = serde_json::to_vec(e).unwrap();
    v.push(b'\n');
    v
}

fn text_stream(text: &str, reason: FinishReason) -> Vec<u8> {
    let mut out = line(&Event::message_start(None, None, Role::Assistant));
    out.extend(line(&Event::ContentStart {
        index: 0,
        kind: ContentKind::Text {},
    }));
    out.extend(line(&Event::ContentDelta {
        index: 0,
        delta: Delta::TextDelta(text.into()),
    }));
    out.extend(line(&Event::Finish { reason }));
    out.extend(line(&Event::End));
    out
}

fn error_stream(kind: ErrorKind) -> Vec<u8> {
    let mut out = line(&Event::message_start(None, None, Role::Assistant));
    out.extend(line(&Event::Error(CanonicalError {
        kind,
        message: "boom".into(),
        provider_detail: None,
        retry_after_seconds: None,
    })));
    out.extend(line(&Event::End));
    out
}

fn retry(max: u32) -> RetryConfig {
    RetryConfig {
        max_attempts: max,
        backoff: Backoff::Exponential,
    }
}

/// Run outcome (`()` on success — content lives in the staging entry,
/// §2.3), backoff-sleep count, and per-attempt stdin bytes.
type Driven = (Result<(), Error>, usize, Vec<Vec<u8>>);

fn run_at(path: &Path, replies: Vec<io::Result<Vec<u8>>>, retry: RetryConfig, hs: bool) -> Driven {
    run_with(path, StubAdapter::new(replies), retry, hs)
}

fn run_with(path: &Path, adapter: StubAdapter, retry: RetryConfig, hs: bool) -> Driven {
    let sleeper = RecSleeper::default();
    let bin = OsString::from("bz");
    let call = ModelCall {
        adapter: &adapter,
        sleeper: &sleeper,
        binary: &bin,
        provider_row: "test",
        retry,
        expect_handshake: hs,
    };
    let result = super::run(&call, b"{}", path);
    let sleeps = sleeper.0.borrow().len();
    (result, sleeps, adapter.stdins.into_inner())
}

/// Run against a fresh tempdir; returns the driven result + response bytes.
fn drive(replies: Vec<io::Result<Vec<u8>>>, retry: RetryConfig, hs: bool) -> (Driven, Vec<u8>) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("steps/c/001/response.json");
    let driven = run_at(&path, replies, retry, hs);
    let bytes = std::fs::read(&path).unwrap_or_default();
    (driven, bytes)
}

fn ends(bytes: &[u8]) -> usize {
    let is_end = |l: &&[u8]| *l == br#"{"type":"end"}"#;
    bytes.split(|b| *b == b'\n').filter(is_end).count()
}
