//! Unit tests for the brazen `v=1` event assembler.

use super::*;
use brazen::{ContentKind, Delta, ErrorKind, Event, FinishReason, Role, Usage};

fn start() -> Event {
    Event::message_start(Some("m".into()), Some("model".into()), Role::Assistant)
}

fn feed_all(events: Vec<Event>) -> Result<SegmentOutcome, AssemblyError> {
    let mut a = Assembler::new();
    for e in events {
        a.feed(e);
    }
    a.into_outcome()
}

fn expect_complete(events: Vec<Event>) -> Completion {
    match feed_all(events).unwrap_or_else(|_| panic!("tool input parse failed")) {
        SegmentOutcome::Complete(c) => c,
        _ => panic!("expected Complete"),
    }
}

#[test]
fn text_deltas_accumulate_into_one_text_block() {
    let c = expect_complete(vec![
        start(),
        Event::ContentStart {
            index: 0,
            kind: ContentKind::Text {},
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::TextDelta("Hel".into()),
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::TextDelta("lo".into()),
        },
        Event::ContentStop { index: 0 },
        Event::Finish {
            reason: FinishReason::Stop,
        },
        Event::End,
    ]);
    assert_eq!(c.content, vec![Content::Text("Hello".into())]);
    assert!(!c.is_tool_use());
    assert_eq!(c.handshake_v(), Some(brazen::EVENT_SCHEMA_VERSION));
}

#[test]
fn tool_use_json_deltas_fold_and_parse() {
    let c = expect_complete(vec![
        start(),
        Event::ContentStart {
            index: 0,
            kind: ContentKind::ToolUse {
                id: "t1".into(),
                name: "bash".into(),
            },
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::JsonDelta("{\"cmd\":".into()),
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::JsonDelta("\"ls\"}".into()),
        },
        Event::ContentStop { index: 0 },
        Event::Finish {
            reason: FinishReason::ToolUse,
        },
        Event::End,
    ]);
    assert!(c.is_tool_use());
    match &c.content[0] {
        Content::ToolUse { id, name, input } => {
            assert_eq!(id, "t1");
            assert_eq!(name, "bash");
            assert_eq!(input["cmd"], "ls");
        }
        other => panic!("expected ToolUse, got {other:?}"),
    }
}

#[test]
fn empty_tool_json_defaults_to_empty_object() {
    let c = expect_complete(vec![
        start(),
        Event::ContentStart {
            index: 0,
            kind: ContentKind::ToolUse {
                id: "t".into(),
                name: "noop".into(),
            },
        },
        Event::ContentStop { index: 0 },
        Event::Finish {
            reason: FinishReason::ToolUse,
        },
        Event::End,
    ]);
    match &c.content[0] {
        Content::ToolUse { input, .. } => assert_eq!(*input, serde_json::json!({})),
        other => panic!("expected ToolUse, got {other:?}"),
    }
}

#[test]
fn invalid_tool_json_surfaces_assembly_error() {
    let err = feed_all(vec![
        start(),
        Event::ContentStart {
            index: 0,
            kind: ContentKind::ToolUse {
                id: "t".into(),
                name: "x".into(),
            },
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::JsonDelta("{not json".into()),
        },
        Event::ContentStop { index: 0 },
        Event::Finish {
            reason: FinishReason::ToolUse,
        },
        Event::End,
    ]);
    assert!(err.is_err(), "malformed tool input JSON must error");
}

#[test]
fn error_event_is_failed_outcome() {
    let out = feed_all(vec![
        start(),
        Event::Error(brazen::CanonicalError {
            kind: ErrorKind::Provider { status: 529 },
            message: "overloaded".into(),
            provider_detail: None,
        }),
        Event::End,
    ])
    .unwrap();
    match out {
        SegmentOutcome::Failed(e) => {
            assert!(e.retryable());
            assert_eq!(e.message, "overloaded");
        }
        _ => panic!("expected Failed"),
    }
}

#[test]
fn no_end_is_half_stream() {
    let out = feed_all(vec![
        start(),
        Event::ContentStart {
            index: 0,
            kind: ContentKind::Text {},
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::TextDelta("par".into()),
        },
    ])
    .unwrap();
    assert!(matches!(out, SegmentOutcome::HalfStream));
}

#[test]
fn thinking_and_forward_compat_blocks_are_not_replayed() {
    // A thinking block plus no-op Usage/ContentStop events fold to
    // nothing; only the text block survives into content.
    let c = expect_complete(vec![
        start(),
        Event::ContentStart {
            index: 0,
            kind: ContentKind::Thinking {},
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::ThinkingDelta("hmm".into()),
        },
        Event::ContentStop { index: 0 },
        Event::Usage(Usage::default()),
        Event::ContentStart {
            index: 1,
            kind: ContentKind::Text {},
        },
        Event::ContentDelta {
            index: 1,
            delta: Delta::TextDelta("answer".into()),
        },
        Event::Finish {
            reason: FinishReason::Stop,
        },
        Event::End,
    ]);
    assert_eq!(c.content, vec![Content::Text("answer".into())]);
}

#[test]
fn delta_for_unknown_block_is_dropped() {
    // A text_delta targeting an index that never opened, and a
    // json_delta targeting a text block, are both ignored.
    let c = expect_complete(vec![
        start(),
        Event::ContentStart {
            index: 0,
            kind: ContentKind::Text {},
        },
        Event::ContentDelta {
            index: 5,
            delta: Delta::TextDelta("void".into()),
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::JsonDelta("void".into()),
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::TextDelta("kept".into()),
        },
        Event::Finish {
            reason: FinishReason::Stop,
        },
        Event::End,
    ]);
    assert_eq!(c.content, vec![Content::Text("kept".into())]);
}

#[test]
fn events_after_end_are_ignored() {
    let c = expect_complete(vec![
        start(),
        Event::ContentStart {
            index: 0,
            kind: ContentKind::Text {},
        },
        Event::ContentDelta {
            index: 0,
            delta: Delta::TextDelta("done".into()),
        },
        Event::Finish {
            reason: FinishReason::Stop,
        },
        Event::End,
        // Ignored — the stream already ended.
        Event::ContentDelta {
            index: 0,
            delta: Delta::TextDelta("ignored".into()),
        },
    ]);
    assert_eq!(c.content, vec![Content::Text("done".into())]);
}
