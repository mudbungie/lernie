//! Adapter-contract streaming events (§4.4 "Response shape (streaming)").
//!
//! Lives in its own file so [`super`] stays under the repo's 300-line cap
//! for code files. The non-streaming [`super::Response`] and the
//! streaming [`StreamEvent`] are both consumed by `src/prompt`, but they
//! are independent shapes — splitting keeps each parseable in isolation.

use super::Usage;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One event in the adapter-contract streaming wire. Tag is `type`;
/// variants are the event names listed in the spec. `message_stop` is
/// terminal and carries `usage` + `api_calls`; an `error` event
/// terminates a failed stream in lieu of `message_stop`.
///
/// Tool-use payloads are intentionally minimal at v0.2: the shape is
/// reserved for v0.3 when tool use lands, and `partial_json` carries the
/// streaming JSON fragment Anthropic emits today.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    /// Initial event. Carries the provider's native message envelope —
    /// id, model, starting usage. Held as [`Value`] so adapters for
    /// non-Anthropic providers can pass through whichever fields their
    /// upstream supplies without harness churn.
    MessageStart { message: Value },
    /// One content block began. `content_block` is the opening payload
    /// (e.g. `{"type":"text","text":""}`).
    ContentBlockStart { index: u32, content_block: Value },
    /// Incremental text appended to the text block at `index`.
    TextDelta { index: u32, text: String },
    /// A streaming chunk for the tool-use block at `index`. The shape
    /// is reserved at v0.2 — `partial_json` lands when v0.3 adds tool
    /// use; for now adapters emit just the index, and v0.2 consumers
    /// see the event arrive without a payload.
    ToolUseDelta {
        index: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        partial_json: Option<String>,
    },
    /// One content block finished.
    ContentBlockStop { index: u32 },
    /// Terminal event for a successful stream. `usage` mirrors the
    /// non-streaming [`Usage`] shape; `api_calls` counts HTTP requests
    /// the adapter made under one `complete` invocation (≥1).
    MessageStop {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        stop_reason: Option<String>,
        usage: Usage,
        api_calls: u32,
    },
    /// Terminal event for a failed stream. Same fields as the
    /// non-streaming error object.
    Error {
        kind: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        http_status: Option<u16>,
        message: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        retry_after_seconds: Option<u64>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_message_start_with_arbitrary_message_payload() {
        let body = r#"{"type":"message_start","message":{"id":"m1","model":"x"}}"#;
        let e: StreamEvent = serde_json::from_str(body).unwrap();
        match e {
            StreamEvent::MessageStart { message } => {
                assert_eq!(message["id"], "m1");
                assert_eq!(message["model"], "x");
            }
            other => panic!("expected MessageStart, got {other:?}"),
        }
    }

    #[test]
    fn parses_content_block_start_and_stop() {
        let start = r#"{"type":"content_block_start","index":0,
            "content_block":{"type":"text","text":""}}"#;
        let stop = r#"{"type":"content_block_stop","index":2}"#;
        let s: StreamEvent = serde_json::from_str(start).unwrap();
        assert!(matches!(s, StreamEvent::ContentBlockStart { index: 0, .. }));
        let s: StreamEvent = serde_json::from_str(stop).unwrap();
        assert!(matches!(s, StreamEvent::ContentBlockStop { index: 2 }));
    }

    #[test]
    fn parses_text_and_tool_use_deltas() {
        let t = r#"{"type":"text_delta","index":0,"text":"hello"}"#;
        // v0.2 adapters emit tool_use_delta with no payload.
        let u_v0_2 = r#"{"type":"tool_use_delta","index":1}"#;
        // v0.3 will add the partial_json field; the parser accepts it now.
        let u_v0_3 = r#"{"type":"tool_use_delta","index":2,"partial_json":"{\"a\":"}"#;
        match serde_json::from_str(t).unwrap() {
            StreamEvent::TextDelta { index, text } => {
                assert_eq!(index, 0);
                assert_eq!(text, "hello");
            }
            other => panic!("got {other:?}"),
        }
        match serde_json::from_str(u_v0_2).unwrap() {
            StreamEvent::ToolUseDelta { index, partial_json } => {
                assert_eq!(index, 1);
                assert!(partial_json.is_none(), "v0.2 omits the payload");
            }
            other => panic!("got {other:?}"),
        }
        match serde_json::from_str(u_v0_3).unwrap() {
            StreamEvent::ToolUseDelta { index, partial_json } => {
                assert_eq!(index, 2);
                assert_eq!(partial_json.as_deref(), Some(r#"{"a":"#));
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn parses_message_stop_with_usage_and_api_calls() {
        let body = r#"{"type":"message_stop","stop_reason":"end_turn",
            "usage":{"input_tokens":3,"output_tokens":7},"api_calls":1}"#;
        let e: StreamEvent = serde_json::from_str(body).unwrap();
        match e {
            StreamEvent::MessageStop {
                stop_reason,
                usage,
                api_calls,
            } => {
                assert_eq!(stop_reason.as_deref(), Some("end_turn"));
                assert_eq!(usage.input_tokens, 3);
                assert_eq!(usage.output_tokens, 7);
                assert_eq!(api_calls, 1);
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn parses_terminal_error_event() {
        let body = r#"{"type":"error","kind":"retryable","http_status":429,
            "message":"slow down","retry_after_seconds":12}"#;
        let e: StreamEvent = serde_json::from_str(body).unwrap();
        match e {
            StreamEvent::Error {
                kind,
                http_status,
                message,
                retry_after_seconds,
            } => {
                assert_eq!(kind, "retryable");
                assert_eq!(http_status, Some(429));
                assert_eq!(message, "slow down");
                assert_eq!(retry_after_seconds, Some(12));
            }
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn round_trips_each_variant() {
        let cases = [
            r#"{"type":"message_start","message":{"id":"x"}}"#,
            r#"{"type":"content_block_start","index":0,"content_block":{}}"#,
            r#"{"type":"text_delta","index":0,"text":"hi"}"#,
            r#"{"type":"tool_use_delta","index":1}"#,
            r#"{"type":"content_block_stop","index":0}"#,
            r#"{"type":"message_stop","usage":{"input_tokens":1,"output_tokens":2},"api_calls":1}"#,
            r#"{"type":"error","kind":"fatal","message":"nope"}"#,
        ];
        for body in cases {
            let e: StreamEvent = serde_json::from_str(body).unwrap();
            let bytes = serde_json::to_vec(&e).unwrap();
            let e2: StreamEvent = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(e, e2, "round-trip differed for: {body}");
        }
    }
}
