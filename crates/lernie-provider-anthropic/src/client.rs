//! Anthropic Messages API client — blocking.
//!
//! One [`Client::send`] call realizes one model call as one HTTP API call
//! against `POST {endpoint}/v1/messages` (terms per `docs/ARCHITECTURE.md`
//! §2.1). [`Client::send_streaming`] performs the same model call with
//! `stream: true`, returning an [`streaming::EventStream`] over Anthropic's
//! native SSE wire events. Tool-use payloads, prompt caching, and retries
//! remain out of scope at this layer (see `docs/ARCHITECTURE.md` §12 and
//! the `v0.1` milestone).
//!
//! Blocking `reqwest` is chosen over async for v0.1: the harness has no
//! in-process concurrency (one adapter subprocess per model call — §4.4),
//! so an async runtime would only add a dependency surface without any
//! parallelism to exploit. Across model calls, concurrency is achieved by
//! spawning multiple adapter subprocesses.
//!
//! # Error taxonomy
//!
//! Each variant of [`Error`] names a distinct failure mode so callers can
//! branch on cause without string-matching:
//!
//! - [`Error::Config`] — local misconfiguration (env var missing,
//!   unsupported auth type). Never reached the network.
//! - [`Error::Network`] — the HTTP API call could not complete (DNS,
//!   connect, TLS, timeout).
//! - [`Error::Auth`] — provider returned 401 or 403.
//! - [`Error::RateLimit`] — provider returned 429.
//! - [`Error::Provider`] — any other non-2xx (typically 4xx validation or
//!   5xx server error).
//! - [`Error::Parse`] — the response body was not valid JSON of the shape
//!   this client expects.
//! - [`Error::Sse`] — a streaming response had a malformed SSE frame or the
//!   connection closed mid-event.

pub mod streaming;

use serde::{Deserialize, Serialize};
use std::io::BufReader;
use std::time::Duration;
use thiserror::Error;

/// Anthropic API version sent on every request (header `anthropic-version`).
/// See <https://docs.anthropic.com/en/api/versioning>.
pub const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Default per-request timeout. Non-streaming message requests should
/// resolve well inside this; exceeding it surfaces as [`Error::Network`].
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

/// Author of a [`Message`] in a request.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// One message in the conversation history sent to the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Input to one model call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    pub model: String,
    pub max_tokens: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<Message>,
}

/// Usage accounting returned by the provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// One block of the model's output. v0.1 only produces `text`; unknown
/// types surface as [`ContentBlock::Unknown`] so future provider-side
/// additions do not fail parsing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    #[serde(other, skip_serializing)]
    Unknown,
}

/// Parsed Messages API response. `stop_reason` is kept as the raw wire
/// string (e.g. `"end_turn"`, `"max_tokens"`) rather than enumified here:
/// see `docs/ARCHITECTURE.md` §2.1 — one of Anthropic's wire values uses a
/// banned term, and the harness does not yet need to branch on it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Response {
    pub id: String,
    pub model: String,
    pub stop_reason: String,
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

impl Response {
    /// Concatenated text from all [`ContentBlock::Text`] blocks, in order.
    /// Non-text blocks are skipped.
    pub fn text(&self) -> String {
        let mut out = String::new();
        for block in &self.content {
            if let ContentBlock::Text { text } = block {
                out.push_str(text);
            }
        }
        out
    }
}

/// Errors surfaced by [`Client::send`]. See the module docstring for the
/// full taxonomy.
#[derive(Debug, Error)]
pub enum Error {
    #[error("anthropic: config: {0}")]
    Config(String),
    #[error("anthropic: network: {0}")]
    Network(#[from] reqwest::Error),
    #[error("anthropic: auth ({status}): {body}")]
    Auth { status: u16, body: String },
    #[error("anthropic: rate-limited ({status}): {body}")]
    RateLimit { status: u16, body: String },
    #[error("anthropic: provider error ({status}): {body}")]
    Provider { status: u16, body: String },
    #[error("anthropic: parse: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("anthropic: sse: {0}")]
    Sse(String),
}

/// Blocking HTTP client for one Anthropic provider config.
#[derive(Debug)]
pub struct Client {
    http: reqwest::blocking::Client,
    endpoint: String,
    api_key: String,
}

impl Client {
    /// Build a client from an explicit endpoint and API key.
    pub fn new(endpoint: impl Into<String>, api_key: impl Into<String>) -> Result<Self, Error> {
        let http = reqwest::blocking::Client::builder()
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .map_err(Error::Network)?;
        Ok(Self {
            http,
            endpoint: endpoint.into(),
            api_key: api_key.into(),
        })
    }

    /// Execute one model call: POST to `/v1/messages`, parse and classify
    /// the response. See the module docstring for error semantics.
    pub fn send(&self, request: &Request) -> Result<Response, Error> {
        let url = format!("{}/v1/messages", self.endpoint.trim_end_matches('/'));
        let http_response = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(request)
            .send()?;

        let status = http_response.status();
        let body = http_response.text()?;

        if status.is_success() {
            return serde_json::from_str(&body).map_err(Error::Parse);
        }

        Err(map_status_error(status.as_u16(), body))
    }

    /// Streaming variant of [`Client::send`]. Posts the request with
    /// `stream: true` and returns an iterator over Anthropic's native SSE
    /// wire events; HTTP-status errors classify exactly as in [`send`]
    /// before the iterator is constructed. Errors arriving as in-band SSE
    /// `error` events surface as [`Error::Provider`] mid-iteration.
    pub fn send_streaming(
        &self,
        request: &Request,
    ) -> Result<streaming::EventStream<BufReader<reqwest::blocking::Response>>, Error> {
        let url = format!("{}/v1/messages", self.endpoint.trim_end_matches('/'));
        let body = StreamingRequest {
            inner: request,
            stream: true,
        };
        let http_response = self
            .http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .json(&body)
            .send()?;

        let status = http_response.status();
        if !status.is_success() {
            let code = status.as_u16();
            let body = http_response.text()?;
            return Err(map_status_error(code, body));
        }
        Ok(streaming::EventStream::new(BufReader::new(http_response)))
    }
}

#[derive(Serialize)]
struct StreamingRequest<'a> {
    #[serde(flatten)]
    inner: &'a Request,
    stream: bool,
}

fn map_status_error(code: u16, body: String) -> Error {
    match code {
        401 | 403 => Error::Auth { status: code, body },
        429 => Error::RateLimit { status: code, body },
        _ => Error::Provider { status: code, body },
    }
}

#[cfg(test)]
mod tests;
