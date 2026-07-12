//! Unit tests for the harness-owned retry driver (ARCH §2.10, §4.4).

use super::*;
use crate::config::{Backoff, RetryConfig};
use crate::prompt::Error;
use crate::provider::segment::{self, Outcome};
use brazen::{CanonicalError, ContentKind, Delta, ErrorKind, Event, FinishReason, Role};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io;
use std::path::Path;

/// FIFO stub adapter: each `run` pops one scripted reply and replays its
/// NDJSON bytes line-by-line, recording the stdin it saw.
struct StubAdapter {
    replies: RefCell<VecDeque<io::Result<Vec<u8>>>>,
    stdins: RefCell<Vec<Vec<u8>>>,
}

impl StubAdapter {
    fn new(replies: Vec<io::Result<Vec<u8>>>) -> Self {
        Self {
            replies: RefCell::new(replies.into_iter().collect()),
            stdins: RefCell::new(Vec::new()),
        }
    }
}

impl AdapterRunner for StubAdapter {
    fn run(
        &self,
        _binary: &OsString,
        _args: &[&str],
        stdin_bytes: &[u8],
        on_line: &mut dyn FnMut(&[u8]) -> io::Result<()>,
    ) -> io::Result<()> {
        self.stdins.borrow_mut().push(stdin_bytes.to_vec());
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
                Ok(())
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
    let adapter = StubAdapter::new(replies);
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
    bytes
        .split(|b| *b == b'\n')
        .filter(|l| *l == br#"{"type":"end"}"#)
        .count()
}

#[test]
fn build_request_is_a_typed_canonical_request() {
    // Message pass-through is asserted in the e2e test; here we pin the
    // typed shape and the composed `tools` array (§3.3).
    let tool = brazen::Tool {
        name: "bash".into(),
        description: None,
        input_schema: serde_json::json!({"type": "object"}),
    };
    let req = build_request("claude-sonnet-4-7", "sys", vec![], vec![tool.clone()], 4096);
    assert_eq!(req.model, "claude-sonnet-4-7");
    assert_eq!(req.max_tokens, Some(4096));
    assert_eq!(req.system, Some(vec![Content::Text("sys".into())]));
    assert_eq!(req.tools, vec![tool]);
    // `stream` absent → brazen default governs; `extra` stays empty.
    assert_eq!(req.stream, None);
    assert!(req.extra.is_empty());
}

#[test]
fn single_attempt_completes_and_writes_one_segment() {
    let ((r, sleeps, stdins), bytes) = drive(
        vec![Ok(text_stream("hi", FinishReason::Stop))],
        retry(3),
        false,
    );
    r.unwrap();
    assert_eq!(sleeps, 0, "no retry, no sleep");
    assert_eq!(segment::classify(&bytes), Outcome::Complete);
    assert_eq!(stdins[0], b"{}");
}

#[test]
fn retryable_error_then_clean_writes_two_segments() {
    // §12 forced-retry criterion: a retryable 529 then a clean stream.
    let ((r, sleeps, stdins), bytes) = drive(
        vec![
            Ok(error_stream(ErrorKind::Provider { status: 529 })),
            Ok(text_stream("recovered", FinishReason::Stop)),
        ],
        retry(3),
        false,
    );
    r.unwrap();
    assert_eq!(sleeps, 1, "one backoff drove the single retry");
    assert_eq!(ends(&bytes), 2, "two attempt segments");
    assert_eq!(segment::classify(&bytes), Outcome::Complete);
    assert_eq!(stdins[0], stdins[1], "identical re-issued request");
}

#[test]
fn non_retryable_error_aborts_without_retry() {
    let ((r, sleeps, _), bytes) = drive(
        vec![Ok(error_stream(ErrorKind::Provider { status: 400 }))],
        retry(3),
        false,
    );
    assert!(matches!(r, Err(Error::AdapterError { .. })));
    assert_eq!(sleeps, 0);
    assert_eq!(segment::classify(&bytes), Outcome::Failed);
}

#[test]
fn retryable_error_exhausts_attempt_cap() {
    let ((r, sleeps, _), _) = drive(
        vec![Ok(error_stream(ErrorKind::Transport))],
        retry(1),
        false,
    );
    assert!(matches!(r, Err(Error::AdapterError { .. })));
    assert_eq!(sleeps, 0);
}

#[test]
fn half_stream_is_a_harness_error() {
    let mut bytes = line(&Event::message_start(None, None, Role::Assistant));
    bytes.extend(line(&Event::ContentDelta {
        index: 0,
        delta: Delta::TextDelta("par".into()),
    }));
    let ((r, _, _), _) = drive(vec![Ok(bytes)], retry(3), false);
    assert!(matches!(r, Err(Error::AdapterHalfStream)));
}

#[test]
fn malformed_event_line_surfaces_adapter_json_error() {
    let ((r, _, _), _) = drive(vec![Ok(b"not json\n".to_vec())], retry(3), false);
    assert!(matches!(r, Err(Error::AdapterJson(_))));
}

#[test]
fn spawn_failure_surfaces_adapter_spawn_error() {
    let ((r, _, _), _) = drive(
        vec![Err(io::Error::new(io::ErrorKind::NotFound, "no bz"))],
        retry(3),
        false,
    );
    assert!(matches!(r, Err(Error::AdapterSpawn(_))));
}

#[test]
fn adapter_override_handshake_accepts_v1() {
    let ((r, _, _), _) = drive(
        vec![Ok(text_stream("ok", FinishReason::Stop))],
        retry(3),
        true,
    );
    assert!(r.is_ok());
}

#[test]
fn adapter_override_handshake_rejects_wrong_version() {
    let mut bytes = line(&Event::MessageStart {
        v: 2,
        id: None,
        model: None,
        role: Role::Assistant,
    });
    bytes.extend(line(&Event::Finish {
        reason: FinishReason::Stop,
    }));
    bytes.extend(line(&Event::End));
    let ((r, _, _), _) = drive(vec![Ok(bytes)], retry(3), true);
    assert!(matches!(
        r,
        Err(Error::HandshakeMismatch {
            found: Some(2),
            expected: 1
        })
    ));
}

#[test]
fn response_path_that_is_a_directory_surfaces_io_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("resp");
    std::fs::create_dir_all(&path).unwrap(); // occupy the path with a dir
    let (r, _, _) = run_at(&path, vec![], retry(3), false);
    assert!(matches!(r, Err(Error::Io(_))));
}

#[test]
fn parent_creation_failure_surfaces_io_error() {
    // A regular file where a step dir is expected → create_dir_all fails.
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"x").unwrap();
    let path = blocker.join("001/response.json");
    let (r, _, _) = run_at(
        &path,
        vec![Ok(text_stream("x", FinishReason::Stop))],
        retry(3),
        false,
    );
    assert!(matches!(r, Err(Error::Io(_))));
}

#[test]
fn real_sleeper_sleeps_without_panicking() {
    RealSleeper.sleep(Duration::ZERO);
}
