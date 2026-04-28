//! In-memory assembly of §4.4 stream events.
//!
//! The harness drives `complete` with `stream: true` and tails the
//! adapter's stdout line by line (ARCH §4.4 "On-disk response shape:
//! JSONL of stream events, always."). Each line is appended to
//! `<conv-repo>/steps/<conv-id>/<NNN>/response.json` for diagnostics
//! (§2.3) and fed through this module to build the *in-memory* shape
//! the next step's wire request needs:
//!
//! - text deltas accumulate into a [`ContentBlock::Text`];
//! - `tool_use` content blocks fold their `input_json_delta`
//!   `partial_json` fragments into a single JSON string, parsed once
//!   on stop into the wire `input` value;
//! - `message_stop` carries the terminal `usage` and `stop_reason`;
//! - an in-band `error` event terminates the stream with the same
//!   shape as the non-streaming adapter error.
//!
//! Per the diagnostic-only contract (§2.3) the harness never reads
//! `response.json` back at runtime — every harness consumer of the
//! model's output goes through this in-memory accumulator.

use crate::provider::wire::{ContentBlock, StreamEvent};
use serde_json::Value;

/// Accumulator state. Built up over a sequence of [`StreamEvent`]s
/// (one per JSONL line). [`Self::is_terminal`] flips on `message_stop`
/// or in-band `error`; [`Self::into_completion`] consumes the final
/// state and returns either the assembled message or a typed error.
///
/// Usage / api_calls / cache fields ride into `response.json` via the
/// JSONL events themselves (§4.4); the assembler does not re-surface
/// them in [`Completion`] — the harness has no runtime consumer for
/// per-step usage in v0.3.1, and keeping the in-memory shape narrow
/// matches "Don't add abstractions beyond what the task requires."
pub(super) struct Assembler {
    blocks: Vec<Option<BlockState>>,
    stop_reason: Option<String>,
    terminal: Option<Terminal>,
}

/// One in-flight content block. We hold the raw partial-JSON buffer
/// for tool_use blocks (joined `partial_json` fragments from
/// `input_json_delta`) and parse on completion — invalid JSON surfaces
/// as [`AssemblyError::ToolInputJson`].
enum BlockState {
    Text(String),
    ToolUse {
        id: String,
        name: String,
        partial_json: String,
    },
    Other,
}

/// Terminal state of a stream: clean stop, or in-band error event.
enum Terminal {
    Stop,
    Error {
        kind: String,
        message: String,
        http_status: Option<u16>,
    },
}

/// Assembled message ready to feed back into the harness step loop.
pub(super) struct Completion {
    pub(super) content: Vec<ContentBlock>,
    pub(super) stop_reason: String,
}

/// Failure modes specific to event assembly. Lifted into the prompt
/// `Error` enum at the dispatch boundary.
pub(super) enum AssemblyError {
    /// Stream ended without `message_stop` or an `error` event.
    HalfStream,
    /// In-band `error` event from the adapter (§4.4 "Errors").
    Adapter {
        kind: String,
        message: String,
        http_status: Option<u16>,
    },
    /// `message_stop` arrived without a `stop_reason` (neither in
    /// `message_start.message`, in `message_delta`, nor on the stop
    /// event itself). The wire contract guarantees one of those slots
    /// carries it, so a missing value is an adapter contract bug.
    MissingStopReason,
    /// A tool_use block's accumulated `partial_json` did not parse as
    /// valid JSON.
    ToolInputJson(serde_json::Error),
}

impl Assembler {
    pub(super) fn new() -> Self {
        Self {
            blocks: Vec::new(),
            stop_reason: None,
            terminal: None,
        }
    }

    /// Fold one event into the accumulator. Idempotent w.r.t. already
    /// being terminal — events arriving after `message_stop` (rare,
    /// mostly defensive) are ignored.
    pub(super) fn feed(&mut self, event: StreamEvent) {
        if self.terminal.is_some() {
            return;
        }
        match event {
            StreamEvent::MessageStart { message } => self.on_message_start(&message),
            StreamEvent::ContentBlockStart {
                index,
                content_block,
            } => {
                let state = initial_block(&content_block);
                set_slot(&mut self.blocks, index as usize, state);
            }
            StreamEvent::TextDelta { index, text } => {
                if let Some(Some(BlockState::Text(buf))) = self.blocks.get_mut(index as usize) {
                    buf.push_str(&text);
                }
            }
            StreamEvent::ToolUseDelta {
                index,
                partial_json,
            } => {
                if let (
                    Some(json),
                    Some(Some(BlockState::ToolUse {
                        partial_json: buf, ..
                    })),
                ) = (partial_json, self.blocks.get_mut(index as usize))
                {
                    buf.push_str(&json);
                }
            }
            StreamEvent::ContentBlockStop { .. } => {}
            StreamEvent::MessageStop {
                stop_reason,
                usage: _,
                api_calls: _,
            } => {
                if stop_reason.is_some() {
                    self.stop_reason = stop_reason;
                }
                self.terminal = Some(Terminal::Stop);
            }
            StreamEvent::Error {
                kind,
                message,
                http_status,
                ..
            } => {
                self.terminal = Some(Terminal::Error {
                    kind,
                    message,
                    http_status,
                });
            }
        }
    }

    fn on_message_start(&mut self, message: &Value) {
        if let Some(stop) = message.get("stop_reason").and_then(Value::as_str) {
            self.stop_reason = Some(stop.to_string());
        }
    }

    pub(super) fn is_terminal(&self) -> bool {
        self.terminal.is_some()
    }

    pub(super) fn into_completion(self) -> Result<Completion, AssemblyError> {
        match self.terminal {
            None => Err(AssemblyError::HalfStream),
            Some(Terminal::Error {
                kind,
                message,
                http_status,
            }) => Err(AssemblyError::Adapter {
                kind,
                message,
                http_status,
            }),
            Some(Terminal::Stop) => {
                let stop_reason = self.stop_reason.ok_or(AssemblyError::MissingStopReason)?;
                let mut content = Vec::new();
                for state in self.blocks.into_iter().flatten() {
                    content.push(finalize_block(state)?);
                }
                Ok(Completion {
                    content,
                    stop_reason,
                })
            }
        }
    }
}

fn initial_block(payload: &Value) -> BlockState {
    match payload.get("type").and_then(Value::as_str) {
        Some("text") => BlockState::Text(
            payload
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        ),
        Some("tool_use") => BlockState::ToolUse {
            id: str_at(payload, "id"),
            name: str_at(payload, "name"),
            partial_json: String::new(),
        },
        _ => BlockState::Other,
    }
}

fn finalize_block(state: BlockState) -> Result<ContentBlock, AssemblyError> {
    Ok(match state {
        BlockState::Text(text) => ContentBlock::Text { text },
        BlockState::ToolUse {
            id,
            name,
            partial_json,
        } => {
            let input: Value = if partial_json.is_empty() {
                Value::Object(serde_json::Map::new())
            } else {
                serde_json::from_str(&partial_json).map_err(AssemblyError::ToolInputJson)?
            };
            ContentBlock::ToolUse { id, name, input }
        }
        BlockState::Other => ContentBlock::Unknown,
    })
}

fn set_slot(blocks: &mut Vec<Option<BlockState>>, index: usize, state: BlockState) {
    if blocks.len() <= index {
        blocks.resize_with(index + 1, || None);
    }
    blocks[index] = Some(state);
}

fn str_at(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

#[cfg(test)]
mod tests;
