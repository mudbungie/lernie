//! Anthropic SSE streaming: native wire-event parser and accumulator.
//!
//! This layer is the raw Anthropic-native streaming shape. Adapter-contract
//! streaming events (`docs/ARCHITECTURE.md` §4.4: `text_delta`,
//! `tool_use_delta`, etc.) are a later ball under the bl-d15d epic; the
//! adapter translates *from* the events here *to* those.
//!
//! The wire protocol is W3C Server-Sent Events over a `text/event-stream`
//! body: `data:`-prefixed lines carry a JSON payload, events are delimited
//! by blank lines. See <https://docs.anthropic.com/en/api/messages-streaming>.
//!
//! # Error mapping
//!
//! - Native `event: error` frames surface as [`Error::Provider`] (reusing
//!   the non-streaming error path; see client module docs).
//! - Malformed frames and truncated streams surface as [`Error::Sse`].

use super::{ContentBlock, Error, Response, Usage};
use serde_json::Value;
use std::io::BufRead;

/// One Anthropic-native SSE event. The payload shapes stay untyped
/// ([`Value`]) so adding wire fields upstream does not break this enum;
/// typed structure can come later when the adapter needs to branch on it.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// `event: message_start` — carries the initial Anthropic `message`
    /// object (id, model, empty content, starting usage).
    MessageStart { message: Value },
    /// `event: content_block_start` — carries the block index and the
    /// initial content-block payload.
    ContentBlockStart { index: u32, content_block: Value },
    /// `event: content_block_delta` — carries the block index and the
    /// delta payload (e.g. `{"type":"text_delta","text":"…"}`).
    ContentBlockDelta { index: u32, delta: Value },
    /// `event: content_block_stop` — carries the block index.
    ContentBlockStop { index: u32 },
    /// `event: message_delta` — carries top-level message updates
    /// (notably `stop_reason`) and a cumulative usage snapshot.
    MessageDelta { delta: Value, usage: Value },
    /// `event: message_stop` — terminal.
    MessageStop,
    /// `event: ping` — keep-alive, no payload of interest.
    Ping,
    /// Forward-compat catchall for event types this version does not
    /// recognize. Carries the full JSON so nothing is lost.
    Unknown(Value),
}

/// Iterator of [`Event`]s parsed from an SSE body.
///
/// Blocking: `next()` reads until a full event frame is assembled or the
/// underlying reader signals EOF. On a clean EOF between frames the stream
/// ends; on EOF mid-frame the stream yields one [`Error::Sse`] and then
/// ends.
#[derive(Debug)]
pub struct EventStream<R: BufRead> {
    reader: R,
    done: bool,
}

impl<R: BufRead> EventStream<R> {
    /// Wrap a reader as an event stream. Called by
    /// [`crate::Client::send_streaming`]; kept `pub` so tests and
    /// downstream consumers can parse cached SSE bodies from disk.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            done: false,
        }
    }
}

impl<R: BufRead> Iterator for EventStream<R> {
    type Item = Result<Event, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let mut data = String::new();
        let mut line = String::new();
        for _ in 0usize.. {
            line.clear();
            let n = match self.reader.read_line(&mut line) {
                Ok(n) => n,
                Err(e) => {
                    self.done = true;
                    return Some(Err(Error::Sse(format!("read error: {e}"))));
                }
            };
            if n == 0 {
                self.done = true;
                return if data.is_empty() {
                    None
                } else {
                    Some(Err(Error::Sse("stream ended mid-frame".into())))
                };
            }
            let trimmed = line.trim_end_matches(['\r', '\n']);
            if trimmed.is_empty() && !data.is_empty() {
                return Some(parse_event(&data));
            }
            if let Some(rest) = trimmed.strip_prefix("data:") {
                let value = rest.strip_prefix(' ').unwrap_or(rest);
                if !data.is_empty() {
                    data.push('\n');
                }
                data.push_str(value);
            }
            // `event:`, `id:`, `retry:`, comment lines (`:`), and stray
            // blank lines between events are ignored: the `data` JSON
            // carries `type`, which is authoritative.
        }
        unreachable!("0usize.. never ends; every exit returns")
    }
}

fn parse_event(data: &str) -> Result<Event, Error> {
    let value: Value =
        serde_json::from_str(data).map_err(|e| Error::Sse(format!("malformed event JSON: {e}")))?;
    let tag = value.get("type").and_then(Value::as_str).unwrap_or("");
    if tag == "error" {
        let body = serde_json::to_string(&value).unwrap_or_else(|_| data.to_string());
        return Err(Error::Provider { status: 200, body });
    }
    Ok(match tag {
        "message_start" => Event::MessageStart {
            message: take(&value, "message"),
        },
        "content_block_start" => Event::ContentBlockStart {
            index: index_of(&value),
            content_block: take(&value, "content_block"),
        },
        "content_block_delta" => Event::ContentBlockDelta {
            index: index_of(&value),
            delta: take(&value, "delta"),
        },
        "content_block_stop" => Event::ContentBlockStop {
            index: index_of(&value),
        },
        "message_delta" => Event::MessageDelta {
            delta: take(&value, "delta"),
            usage: take(&value, "usage"),
        },
        "message_stop" => Event::MessageStop,
        "ping" => Event::Ping,
        _ => Event::Unknown(value),
    })
}

fn take(v: &Value, key: &str) -> Value {
    v.get(key).cloned().unwrap_or(Value::Null)
}

fn index_of(v: &Value) -> u32 {
    v.get("index").and_then(Value::as_u64).unwrap_or(0) as u32
}

#[derive(Default)]
struct Acc {
    id: Option<String>,
    model: Option<String>,
    stop_reason: Option<String>,
    usage: Option<Usage>,
    blocks: Vec<Option<ContentBlock>>,
}

impl Acc {
    fn on_message_start(&mut self, message: &Value) {
        if let Some(s) = str_field(message, "id") {
            self.id = Some(s);
        }
        if let Some(s) = str_field(message, "model") {
            self.model = Some(s);
        }
        if let Some(s) = str_field(message, "stop_reason") {
            self.stop_reason = Some(s);
        }
        if let Some(u) = message.get("usage") {
            self.usage = Some(Usage {
                input_tokens: u32_field(u, "input_tokens"),
                output_tokens: u32_field(u, "output_tokens"),
            });
        }
    }

    fn on_block_start(&mut self, index: u32, payload: &Value) {
        *ensure_slot(&mut self.blocks, index as usize) = Some(initial_block(payload));
    }

    fn on_block_delta(&mut self, index: u32, delta: &Value) {
        if let Some(Some(b)) = self.blocks.get_mut(index as usize) {
            apply_delta(b, delta);
        }
    }

    fn on_message_delta(&mut self, delta: &Value, usage_delta: &Value) {
        if let Some(s) = str_field(delta, "stop_reason") {
            self.stop_reason = Some(s);
        }
        if let (Some(u), Some(out)) = (
            self.usage.as_mut(),
            usage_delta.get("output_tokens").and_then(Value::as_u64),
        ) {
            u.output_tokens = out as u32;
        }
    }

    fn into_response(self) -> Result<Response, Error> {
        let missing = |what: &str| Error::Sse(format!("missing {what}"));
        Ok(Response {
            id: self.id.ok_or_else(|| missing("id (no message_start)"))?,
            model: self.model.ok_or_else(|| missing("model"))?,
            stop_reason: self.stop_reason.ok_or_else(|| missing("stop_reason"))?,
            content: self.blocks.into_iter().flatten().collect(),
            usage: self.usage.ok_or_else(|| missing("usage"))?,
        })
    }
}

/// Fold a stream of [`Event`]s into the non-streaming [`Response`] shape.
///
/// Round-trips with the non-streaming parser: feeding the event stream
/// Anthropic produces for a given message yields a [`Response`] equal to
/// the body of the non-streaming `POST /v1/messages` for that same message.
///
/// Terminates on `message_stop` or end-of-iterator. Returns [`Error::Sse`]
/// if the stream lacks the fields needed to construct a [`Response`].
pub fn accumulate<I>(events: I) -> Result<Response, Error>
where
    I: IntoIterator<Item = Result<Event, Error>>,
{
    let mut acc = Acc::default();
    for event in events.into_iter().take_while(before_message_stop) {
        dispatch(&mut acc, event?);
    }
    acc.into_response()
}

fn before_message_stop(event: &Result<Event, Error>) -> bool {
    !matches!(event, Ok(Event::MessageStop))
}

/// One match arm per variant; each body is a single expression so
/// coverage instrumentation attaches cleanly per arm.
fn dispatch(acc: &mut Acc, event: Event) {
    match event {
        Event::MessageStart { message } => acc.on_message_start(&message),
        Event::ContentBlockStart {
            index,
            content_block,
        } => acc.on_block_start(index, &content_block),
        Event::ContentBlockDelta { index, delta } => acc.on_block_delta(index, &delta),
        Event::MessageDelta { delta, usage } => acc.on_message_delta(&delta, &usage),
        Event::MessageStop | Event::ContentBlockStop { .. } | Event::Ping | Event::Unknown(_) => {}
    }
}

fn ensure_slot(blocks: &mut Vec<Option<ContentBlock>>, index: usize) -> &mut Option<ContentBlock> {
    if blocks.len() <= index {
        blocks.resize_with(index + 1, || None);
    }
    &mut blocks[index]
}

fn initial_block(payload: &Value) -> ContentBlock {
    let kind = payload.get("type").and_then(Value::as_str).unwrap_or("");
    if kind == "text" {
        let text = payload
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        ContentBlock::Text { text }
    } else {
        ContentBlock::Unknown
    }
}

fn apply_delta(block: &mut ContentBlock, delta: &Value) {
    let kind = delta.get("type").and_then(Value::as_str).unwrap_or("");
    if let (ContentBlock::Text { text }, "text_delta") = (block, kind)
        && let Some(chunk) = delta.get("text").and_then(Value::as_str)
    {
        text.push_str(chunk);
    }
}

fn str_field(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_string)
}

fn u32_field(v: &Value, key: &str) -> u32 {
    v.get(key).and_then(Value::as_u64).unwrap_or(0) as u32
}

#[cfg(test)]
mod tests;
