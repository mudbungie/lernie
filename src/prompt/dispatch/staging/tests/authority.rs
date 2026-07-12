//! Segment-authority behavior of the staging sink (ARCH §4.4): an
//! `Error` segment truncates, a `Pause` segment accumulates, `Finish`
//! seals. Also the raw byte shape (array brackets/commas) and the
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
            Event::Usage(Usage::default()),
            Event::Finish {
                reason: FinishReason::Stop,
            },
            Event::Error(CanonicalError {
                kind: ErrorKind::Transport,
                message: "ignored here".into(),
                provider_detail: None,
            }),
            Event::End,
        ],
    );
    assert_eq!(sealed_blocks(w, &path), vec![Content::Text("x".into())]);
}

#[test]
fn sealed_file_is_valid_json_after_multiple_segments() {
    // Guards the raw byte shape (the array brackets/commas) independent
    // of the Content round-trip: a truncated segment then a two-block
    // segment must seal to a well-formed two-element array.
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
        json!([{"type":"text","text":"one"},{"type":"text","text":"two"}])
    );
}
