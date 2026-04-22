//! Anthropic Messages API client — blocking, non-streaming.
//!
//! One [`Client::send`] call realizes one model call as one HTTP API call
//! against `POST {endpoint}/v1/messages` (terms per `docs/ARCHITECTURE.md`
//! §2.1). Streaming, tool-use payloads, prompt caching, and retries are
//! out of scope for v0.1 (see `docs/ARCHITECTURE.md` §12 and the `v0.1`
//! milestone).
//!
//! Blocking `reqwest` is chosen over async for v0.1: the harness has no
//! concurrency yet (one exchange at a time, no subagents), so an async
//! runtime would only add a dependency surface without any parallelism to
//! exploit. The client can be replaced or wrapped when v0.4 introduces
//! concurrent invocations.
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

use crate::config::{Auth, Provider};
use serde::{Deserialize, Serialize};
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
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Request {
    pub model: String,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<Message>,
}

/// Usage accounting returned by the provider.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// One block of the model's output. v0.1 only produces `text`; unknown
/// types surface as [`ContentBlock::Unknown`] so future provider-side
/// additions do not fail parsing.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    #[serde(other)]
    Unknown,
}

/// Parsed Messages API response. `stop_reason` is kept as the raw wire
/// string (e.g. `"end_turn"`, `"max_tokens"`) rather than enumified here:
/// see `docs/ARCHITECTURE.md` §2.1 — one of Anthropic's wire values uses a
/// banned term, and the harness does not yet need to branch on it.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
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

/// Errors surfaced by [`Client::send`] and [`Client::from_provider`]. See
/// the module docstring for the full taxonomy.
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
}

/// Blocking HTTP client for one Anthropic provider config.
#[derive(Debug)]
pub struct Client {
    http: reqwest::blocking::Client,
    endpoint: String,
    api_key: String,
}

impl Client {
    /// Build a client from an explicit endpoint and API key. Prefer
    /// [`Client::from_provider`] when you already have a parsed
    /// [`Provider`] and want env-var resolution.
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

    /// Build a client from a parsed [`Provider`]. Requires
    /// [`Auth::ApiKey`]; other auth types return [`Error::Config`]. The
    /// named environment variable must be set.
    pub fn from_provider(provider: &Provider) -> Result<Self, Error> {
        let env_name = match &provider.auth {
            Auth::ApiKey { env } => env,
            Auth::AwsSigv4 { .. } => {
                return Err(Error::Config(
                    "anthropic client requires api_key auth; got aws_sigv4".into(),
                ));
            }
        };
        let api_key = std::env::var(env_name).map_err(|_| {
            Error::Config(format!(
                "env var {env_name:?} (declared in providers.yaml) is not set"
            ))
        })?;
        Self::new(provider.endpoint.clone(), api_key)
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

        let code = status.as_u16();
        Err(match code {
            401 | 403 => Error::Auth { status: code, body },
            429 => Error::RateLimit { status: code, body },
            _ => Error::Provider { status: code, body },
        })
    }
}
