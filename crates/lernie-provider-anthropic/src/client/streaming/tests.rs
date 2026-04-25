//! Streaming-client test helpers and submodule wiring.
//!
//! Tests are split across three submodules, each well under the 300-line
//! code-file cap:
//!
//! - `http` — end-to-end against a local `httpmock` server.
//! - `parser` — SSE frame-parser unit tests driven from in-memory
//!   `Cursor`-style readers.
//! - `accumulate` — unit tests on the `accumulate` fold.
//!
//! No test hits the real Anthropic endpoint.

use super::*;
use crate::client::{Client, Message, Request, Role};
use httpmock::MockServer;

mod accumulate_tests;
mod http;
mod parser;

fn request() -> Request {
    Request {
        model: "claude-sonnet-4-7".into(),
        max_tokens: 32,
        system: None,
        messages: vec![Message {
            role: Role::User,
            content: "hi".into(),
        }],
        tools: None,
    }
}

fn client(server: &MockServer) -> Client {
    Client::new(server.base_url(), "test-key").unwrap()
}

/// Well-formed Anthropic-native stream for "Hello world" across two
/// text_delta chunks. Ends with a clean `message_stop`.
const WELL_FORMED: &str = concat!(
    "event: message_start\n",
    "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"model\":\"claude-sonnet-4-7\",\"stop_reason\":null,\"content\":[],\"usage\":{\"input_tokens\":5,\"output_tokens\":0}}}\n",
    "\n",
    "event: content_block_start\n",
    "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n",
    "\n",
    "event: ping\n",
    "data: {\"type\":\"ping\"}\n",
    "\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello \"}}\n",
    "\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"world\"}}\n",
    "\n",
    "event: content_block_stop\n",
    "data: {\"type\":\"content_block_stop\",\"index\":0}\n",
    "\n",
    "event: message_delta\n",
    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":7}}\n",
    "\n",
    "event: message_stop\n",
    "data: {\"type\":\"message_stop\"}\n",
    "\n",
);
