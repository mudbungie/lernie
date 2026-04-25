//! Tests for [`super`]. Split into submodules so each file stays under
//! the repo's 300-line cap; shared fixtures live here.

use crate::client::{ANTHROPIC_VERSION, Client, Message, Request, Role};
use httpmock::Method::POST;
use httpmock::MockServer;
use serde_json::Value;
use std::sync::atomic::AtomicBool;

mod dispatch;
mod errors;
mod sigterm;
mod translation;

const SSE_HEADERS: &[(&str, &str)] = &[("content-type", "text/event-stream")];

pub(super) fn request() -> Request {
    Request {
        model: "claude-sonnet-4-7".into(),
        max_tokens: 32,
        system: None,
        messages: vec![Message {
            role: Role::User,
            content: "hi".into(),
        }],
    }
}

pub(super) fn run_against(server: &MockServer, stop: &AtomicBool) -> Vec<Value> {
    let client = Client::new(server.base_url(), "test-key").unwrap();
    let mut out = Vec::new();
    super::run(&mut out, &client, &request(), stop).unwrap();
    parse_jsonl(&out)
}

pub(super) fn parse_jsonl(bytes: &[u8]) -> Vec<Value> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

pub(super) fn mock_sse(server: &MockServer, body: &str) {
    server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("x-api-key", "test-key")
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("accept", "text/event-stream");
        let mut t = then.status(200);
        for (k, v) in SSE_HEADERS {
            t = t.header(*k, *v);
        }
        t.body(body);
    });
}

/// Standard well-formed Anthropic native SSE stream.
pub(super) const HAPPY_SSE: &str = concat!(
    "event: message_start\n",
    "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_X\",\"model\":\"claude-sonnet-4-7\",\"stop_reason\":null,\"content\":[],\"usage\":{\"input_tokens\":4,\"output_tokens\":0,\"cache_read_input_tokens\":2}}}\n",
    "\n",
    "event: ping\n",
    "data: {\"type\":\"ping\"}\n",
    "\n",
    "event: content_block_start\n",
    "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n",
    "\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi \"}}\n",
    "\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"there\"}}\n",
    "\n",
    "event: content_block_stop\n",
    "data: {\"type\":\"content_block_stop\",\"index\":0}\n",
    "\n",
    "event: content_block_start\n",
    "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"t1\",\"name\":\"bash\",\"input\":{}}}\n",
    "\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"a\\\":\"}}\n",
    "\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"signature_delta\",\"signature\":\"ignore-me\"}}\n",
    "\n",
    "event: content_block_stop\n",
    "data: {\"type\":\"content_block_stop\",\"index\":1}\n",
    "\n",
    "event: message_delta\n",
    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":11}}\n",
    "\n",
    "event: message_stop\n",
    "data: {\"type\":\"message_stop\"}\n",
    "\n",
);
