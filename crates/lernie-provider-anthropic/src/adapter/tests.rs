//! Tests for [`super`]. Split out so `adapter.rs` stays under the repo's
//! 300-line cap for code files.

use super::*;
use crate::client::{ANTHROPIC_VERSION, Message, Role};
use httpmock::Method::POST;
use httpmock::MockServer;
use serde_json::Value;

fn body_value<W: AsRef<[u8]>>(raw: W) -> Value {
    serde_json::from_slice(raw.as_ref()).unwrap()
}

fn sample_request_json() -> String {
    serde_json::to_string(&Request {
        model: "claude-sonnet-4-7".into(),
        max_tokens: 32,
        system: None,
        messages: vec![Message {
            role: Role::User,
            content: "hello".into(),
        }],
    })
    .unwrap()
}

/// Drive `run_complete` against `endpoint` with a valid key and the
/// standard sample request. Returns the parsed stdout JSON.
fn complete_against(endpoint: &str) -> Value {
    let mut out = Vec::new();
    run_complete(
        &mut sample_request_json().as_bytes(),
        &mut out,
        Some("k"),
        endpoint,
    )
    .unwrap();
    body_value(&out)
}

fn mock_status(status: u16, body: &str) -> MockServer {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(status).body(body);
    });
    server
}

#[test]
fn describe_shape_matches_contract() {
    let mut buf = Vec::new();
    run_describe(&mut buf).unwrap();
    let v = body_value(&buf);
    assert_eq!(v["name"], "anthropic");
    assert_eq!(v["schema_version"], 2);
    let caps: Vec<&str> = v["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    assert!(caps.contains(&"tool_use_native"));
    assert!(
        caps.contains(&"streaming"),
        "streaming capability lands with bl-de80; describe must advertise it"
    );
    assert_eq!(v["auth_env"][0], "ANTHROPIC_API_KEY");
    assert_eq!(v["endpoint_env"][0], "LERNIE_PROVIDER_ANTHROPIC_ENDPOINT");
    assert!(!v["models"].as_array().unwrap().is_empty());
}

#[test]
fn describe_surfaces_write_errors() {
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("nope"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    assert!(run_describe(&mut Broken).is_err());
}

#[test]
fn complete_happy_path_writes_response() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("x-api-key", "k")
            .header("anthropic-version", ANTHROPIC_VERSION);
        then.status(200).body(
            r#"{"id":"msg_01","model":"claude-sonnet-4-7","stop_reason":"end_turn",
               "content":[{"type":"text","text":"hi"}],
               "usage":{"input_tokens":1,"output_tokens":1}}"#,
        );
    });
    let v = complete_against(&server.base_url());
    mock.assert();
    assert_eq!(v["id"], "msg_01");
    assert_eq!(v["content"][0]["text"], "hi");
}

#[test]
fn complete_missing_api_key_is_fatal() {
    let mut out = Vec::new();
    run_complete(&mut b"".as_ref(), &mut out, None, "http://unused").unwrap();
    let v = body_value(&out);
    assert_eq!(v["type"], "error");
    assert_eq!(v["kind"], "fatal");
    assert!(v["message"].as_str().unwrap().contains("ANTHROPIC_API_KEY"));
}

#[test]
fn complete_empty_api_key_is_fatal() {
    let mut out = Vec::new();
    run_complete(&mut b"".as_ref(), &mut out, Some(""), "http://unused").unwrap();
    assert_eq!(body_value(&out)["kind"], "fatal");
}

#[test]
fn complete_malformed_stdin_is_fatal() {
    let mut out = Vec::new();
    run_complete(
        &mut b"{ not json".as_ref(),
        &mut out,
        Some("k"),
        "http://unused",
    )
    .unwrap();
    let v = body_value(&out);
    assert_eq!(v["kind"], "fatal");
    assert!(v["message"].as_str().unwrap().contains("parse stdin"));
}

#[test]
fn complete_stdin_read_errors_propagate() {
    struct BadRead;
    impl Read for BadRead {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("boom"))
        }
    }
    let mut out = Vec::new();
    let err = run_complete(&mut BadRead, &mut out, Some("k"), "http://unused").unwrap_err();
    assert_eq!(err.to_string(), "boom");
}

#[test]
fn complete_429_is_retryable_with_retry_after() {
    let server = mock_status(429, r#"{"error":"slow down","retry_after":12}"#);
    let v = complete_against(&server.base_url());
    assert_eq!(v["type"], "error");
    assert_eq!(v["kind"], "retryable");
    assert_eq!(v["http_status"], 429);
    assert_eq!(v["retry_after_seconds"], 12);
}

#[test]
fn complete_429_without_retry_after_still_retryable() {
    let server = mock_status(429, "not even json");
    let v = complete_against(&server.base_url());
    assert_eq!(v["kind"], "retryable");
    assert!(v["retry_after_seconds"].is_null());
}

#[test]
fn complete_429_with_non_integer_retry_after_drops_hint() {
    let server = mock_status(429, r#"{"retry_after":"soon"}"#);
    let v = complete_against(&server.base_url());
    assert!(v["retry_after_seconds"].is_null());
}

#[test]
fn complete_5xx_is_retryable() {
    let server = mock_status(503, "upstream flaking");
    let v = complete_against(&server.base_url());
    assert_eq!(v["kind"], "retryable");
    assert_eq!(v["http_status"], 503);
}

#[test]
fn complete_4xx_non_auth_is_fatal() {
    let server = mock_status(400, "bad input");
    let v = complete_against(&server.base_url());
    assert_eq!(v["kind"], "fatal");
    assert_eq!(v["http_status"], 400);
}

#[test]
fn complete_401_is_fatal_auth() {
    let server = mock_status(401, "bad key");
    let v = complete_against(&server.base_url());
    assert_eq!(v["kind"], "fatal");
    assert_eq!(v["http_status"], 401);
    assert!(v["message"].as_str().unwrap().contains("auth"));
}

#[test]
fn complete_network_failure_is_retryable() {
    // Port 1 on localhost is guaranteed-refused.
    let v = complete_against("http://127.0.0.1:1");
    assert_eq!(v["kind"], "retryable");
    assert!(v["http_status"].is_null());
    assert!(v["message"].as_str().unwrap().contains("network"));
}

#[test]
fn complete_malformed_upstream_json_is_fatal() {
    let server = mock_status(200, "{ not json");
    let v = complete_against(&server.base_url());
    assert_eq!(v["kind"], "fatal");
    assert!(v["message"].as_str().unwrap().contains("upstream JSON"));
}

#[test]
fn map_config_error_is_fatal() {
    let mapped = map_error(&client::Error::Config("nope".into()));
    assert_eq!(mapped.kind, ErrorKind::Fatal);
    assert!(mapped.message.contains("config"));
}

#[test]
fn map_sse_error_is_fatal() {
    let mapped = map_error(&client::Error::Sse("malformed frame".into()));
    assert_eq!(mapped.kind, ErrorKind::Fatal);
    assert!(mapped.message.contains("SSE"));
}
