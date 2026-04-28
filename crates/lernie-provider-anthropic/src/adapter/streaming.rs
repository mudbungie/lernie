//! Adapter-side streaming wire (`docs/ARCHITECTURE.md` §4.4 "Response
//! shape (streaming)").
//!
//! Translates Anthropic's native SSE events ([`crate::client::Event`])
//! into the §4.4 normalized JSON-Lines event stream the harness reads.
//! The translation is a one-pass scan over the iterator: text/tool-use
//! deltas flatten content_block_delta into top-level `text_delta` /
//! `tool_use_delta` events; `message_delta` is folded into the terminal
//! `message_stop` (which carries `usage` + `api_calls`); `ping` is
//! dropped per the task scope; unknown event types are dropped
//! (forward-compat).
//!
//! Errors land as a single terminal `{"type":"error",...}` event with
//! the same shape as the non-streaming error object — same kinds, same
//! `http_status`, same `retry_after_seconds`. SIGTERM (signaled via the
//! shared [`AtomicBool`]) flushes a retryable terminal `error` event.
//!
//! **Concurrency model.** Synchronous and blocking. Each event is
//! written as it arrives, then the loop waits on the next iterator
//! item; no async runtime, no buffering layer. The harness's
//! consumption-side commit cadence is undecided at v0.2 — the harness
//! does not yet read the streaming output (the v0.5 UI ball is the
//! first consumer). This module emits one JSON line per event so any
//! future commit-cadence policy lives wholly on the harness side.

use super::{AdapterError, write_json};
use crate::client::streaming::Event;
use crate::client::{self, Client, Request};
use serde::Serialize;
use serde_json::Value;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

/// Always 1 in v0.2: the adapter issues exactly one HTTP request (one
/// API call per ARCH §2.1) per `complete` invocation — no retry yet.
/// Retry/reconnect work in a later ball will increment this per
/// attempt.
const API_CALLS_V0_2: u32 = 1;

/// Drive a streaming `complete`. Writes JSON Lines events to `out` and
/// returns `Ok(())` once the stream terminates (success, error, or
/// SIGTERM). Stream-level errors land as in-band `error` events; per
/// §4.4 the process still exits `0`.
pub(crate) fn run<W: Write>(
    out: &mut W,
    client: &Client,
    request: &Request,
    stop: &AtomicBool,
) -> std::io::Result<()> {
    if stop.load(Ordering::Acquire) {
        return write_json(out, &interrupted_error());
    }
    let stream = match client.send_streaming(request) {
        Ok(s) => s,
        Err(e) => return write_json(out, &super::map_error(&e)),
    };
    drain(out, stream, stop)
}

/// Iterator-driven core of [`run`]. Split out so unit tests can inject
/// any `Iterator<Item = Result<Event, client::Error>>` and exercise the
/// SIGTERM checkpoint without a live HTTP stream. A flipped stop flag is
/// detected at the head of every loop iteration; an iter `Err` arriving
/// at the same time as a stop will be observed as `interrupted` because
/// the early check fires before the match body.
pub(crate) fn drain<W, I>(out: &mut W, events: I, stop: &AtomicBool) -> std::io::Result<()>
where
    W: Write,
    I: IntoIterator<Item = Result<Event, client::Error>>,
{
    let mut state = TerminalState::default();
    for event in events {
        if stop.load(Ordering::Acquire) {
            return write_json(out, &interrupted_error());
        }
        match event {
            Ok(ev) => {
                handle(out, &mut state, ev)?;
                if state.emitted_terminator {
                    return Ok(());
                }
            }
            Err(e) => return write_json(out, &super::map_error(&e)),
        }
    }
    // Stream ended without `message_stop` and without an error. Per
    // §4.4 the harness must always see a terminator — emit a fatal
    // so a half-stream is visible rather than silently dropped.
    write_json(out, &half_stream_fatal())
}

/// Terminal `error` event for the "iterator drained without seeing
/// `message_stop`" case. Split out so [`drain`] stays narrow and the
/// path is reachable from a unit test that injects a custom iterator.
fn half_stream_fatal() -> AdapterError {
    AdapterError::fatal("stream ended without message_stop")
}

/// Per-stream accumulator. Holds the values folded from `message_start`
/// (initial usage) and `message_delta` (final stop_reason, cumulative
/// output_tokens) so the terminal `message_stop` event can carry them.
#[derive(Default)]
struct TerminalState {
    starting_usage: Option<Value>,
    stop_reason: Option<String>,
    output_tokens: Option<u64>,
    emitted_terminator: bool,
}

#[rustfmt::skip]
fn handle<W: Write>(
    out: &mut W,
    state: &mut TerminalState,
    event: Event,
) -> std::io::Result<()> {
    match event {
        Event::MessageStart { message } => {
            state.starting_usage = message.get("usage").cloned();
            write_json(out, &Wire::MessageStart { message: &message })
        }
        Event::ContentBlockStart { index, content_block } => {
            let wire = Wire::ContentBlockStart { index, content_block: &content_block };
            write_json(out, &wire)
        }
        Event::ContentBlockDelta { index, delta } => emit_delta(out, index, &delta),
        Event::ContentBlockStop { index } => write_json(out, &Wire::ContentBlockStop { index }),
        Event::MessageDelta { delta, usage } => {
            if let Some(s) = delta.get("stop_reason").and_then(Value::as_str) {
                state.stop_reason = Some(s.to_string());
            }
            if let Some(n) = usage.get("output_tokens").and_then(Value::as_u64) {
                state.output_tokens = Some(n);
            }
            Ok(())
        }
        Event::MessageStop => { state.emitted_terminator = true; write_json(out, &message_stop(state)) }
        // `ping` is intentionally not part of the normalized event set
        // (§4.4); drop it. Same for forward-compat `Unknown` payloads.
        Event::Ping | Event::Unknown(_) => Ok(()),
    }
}

/// Translate Anthropic's `content_block_delta.delta` shape (`text_delta`
/// or `input_json_delta`) into the §4.4 normalized top-level `text_delta`
/// / `tool_use_delta` events. Unknown delta kinds are dropped.
#[rustfmt::skip]
fn emit_delta<W: Write>(out: &mut W, index: u32, delta: &Value) -> std::io::Result<()> {
    match delta.get("type").and_then(Value::as_str) {
        Some("text_delta") => {
            let text = delta.get("text").and_then(Value::as_str).unwrap_or("");
            write_json(out, &Wire::TextDelta { index, text })
        }
        Some("input_json_delta") => {
            let partial_json = delta.get("partial_json").and_then(Value::as_str);
            write_json(out, &Wire::ToolUseDelta { index, partial_json })
        }
        _ => Ok(()),
    }
}

/// Build the terminal `message_stop` payload from accumulated state.
/// Splits out so the [`handle`] arm stays narrow and the JSON shape is
/// straightforward to scan.
fn message_stop<'a>(state: &'a TerminalState) -> Wire<'a> {
    let usage = match state.starting_usage.as_ref() {
        Some(Value::Object(map)) => {
            let mut merged = map.clone();
            if let Some(n) = state.output_tokens {
                merged.insert("output_tokens".into(), Value::from(n));
            }
            Value::Object(merged)
        }
        Some(other) => other.clone(),
        None => Value::Object(serde_json::Map::new()),
    };
    Wire::MessageStop {
        stop_reason: state.stop_reason.as_deref(),
        usage,
        api_calls: API_CALLS_V0_2,
    }
}

/// SIGTERM-induced terminal error. Retryable so the harness re-issues
/// the model call rather than failing the conversation.
fn interrupted_error() -> AdapterError {
    AdapterError::retryable("interrupted by SIGTERM")
}

/// Adapter-side serialization of the §4.4 event variants. Uses
/// borrowed payloads so each emitted line is one allocation, not two.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Wire<'a> {
    MessageStart {
        message: &'a Value,
    },
    ContentBlockStart {
        index: u32,
        content_block: &'a Value,
    },
    TextDelta {
        index: u32,
        text: &'a str,
    },
    ToolUseDelta {
        index: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        partial_json: Option<&'a str>,
    },
    ContentBlockStop {
        index: u32,
    },
    MessageStop {
        #[serde(skip_serializing_if = "Option::is_none")]
        stop_reason: Option<&'a str>,
        usage: Value,
        api_calls: u32,
    },
}

#[cfg(test)]
mod tests;
