//! SIGTERM-related cases. The adapter binary's signal handler does
//! its own write+`_exit(0)` from the handler frame (see `main.rs`); the
//! library-level [`super::super::run`]/[`super::super::drain`] also
//! check the shared stop flag so unit tests can prove the
//! short-circuit without sending a real signal.

use super::*;
use std::sync::atomic::Ordering;

#[test]
fn run_short_circuits_when_stop_is_already_set() {
    let server = MockServer::start();
    mock_sse(&server, HAPPY_SSE);
    let stop = AtomicBool::new(true);
    let events = run_against(&server, &stop);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["type"], "error");
    assert_eq!(events[0]["kind"], "retryable");
    assert!(events[0]["message"].as_str().unwrap().contains("SIGTERM"));
}

/// Iterator that flips `stop` to true after yielding `n` events. Used to
/// exercise the mid-iteration SIGTERM checkpoint without racing a real
/// network stream.
struct FlipStopAfter<'a, I> {
    inner: I,
    stop: &'a AtomicBool,
    seen: usize,
    flip_after: usize,
}

impl<'a, I: Iterator<Item = Result<crate::client::streaming::Event, crate::client::Error>>>
    Iterator for FlipStopAfter<'a, I>
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next();
        if item.is_some() {
            self.seen += 1;
            if self.seen == self.flip_after {
                self.stop.store(true, Ordering::Release);
            }
        }
        item
    }
}

#[test]
fn drain_emits_interrupted_when_stop_flips_mid_iteration() {
    use crate::client::streaming::EventStream;
    use std::io::Cursor;

    let stop = AtomicBool::new(false);
    let inner = EventStream::new(Cursor::new(HAPPY_SSE.as_bytes()));
    let events = FlipStopAfter {
        inner,
        stop: &stop,
        seen: 0,
        flip_after: 2, // first event lands, then check on the next iteration fires
    };
    let mut out = Vec::new();
    crate::adapter::streaming::drain(&mut out, events, &stop).unwrap();
    let parsed = parse_jsonl(&out);
    assert_eq!(parsed.first().unwrap()["type"], "message_start");
    assert_eq!(parsed.last().unwrap()["type"], "error");
    assert!(parsed.last().unwrap()["message"].as_str().unwrap().contains("SIGTERM"));
}

#[test]
fn drain_observes_stop_before_iter_error_classifies_as_interrupted() {
    // When SIGTERM lands at the same instant the underlying stream
    // surfaces an Sse error, the early stop check at the head of the
    // next iteration must classify the situation as `interrupted`
    // rather than as a provider fault. (Mirror of the without-stop
    // case in tests/errors.rs.)
    use crate::client::Error;
    use crate::client::streaming::Event;

    let stop = AtomicBool::new(false);
    let stop_ref = &stop;
    let events = vec![
        Ok::<Event, Error>(Event::MessageStart {
            message: serde_json::json!({"id":"x","model":"m","usage":{"input_tokens":1,"output_tokens":0}}),
        }),
        Err::<Event, Error>(Error::Sse("read error: Interrupted".into())),
    ]
    .into_iter()
    .inspect(move |item| {
        if item.is_err() {
            stop_ref.store(true, Ordering::Release);
        }
    });
    let mut out = Vec::new();
    crate::adapter::streaming::drain(&mut out, events, &stop).unwrap();
    let parsed = parse_jsonl(&out);
    assert_eq!(parsed.first().unwrap()["type"], "message_start");
    assert_eq!(parsed.last().unwrap()["type"], "error");
    assert!(parsed.last().unwrap()["message"].as_str().unwrap().contains("SIGTERM"));
}

#[test]
fn set_stop_flips_the_process_global() {
    // The signal handler in `main.rs` is the only production caller of
    // set_stop, but the function itself must be exercised so the 100%
    // coverage floor stays meaningful. Single-threaded test; no race.
    crate::adapter::STOP.store(false, Ordering::Release);
    crate::adapter::set_stop();
    assert!(crate::adapter::STOP.load(Ordering::Acquire));
    crate::adapter::STOP.store(false, Ordering::Release);
}
