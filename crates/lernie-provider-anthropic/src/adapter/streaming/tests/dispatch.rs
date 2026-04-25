//! End-to-end `run_complete` dispatch: prove that the `stream` field on
//! the request envelope routes to streaming/non-streaming correctly.

use super::*;
use crate::adapter::run_complete_with_stop;

#[test]
fn run_complete_with_stream_true_routes_to_streaming() {
    let server = MockServer::start();
    mock_sse(&server, HAPPY_SSE);
    let request_json = serde_json::json!({
        "model": "claude-sonnet-4-7",
        "max_tokens": 8,
        "messages": [{"role": "user", "content": "hi"}],
        "stream": true,
    })
    .to_string();
    let mut out = Vec::new();
    let stop = AtomicBool::new(false);
    run_complete_with_stop(
        &mut request_json.as_bytes(),
        &mut out,
        Some("test-key"),
        &server.base_url(),
        &stop,
    )
    .unwrap();
    let events = parse_jsonl(&out);
    assert!(events.len() > 1, "expected JSONL stream, got {events:?}");
    assert_eq!(events.first().unwrap()["type"], "message_start");
    assert_eq!(events.last().unwrap()["type"], "message_stop");
}

#[test]
fn run_complete_default_stays_non_streaming() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200).body(
            r#"{"id":"msg_q","model":"m","stop_reason":"end_turn",
               "content":[{"type":"text","text":"hi"}],
               "usage":{"input_tokens":1,"output_tokens":1}}"#,
        );
    });
    let request_json = serde_json::json!({
        "model": "claude-sonnet-4-7",
        "max_tokens": 8,
        "messages": [{"role":"user","content":"hi"}],
    })
    .to_string();
    let mut out = Vec::new();
    let stop = AtomicBool::new(false);
    run_complete_with_stop(
        &mut request_json.as_bytes(),
        &mut out,
        Some("k"),
        &server.base_url(),
        &stop,
    )
    .unwrap();
    let v: Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["id"], "msg_q");
    assert_eq!(v["content"][0]["text"], "hi");
}
