//! HTTP-layer streaming tests: driven against a local `httpmock` server.

use super::*;
use crate::client::{ANTHROPIC_VERSION, Client, ContentBlock, Error};
use httpmock::Method::POST;
use httpmock::MockServer;

#[test]
fn send_streaming_well_formed_yields_all_events_and_accumulates() {
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("accept", "text/event-stream")
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json_body_partial(r#"{"stream": true}"#);
        then.status(200)
            .header("content-type", "text/event-stream")
            .body(WELL_FORMED);
    });

    let stream = client(&server).send_streaming(&request()).unwrap();
    let response = accumulate(stream).unwrap();
    mock.assert();

    assert_eq!(response.id, "msg_1");
    assert_eq!(response.model, "claude-sonnet-4-7");
    assert_eq!(response.stop_reason, "end_turn");
    assert_eq!(response.usage.input_tokens, 5);
    assert_eq!(response.usage.output_tokens, 7);
    assert_eq!(response.content.len(), 1);
    assert!(matches!(
        &response.content[0],
        ContentBlock::Text { text } if text == "Hello world"
    ));
    assert_eq!(response.text(), "Hello world");
}

#[test]
fn error_event_midflight_surfaces_as_provider_error() {
    let body = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"x\",\"model\":\"m\",\"stop_reason\":null,\"content\":[],\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n",
        "\n",
        "event: error\n",
        "data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}\n",
        "\n",
    );
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "text/event-stream")
            .body(body);
    });

    let mut stream = client(&server).send_streaming(&request()).unwrap();
    assert!(matches!(
        stream.next(),
        Some(Ok(Event::MessageStart { .. }))
    ));
    match stream.next() {
        Some(Err(Error::Provider { status, body })) => {
            assert_eq!(status, 200);
            assert!(body.contains("overloaded_error"), "body={body}");
        }
        other => panic!("expected Provider error, got {other:?}"),
    }
    assert!(stream.next().is_none());
}

#[test]
fn non_2xx_status_on_streaming_request_classifies_like_send() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(429).body(r#"{"error":"slow down"}"#);
    });
    let err = client(&server).send_streaming(&request()).unwrap_err();
    let Error::RateLimit { status, body } = err else {
        panic!("expected RateLimit, got {err:?}")
    };
    assert_eq!(status, 429);
    assert!(body.contains("slow down"));
}

#[test]
fn streaming_auth_error_maps_to_auth_variant() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(401).body("nope");
    });
    let err = client(&server).send_streaming(&request()).unwrap_err();
    assert!(
        matches!(err, Error::Auth { status: 401, .. }),
        "got {err:?}"
    );
}

#[test]
fn streaming_500_maps_to_provider_variant() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(500).body("boom");
    });
    let err = client(&server).send_streaming(&request()).unwrap_err();
    assert!(
        matches!(err, Error::Provider { status: 500, .. }),
        "got {err:?}"
    );
}

#[test]
fn streaming_network_failure_maps_to_network_variant() {
    let client = Client::new("http://127.0.0.1:1", "k").unwrap();
    let err = client.send_streaming(&request()).unwrap_err();
    assert!(matches!(err, Error::Network(_)), "got {err:?}");
}
