//! Segment-authority behavior of the staging sink (ARCH §4.4): an
//! `Error` segment truncates, a `Pause` segment accumulates, `Finish`
//! seals — for the segment's blocks and for its usage report alike
//! (§2.3 *Usage rides the entry*). Also the sealed object's raw shape and the
//! ignore-paths for stray deltas / non-content events.

use super::*;
use brazen::{CanonicalError, ContentKind, Delta, ErrorKind, Event, FinishReason, Role, Usage};
use serde_json::{Value, json};
use tempfile::TempDir;

#[test]
fn error_terminated_segment_truncates_then_retry_reaccumulates() {
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    // Attempt 1: a partial block, then an Error segment truncates it.
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            start(0, ContentKind::Text {}),
            delta(0, Delta::TextDelta("discard".into())),
            stop(0),
        ],
    );
    w.truncate_segment().unwrap();
    // Attempt 2 (retry, identical request): the full content re-streams.
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            start(0, ContentKind::Text {}),
            delta(0, Delta::TextDelta("kept".into())),
            stop(0),
        ],
    );
    assert_eq!(sealed_blocks(w, &path), vec![Content::Text("kept".into())]);
}

#[test]
fn pause_terminated_segment_accumulates_across_the_continuation() {
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    // Pause segment: blocks A, B contributed and left un-truncated.
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            start(0, ContentKind::Text {}),
            delta(0, Delta::TextDelta("A".into())),
            stop(0),
            start(1, ContentKind::Text {}),
            delta(1, Delta::TextDelta("B".into())),
            stop(1),
        ],
    );
    // Continuation resumes past them, contributing C, then seals.
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            start(2, ContentKind::Text {}),
            delta(2, Delta::TextDelta("C".into())),
            stop(2),
        ],
    );
    assert_eq!(
        sealed_blocks(w, &path),
        vec![
            Content::Text("A".into()),
            Content::Text("B".into()),
            Content::Text("C".into()),
        ]
    );
}

#[test]
fn deltas_for_a_mismatched_or_absent_block_are_ignored() {
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            // Delta before any content_start: no in-progress block.
            delta(0, Delta::TextDelta("orphan".into())),
            start(0, ContentKind::Text {}),
            // Wrong delta kind for a text block: ignored.
            delta(0, Delta::JsonDelta("nope".into())),
            delta(0, Delta::TextDelta("real".into())),
            stop(0),
            // content_stop with no open block: a no-op.
            stop(1),
        ],
    );
    assert_eq!(sealed_blocks(w, &path), vec![Content::Text("real".into())]);
}

#[test]
fn non_content_events_do_not_touch_the_entry() {
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            Event::message_start(Some("m".into()), Some("mdl".into()), Role::Assistant),
            start(0, ContentKind::Text {}),
            delta(0, Delta::TextDelta("x".into())),
            stop(0),
            Event::Finish {
                reason: FinishReason::Stop,
            },
            Event::Error(CanonicalError {
                kind: ErrorKind::Transport,
                message: "ignored here".into(),
                provider_detail: None,
                retry_after_seconds: None,
            }),
            Event::End,
        ],
    );
    assert_eq!(sealed_blocks(w, &path), vec![Content::Text("x".into())]);
}

#[test]
fn sealed_file_is_valid_json_after_multiple_segments() {
    // Guards the raw byte shape (the entry object, its array brackets and
    // commas) independent of the Content round-trip: a truncated segment
    // then a two-block segment must seal to a well-formed object
    // wrapping a two-element `content` array.
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(&mut w, &[start(0, ContentKind::Text {}), stop(0)]);
    w.truncate_segment().unwrap();
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            start(0, ContentKind::Text {}),
            delta(0, Delta::TextDelta("one".into())),
            stop(0),
            start(1, ContentKind::Text {}),
            delta(1, Delta::TextDelta("two".into())),
            stop(1),
        ],
    );
    w.seal().unwrap();
    let raw = std::fs::read_to_string(&path).unwrap();
    let parsed: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(
        parsed,
        json!({"content": [{"type":"text","text":"one"},{"type":"text","text":"two"}]})
    );
}

/// A `usage` event carrying `input`/`output` counters.
fn usage(input: Option<u32>, output: Option<u32>) -> Event {
    let mut u = Usage::default();
    u.input_tokens = input;
    u.output_tokens = output;
    Event::Usage(u)
}

/// The sealed entry's `usage` sibling, or `Value::Null` when it has none.
fn sealed_usage(w: StagingWriter, path: &std::path::Path) -> Value {
    let parsed: Value = serde_json::from_slice(&sealed_bytes(w, path)).unwrap();
    parsed.get("usage").cloned().unwrap_or(Value::Null)
}

#[test]
fn the_sealed_entry_carries_the_providers_usage_report() {
    // The provider states one report in installments (§2.3 *Usage rides the entry*): the
    // entry seals both counters, recorded verbatim — never summed.
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(
        &mut w,
        &[
            usage(Some(120), Some(0)),
            start(0, ContentKind::Text {}),
            delta(0, Delta::TextDelta("x".into())),
            stop(0),
            usage(None, Some(37)),
            Event::Finish {
                reason: FinishReason::Stop,
            },
            Event::End,
        ],
    );
    assert_eq!(
        sealed_usage(w, &path),
        json!({"input_tokens": 120, "output_tokens": 37})
    );
}

#[test]
fn a_call_the_provider_reported_no_usage_for_seals_without_the_sibling() {
    // Absence is the general path, not an error: a usage-free entry is
    // exactly what every entry looked like before the counters rode along.
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(&mut w, &[start(0, ContentKind::Text {}), stop(0)]);
    assert_eq!(sealed_usage(w, &path), Value::Null);
}

#[test]
fn a_truncated_segments_usage_is_discarded_with_its_blocks() {
    // §4.4: an `Error`-terminated segment contributes nothing — its
    // counters are billed from `response.json` (§6, §8), never from the
    // committed output's own report.
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(&mut w, &[usage(Some(999), Some(999))]);
    w.truncate_segment().unwrap();
    w.begin_segment();
    feed_all(&mut w, &[usage(Some(4), Some(1))]);
    assert_eq!(
        sealed_usage(w, &path),
        json!({"input_tokens": 4, "output_tokens": 1})
    );
}

#[test]
fn a_retried_calls_entry_reports_only_the_sealing_segment() {
    // The counters of a superseded attempt do not leak into the report,
    // even without an explicit truncate: each segment starts fresh.
    let dir = TempDir::new().unwrap();
    let (mut w, path) = new_writer(&dir);
    w.begin_segment();
    feed_all(&mut w, &[usage(Some(50), Some(9))]);
    w.begin_segment();
    feed_all(&mut w, &[usage(Some(60), None)]);
    assert_eq!(sealed_usage(w, &path), json!({"input_tokens": 60}));
}
