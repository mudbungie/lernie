//! Error-path cases: HTTP status, mid-stream native error, truncations.

use super::*;

#[test]
fn run_emits_terminal_error_when_http_status_is_429() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(429).body(r#"{"error":"slow","retry_after":7}"#);
    });
    let events = run_against(&server, &AtomicBool::new(false));
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["type"], "error");
    assert_eq!(events[0]["kind"], "retryable");
    assert_eq!(events[0]["http_status"], 429);
    assert_eq!(events[0]["retry_after_seconds"], 7);
}

#[test]
fn run_emits_terminal_error_for_mid_stream_native_error() {
    // Anthropic native `event: error` becomes Error::Provider mid-iteration;
    // streaming wraps it as a normalized error event.
    let body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"x\",\"model\":\"m\",\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n\n",
        "event: error\n",
        "data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"busy\"}}\n\n",
    );
    let server = MockServer::start();
    mock_sse(&server, body);
    let events = run_against(&server, &AtomicBool::new(false));
    assert_eq!(events.last().unwrap()["type"], "error");
    assert_eq!(events.last().unwrap()["http_status"], 200);
}

#[test]
fn run_emits_fatal_when_stream_truncates_without_terminator() {
    // No message_stop, no error event, just connection EOF — surface a
    // fatal so the consumer can't silently drop a half-stream.
    let body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"x\",\"model\":\"m\",\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
    );
    let server = MockServer::start();
    mock_sse(&server, body);
    let events = run_against(&server, &AtomicBool::new(false));
    let last = events.last().unwrap();
    assert_eq!(last["type"], "error");
    assert_eq!(last["kind"], "fatal");
    assert!(
        last["message"].as_str().unwrap().contains("message_stop"),
        "got: {last}"
    );
}

#[test]
fn run_emits_fatal_when_stream_truncates_mid_frame() {
    // Cut off mid-event so EventStream surfaces an Sse error first.
    let body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"x\",\"model\":\"m\",\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_de",
    );
    let server = MockServer::start();
    mock_sse(&server, body);
    let events = run_against(&server, &AtomicBool::new(false));
    let last = events.last().unwrap();
    assert_eq!(last["type"], "error");
    assert_eq!(last["kind"], "fatal");
    assert!(last["message"].as_str().unwrap().contains("SSE"));
}

#[test]
fn drain_treats_iter_error_without_stop_as_provider_fault() {
    // A real Sse error with stop=false surfaces as a fatal provider
    // error, mirroring the SIGTERM-stop variant in tests/sigterm.rs.
    use crate::client::Error;
    use crate::client::streaming::Event;

    let stop = AtomicBool::new(false);
    let events = std::iter::once(Err::<Event, Error>(Error::Sse("malformed frame".into())));
    let mut out = Vec::new();
    crate::adapter::streaming::drain(&mut out, events, &stop).unwrap();
    let parsed = parse_jsonl(&out);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0]["type"], "error");
    assert_eq!(parsed[0]["kind"], "fatal");
    assert!(parsed[0]["message"].as_str().unwrap().contains("SSE"));
}
