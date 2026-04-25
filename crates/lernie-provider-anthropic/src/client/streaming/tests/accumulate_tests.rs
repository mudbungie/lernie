//! Unit tests on [`super::super::accumulate`]. Mix of in-memory
//! event-list feeds (for targeted branches) and a round-trip check
//! against the full [`WELL_FORMED`] stream (via `super::WELL_FORMED`).

use super::super::*;
use super::WELL_FORMED;
use crate::client::{ContentBlock, Error, Response, Usage};
use std::io::Cursor;

#[test]
fn accumulator_round_trips_to_non_streaming_response_shape() {
    let expected = Response {
        id: "msg_1".into(),
        model: "claude-sonnet-4-7".into(),
        stop_reason: "end_turn".into(),
        content: vec![ContentBlock::Text {
            text: "Hello world".into(),
        }],
        usage: Usage {
            input_tokens: 5,
            output_tokens: 7,
        },
    };
    let expected_json = serde_json::to_value(&expected).unwrap();

    let stream = EventStream::new(Cursor::new(WELL_FORMED));
    let accumulated = accumulate(stream).unwrap();
    let accumulated_json = serde_json::to_value(&accumulated).unwrap();

    assert_eq!(expected_json, accumulated_json);
}

#[test]
fn accumulator_errors_when_no_message_start() {
    let body = "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n";
    let stream = EventStream::new(Cursor::new(body));
    let err = accumulate(stream).unwrap_err();
    assert!(matches!(err, Error::Sse(_)), "got {err:?}");
}

#[test]
fn accumulator_propagates_iterator_error() {
    let events: Vec<Result<Event, Error>> = vec![Err(Error::Sse("boom".into()))];
    let err = accumulate(events).unwrap_err();
    assert!(
        matches!(err, Error::Sse(ref m) if m == "boom"),
        "got {err:?}"
    );
}

#[test]
fn content_block_delta_for_missing_index_is_ignored() {
    // A delta whose index was never opened is dropped silently —
    // defensive for out-of-order or unknown-block streams.
    let events: Vec<Result<Event, Error>> = vec![
        Ok(Event::MessageStart {
            message: serde_json::json!({
                "id": "x", "model": "m", "stop_reason": null,
                "usage": {"input_tokens": 0, "output_tokens": 0},
            }),
        }),
        Ok(Event::ContentBlockDelta {
            index: 7,
            delta: serde_json::json!({"type": "text_delta", "text": "ghost"}),
        }),
        Ok(Event::MessageDelta {
            delta: serde_json::json!({"stop_reason": "end_turn"}),
            usage: serde_json::json!({"output_tokens": 0}),
        }),
        Ok(Event::MessageStop),
    ];
    let response = accumulate(events).unwrap();
    assert!(response.content.is_empty());
    assert_eq!(response.stop_reason, "end_turn");
}

#[test]
fn non_text_content_block_start_accumulates_as_unknown() {
    let events: Vec<Result<Event, Error>> = vec![
        Ok(Event::MessageStart {
            message: serde_json::json!({
                "id": "x", "model": "m", "stop_reason": null,
                "usage": {"input_tokens": 0, "output_tokens": 0},
            }),
        }),
        Ok(Event::ContentBlockStart {
            index: 0,
            content_block: serde_json::json!({"type": "tool_use", "id": "t_1"}),
        }),
        // text_delta against a non-text block is a no-op.
        Ok(Event::ContentBlockDelta {
            index: 0,
            delta: serde_json::json!({"type": "text_delta", "text": "ignored"}),
        }),
        Ok(Event::ContentBlockStop { index: 0 }),
        Ok(Event::MessageDelta {
            delta: serde_json::json!({"stop_reason": "end_turn"}),
            usage: serde_json::json!({"output_tokens": 0}),
        }),
        Ok(Event::MessageStop),
    ];
    let response = accumulate(events).unwrap();
    assert_eq!(response.content, vec![ContentBlock::Unknown]);
}

#[test]
fn accumulator_uses_message_start_stop_reason_when_present() {
    // Some providers populate stop_reason directly on the initial
    // message (no later message_delta). accumulate() must take it.
    let events: Vec<Result<Event, Error>> = vec![
        Ok(Event::MessageStart {
            message: serde_json::json!({
                "id": "x", "model": "m", "stop_reason": "end_turn",
                "usage": {"input_tokens": 1, "output_tokens": 2},
            }),
        }),
        Ok(Event::MessageStop),
    ];
    let response = accumulate(events).unwrap();
    assert_eq!(response.stop_reason, "end_turn");
    assert_eq!(response.usage.input_tokens, 1);
    assert_eq!(response.usage.output_tokens, 2);
}
