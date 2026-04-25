//! Translation cases: native Anthropic SSE → §4.4 normalized JSONL.

use super::*;

#[test]
fn run_translates_native_sse_to_normalized_jsonl() {
    let server = MockServer::start();
    mock_sse(&server, HAPPY_SSE);
    let stop = AtomicBool::new(false);
    let events = run_against(&server, &stop);

    let types: Vec<&str> = events.iter().map(|e| e["type"].as_str().unwrap()).collect();
    assert_eq!(
        types,
        vec![
            "message_start",
            "content_block_start",
            "text_delta",
            "text_delta",
            "content_block_stop",
            "content_block_start",
            "tool_use_delta",
            "content_block_stop",
            "message_stop",
        ],
        "ping is dropped; signature_delta is dropped; message_delta folds into terminal"
    );

    assert_eq!(events[0]["message"]["id"], "msg_X");
    assert_eq!(events[2]["text"], "Hi ");
    assert_eq!(events[2]["index"], 0);
    // v0.2 reserves the tool-use payload — the event arrives with just
    // its index, no `partial_json`. v0.3 will add the payload.
    assert_eq!(events[6]["index"], 1);
    assert!(events[6].get("partial_json").is_none(), "v0.2 omits payload, got {}", events[6]);

    let stop = events.last().unwrap();
    assert_eq!(stop["stop_reason"], "end_turn");
    assert_eq!(stop["api_calls"], 1);
    assert_eq!(stop["usage"]["input_tokens"], 4);
    assert_eq!(stop["usage"]["output_tokens"], 11);
    assert_eq!(stop["usage"]["cache_read_input_tokens"], 2);
}

#[test]
fn message_stop_falls_back_to_empty_usage_when_message_start_missing() {
    // Hand-rolled stream with no `message_start` to exercise the
    // "starting_usage absent" branch — we still emit a terminator.
    let body = concat!(
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
        "event: message_stop\n",
        "data: {\"type\":\"message_stop\"}\n\n",
    );
    let server = MockServer::start();
    mock_sse(&server, body);
    let events = run_against(&server, &AtomicBool::new(false));
    let stop = events.last().unwrap();
    assert_eq!(stop["type"], "message_stop");
    assert!(stop["usage"].is_object());
    assert!(stop["usage"].as_object().unwrap().is_empty());
}

#[test]
fn message_stop_preserves_non_object_usage_when_provider_misbehaves() {
    // Some upstreams have shipped `usage` as a number or null in the
    // past; preserve it verbatim so the harness sees what the adapter
    // saw rather than a fabricated object.
    use crate::client::streaming::Event;
    let mut state = crate::adapter::streaming::TerminalState::default();
    crate::adapter::streaming::handle(
        &mut Vec::new(),
        &mut state,
        Event::MessageStart {
            message: serde_json::json!({"usage": "weird-string"}),
        },
    )
    .unwrap();
    let mut out = Vec::new();
    crate::adapter::streaming::handle(&mut out, &mut state, Event::MessageStop).unwrap();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["usage"], "weird-string");
}
