//! §4.4 JSONL stream synthesis helpers for the harness test suite.
//!
//! Tests script the harness's streaming `complete` path by handing
//! [`super::fixtures::StubAdapter`] pre-baked JSONL bytes — one event
//! per `\n`-terminated line. The helpers here turn ergonomic content-
//! block descriptions (text, tool_use) into the §4.4 wire shape so
//! tests focus on what the model "said," not on the event protocol.
//!
//! Lives in its own file to keep `fixtures.rs` under the repo's 300-
//! line per-file cap.

use serde_json::{Value, json};

/// Parse a JSONL byte buffer into one [`Value`] per non-empty line.
/// Used by tests that read `response.json` back to assert event-stream
/// shape on disk.
pub(super) fn parse_jsonl(bytes: &[u8]) -> Vec<Value> {
    bytes
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_slice(l).expect("each line is valid JSON"))
        .collect()
}

/// Build a §4.4 JSONL stream from a single text response — the
/// streaming-mode equivalent of the legacy non-streaming
/// `HAPPY_RESPONSE_JSON` payload. Used as `complete_bytes` for
/// [`super::fixtures::StubAdapter::happy`] in single-step happy-path
/// tests.
pub(super) fn happy_text_stream(text: &str, stop_reason: &str) -> Vec<u8> {
    streaming_response(stop_reason, &json!([{"type":"text","text":text}]))
}

/// Pre-baked happy-path stream with `stop_reason=end_turn` and a single
/// `"hi there"` text block. Mirrors the legacy `HAPPY_RESPONSE_JSON`
/// non-streaming body; tests that just need *any* successful complete
/// pull this off the shelf.
pub(super) fn happy_response_bytes() -> Vec<u8> {
    happy_text_stream("hi there", "end_turn")
}

/// Synthesize a §4.4 JSONL byte stream from one set of content blocks
/// plus a stop_reason. Each text block emits a `text_delta` carrying
/// its full `text` field; each tool_use block emits an
/// `input_json_delta` carrying its serialized `input` as one
/// `partial_json` payload. The terminal `message_stop` carries
/// `stop_reason`, a stub usage object, and `api_calls = 1`.
pub(super) fn streaming_response(stop_reason: &str, content: &serde_json::Value) -> Vec<u8> {
    let mut out = Vec::new();
    let mut push = |v: serde_json::Value| {
        out.extend_from_slice(&serde_json::to_vec(&v).unwrap());
        out.push(b'\n');
    };
    push(json!({
        "type":"message_start",
        "message":{"id":"msg_x","model":"claude-sonnet-4-7",
            "usage":{"input_tokens":5,"output_tokens":0}}
    }));
    let blocks = content.as_array().expect("content must be an array");
    for (idx, block) in blocks.iter().enumerate() {
        let kind = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match kind {
            "text" => {
                let text = block.get("text").and_then(|v| v.as_str()).unwrap_or("");
                push(json!({"type":"content_block_start","index":idx,
                    "content_block":{"type":"text","text":""}}));
                push(json!({"type":"text_delta","index":idx,"text":text}));
                push(json!({"type":"content_block_stop","index":idx}));
            }
            "tool_use" => {
                let id = block.get("id").cloned().unwrap_or(json!(""));
                let name = block.get("name").cloned().unwrap_or(json!(""));
                let input = block.get("input").cloned().unwrap_or(json!({}));
                let partial_json = serde_json::to_string(&input).unwrap();
                push(json!({"type":"content_block_start","index":idx,
                    "content_block":{"type":"tool_use","id":id,"name":name,"input":{}}}));
                push(json!({"type":"tool_use_delta","index":idx,
                    "partial_json":partial_json}));
                push(json!({"type":"content_block_stop","index":idx}));
            }
            _ => panic!("streaming_response: unsupported block type {kind:?}"),
        }
    }
    push(json!({"type":"message_stop","stop_reason":stop_reason,
        "usage":{"input_tokens":5,"output_tokens":3},"api_calls":1}));
    out
}
