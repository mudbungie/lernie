//! Subcommand backends for the `lernie-provider-anthropic` binary.
//!
//! The binary itself (`src/main.rs`) is a thin wrapper — it parses argv,
//! installs a signal handler, and delegates to the functions here. This
//! module is the part with logic and tests.
//!
//! Wire shape matches the provider-adapter contract in
//! `docs/ARCHITECTURE.md` §4.4:
//!
//! - [`run_describe`] writes the adapter's self-description JSON.
//! - [`run_complete`] reads an Anthropic Messages-API request on stdin,
//!   issues one HTTP API call, and writes either the parsed response or an
//!   in-band error object on stdout. The process exits `0` in both cases;
//!   non-zero is reserved for adapter-side crashes (§4.4).
//!
//! The non-streaming response is the Anthropic Messages-API wire shape,
//! passed through verbatim (ARCH §4.4 "Response shape (non-streaming)"):
//! `{ id, model, stop_reason, content, usage }` at top level, with no
//! `type` tag. Errors are distinguished by `{ "type": "error", ... }`; the
//! reserved `type` field is the only way a consumer tells success from
//! error on the same stream.
//!
//! Streaming, tool-use, and prompt caching are out of scope for v0.1; see
//! the binary's `--help` and `docs/ARCHITECTURE.md` §12.

use crate::client::{self, Client, Request};
use serde::Serialize;
use std::io::{Read, Write};

/// Adapter name returned from `describe` and used in the binary name suffix.
pub const ADAPTER_NAME: &str = "anthropic";

/// `describe.schema_version`. Bumped on breaking changes to the adapter
/// contract; the harness rejects unknown major versions at load (ARCH §4.4).
///
/// v2 — `complete` no longer accepts `--endpoint`; endpoint flows via the
/// env var named in `describe.endpoint_env` instead. Adding `endpoint_env`
/// to `describe` is additive, but removing the argv flag is breaking, so
/// the contract version bumps.
pub const SCHEMA_VERSION: u32 = 2;

/// Default upstream endpoint when the binary's `endpoint_env` var is unset.
pub const DEFAULT_ENDPOINT: &str = "https://api.anthropic.com";

/// Env vars whose *values* the harness must forward to the adapter process
/// (ARCH §4.4 "Auth"). Declared here so `describe` and the binary agree.
pub const AUTH_ENV: &[&str] = &["ANTHROPIC_API_KEY"];

/// Env vars the harness sets to `providers.<name>.endpoint` before invoking
/// `complete` (ARCH §4.4 "Endpoint"). Symmetric with [`AUTH_ENV`]: the binary
/// reads whichever of these is set; the harness needs only the name.
pub const ENDPOINT_ENV: &[&str] = &["LERNIE_PROVIDER_ANTHROPIC_ENDPOINT"];

/// Capabilities advertised by this v0.1 non-streaming adapter. `streaming`
/// is deliberately absent — that capability lands with the streaming
/// children of the bl-d15d epic.
pub const CAPABILITIES: &[&str] = &["tool_use_native", "prompt_caching", "stop_sequences"];

/// Model ids this adapter knows about. Informational: the harness may call
/// `complete` with any model string; the upstream validates.
pub const MODELS: &[&str] = &["claude-sonnet-4-7", "claude-haiku-4-5"];

/// Classification of [`AdapterError::kind`]. Mirrors the contract in
/// `docs/ARCHITECTURE.md` §4.4:
/// - `retryable` — a retry with the same request *might* succeed (429, 5xx,
///   transient network failure).
/// - `fatal` — no point retrying (400, 401/403, malformed upstream JSON,
///   local misconfig).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ErrorKind {
    Retryable,
    Fatal,
}

/// In-band error object. This is what the binary writes to stdout when a
/// `complete` invocation fails; the adapter still exits `0` (ARCH §4.4).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AdapterError {
    #[serde(rename = "type")]
    pub kind_tag: &'static str,
    pub kind: ErrorKind,
    pub http_status: Option<u16>,
    pub message: String,
    pub retry_after_seconds: Option<u64>,
}

impl AdapterError {
    fn fatal(message: impl Into<String>) -> Self {
        Self {
            kind_tag: "error",
            kind: ErrorKind::Fatal,
            http_status: None,
            message: message.into(),
            retry_after_seconds: None,
        }
    }

    fn retryable(message: impl Into<String>) -> Self {
        Self {
            kind_tag: "error",
            kind: ErrorKind::Retryable,
            http_status: None,
            message: message.into(),
            retry_after_seconds: None,
        }
    }

    fn with_status(mut self, status: u16) -> Self {
        self.http_status = Some(status);
        self
    }
}

/// Self-description object written by `describe`. Matches the shape in
/// ARCH §4.4; field order is stable so diffs are readable.
#[derive(Debug, Serialize)]
struct Describe<'a> {
    name: &'a str,
    schema_version: u32,
    capabilities: &'a [&'a str],
    models: &'a [&'a str],
    auth_env: &'a [&'a str],
    endpoint_env: &'a [&'a str],
}

/// Write the `describe` JSON to `out`. Never fails intrinsically — the only
/// error is a write failure on `out`.
pub fn run_describe<W: Write>(out: &mut W) -> std::io::Result<()> {
    let body = Describe {
        name: ADAPTER_NAME,
        schema_version: SCHEMA_VERSION,
        capabilities: CAPABILITIES,
        models: MODELS,
        auth_env: AUTH_ENV,
        endpoint_env: ENDPOINT_ENV,
    };
    serde_json::to_writer(&mut *out, &body).map_err(std::io::Error::other)?;
    out.write_all(b"\n")?;
    Ok(())
}

/// Read one Messages-API request from `stdin`, dispatch it, and write
/// either the parsed response or an in-band [`AdapterError`] to `stdout`.
/// Returns `Ok(())` whenever stdout was written successfully; the caller
/// should exit `0`.
pub fn run_complete<R: Read, W: Write>(
    stdin: &mut R,
    stdout: &mut W,
    api_key: Option<&str>,
    endpoint: &str,
) -> std::io::Result<()> {
    let api_key = match api_key {
        Some(k) if !k.is_empty() => k,
        _ => return write_json(stdout, &AdapterError::fatal(missing_key_message())),
    };

    let mut raw = String::new();
    stdin.read_to_string(&mut raw)?;
    let request: Request = match serde_json::from_str(&raw) {
        Ok(r) => r,
        Err(e) => {
            let err = AdapterError::fatal(format!("could not parse stdin JSON: {e}"));
            return write_json(stdout, &err);
        }
    };

    // Client::new's only failure mode is the reqwest builder failing to
    // construct a TLS/DNS stack — an infra-level crash, not a provider
    // error. `.expect` surfaces it as an adapter-side fault, which is
    // exactly how the contract treats non-zero exits (§4.4).
    let client = Client::new(endpoint, api_key).expect("reqwest blocking client builder failed");

    match client.send(&request) {
        Ok(response) => write_json(stdout, &response),
        Err(e) => write_json(stdout, &map_error(&e)),
    }
}

fn missing_key_message() -> String {
    format!(
        "env var {:?} is not set (declared in describe.auth_env)",
        AUTH_ENV[0]
    )
}

/// Translate the transport-level [`client::Error`] taxonomy into the
/// adapter-contract [`AdapterError`] taxonomy.
///
/// The mapping reflects the contract's retry intent (ARCH §4.4): things the
/// harness could usefully retry (`Retryable`) vs things it should not
/// (`Fatal`). 5xx and 429 are retryable; 4xx (including auth) is fatal;
/// network failures are retryable; parse errors are fatal because a
/// malformed upstream response is not going to un-malform itself.
fn map_error(e: &client::Error) -> AdapterError {
    match e {
        client::Error::Config(msg) => AdapterError::fatal(format!("config: {msg}")),
        client::Error::Network(err) => AdapterError::retryable(format!("network: {err}")),
        client::Error::Auth { status, body } => {
            AdapterError::fatal(format!("auth: {body}")).with_status(*status)
        }
        client::Error::RateLimit { status, body } => AdapterError {
            kind_tag: "error",
            kind: ErrorKind::Retryable,
            http_status: Some(*status),
            message: format!("rate-limited: {body}"),
            retry_after_seconds: parse_retry_after(body),
        },
        client::Error::Provider { status, body } => {
            let kind = if *status >= 500 {
                ErrorKind::Retryable
            } else {
                ErrorKind::Fatal
            };
            AdapterError {
                kind_tag: "error",
                kind,
                http_status: Some(*status),
                message: format!("provider: {body}"),
                retry_after_seconds: None,
            }
        }
        client::Error::Parse(err) => AdapterError::fatal(format!("upstream JSON parse: {err}")),
    }
}

/// Pull a `retry_after` field out of a JSON error body if present. Returns
/// `None` on any shape we don't recognize — the adapter never fabricates
/// retry hints.
fn parse_retry_after(body: &str) -> Option<u64> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    v.get("retry_after").and_then(|n| n.as_u64())
}

fn write_json<W: Write, T: Serialize>(out: &mut W, value: &T) -> std::io::Result<()> {
    serde_json::to_writer(&mut *out, value).map_err(std::io::Error::other)?;
    out.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests;
