//! Tests for [`super`]. Driven against a locally-bound `httpmock` server
//! so every HTTP branch in the error taxonomy is exercised without
//! touching the real provider. No live API calls.

use super::*;
use httpmock::Method::POST;
use httpmock::MockServer;

fn request() -> Request {
    Request {
        model: "claude-sonnet-4-7".into(),
        max_tokens: 32,
        system: None,
        messages: vec![Message {
            role: Role::User,
            content: "hello".into(),
        }],
        tools: None,
    }
}

fn client(server: &MockServer) -> Client {
    Client::new(server.base_url(), "test-key").unwrap()
}

#[test]
fn happy_path_parses_message_and_usage() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("x-api-key", "test-key")
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{
              "id": "msg_01",
              "type": "message",
              "role": "assistant",
              "model": "claude-sonnet-4-7",
              "stop_reason": "end_turn",
              "content": [{"type": "text", "text": "hi there"}],
              "usage": {"input_tokens": 3, "output_tokens": 2}
            }"#,
            );
    });

    let response = client(&server).send(&request()).unwrap();

    mock.assert();
    assert_eq!(response.id, "msg_01");
    assert_eq!(response.model, "claude-sonnet-4-7");
    assert_eq!(response.stop_reason, "end_turn");
    assert_eq!(response.usage.input_tokens, 3);
    assert_eq!(response.usage.output_tokens, 2);
    assert_eq!(response.content.len(), 1);
    assert!(matches!(
        &response.content[0],
        ContentBlock::Text { text } if text == "hi there"
    ));
    assert_eq!(response.text(), "hi there");
}

#[test]
fn unauthorized_maps_to_auth_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(401).body(r#"{"error":"invalid x-api-key"}"#);
    });

    let err = client(&server).send(&request()).unwrap_err();
    let Error::Auth { status, body } = err else {
        panic!("expected Auth, got {err:?}")
    };
    assert_eq!(status, 401);
    assert!(body.contains("invalid x-api-key"));
}

#[test]
fn rate_limited_maps_to_rate_limit_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(429).body(r#"{"error":"slow down"}"#);
    });

    let err = client(&server).send(&request()).unwrap_err();
    let Error::RateLimit { status, body } = err else {
        panic!("expected RateLimit, got {err:?}")
    };
    assert_eq!(status, 429);
    assert!(body.contains("slow down"));
}

#[test]
fn server_error_maps_to_provider_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(500).body("upstream boom");
    });

    let err = client(&server).send(&request()).unwrap_err();
    let Error::Provider { status, body } = err else {
        panic!("expected Provider, got {err:?}")
    };
    assert_eq!(status, 500);
    assert_eq!(body, "upstream boom");
}

#[test]
fn malformed_json_body_maps_to_parse_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "application/json")
            .body("{ not json");
    });

    let err = client(&server).send(&request()).unwrap_err();
    assert!(matches!(err, Error::Parse(_)), "got {err:?}");
}

#[test]
fn network_failure_maps_to_network_error() {
    // Port 1 on localhost is guaranteed not to accept connections.
    let client = Client::new("http://127.0.0.1:1", "k").unwrap();
    let err = client.send(&request()).unwrap_err();
    assert!(matches!(err, Error::Network(_)), "got {err:?}");
}

#[test]
fn endpoint_trailing_slash_is_tolerated() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200).body(
            r#"{"id":"x","model":"m","stop_reason":"end_turn",
            "content":[],"usage":{"input_tokens":0,"output_tokens":0}}"#,
        );
    });

    let base = format!("{}/", server.base_url());
    let client = Client::new(base, "k").unwrap();
    client.send(&request()).unwrap();
    mock.assert();
}

#[test]
fn bad_request_without_rate_limit_status_is_provider_error() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(400).body("bad input");
    });

    let err = client(&server).send(&request()).unwrap_err();
    let Error::Provider { status, .. } = err else {
        panic!("expected Provider, got {err:?}")
    };
    assert_eq!(status, 400);
}

// Type-only tests live next to their module in `client::types::tests`
// so HTTP-driven cases stay colocated here. See `client/types/tests.rs`.
