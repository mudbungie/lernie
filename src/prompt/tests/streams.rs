//! brazen `v=1` NDJSON synthesis helpers for the harness test suite.
//!
//! Tests script the harness's model-call path by handing
//! [`super::fixtures::StubAdapter`] pre-baked NDJSON bytes — one
//! [`brazen::Event`] per `\n`-terminated line, exactly the shape `bz`
//! writes (§4.4). Events are constructed from the linked crate's typed
//! vocabulary and serialized, so the bytes can never drift from what the
//! assembler parses.
//!
//! Lives in its own file to keep `fixtures.rs` under the repo's 300-line
//! per-file cap.

use brazen::{CanonicalError, ContentKind, Delta, ErrorKind, Event, FinishReason, Role};
use serde_json::Value;

/// Parse an NDJSON byte buffer into one [`Value`] per non-empty line.
/// Used by tests that read `response.json` back to assert on-disk shape.
pub(super) fn parse_jsonl(bytes: &[u8]) -> Vec<Value> {
    bytes
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_slice(l).expect("each line is valid JSON"))
        .collect()
}

/// Serialize one event as an NDJSON line (trailing `\n`).
fn line(event: &Event) -> Vec<u8> {
    let mut out = serde_json::to_vec(event).expect("Event serializes");
    out.push(b'\n');
    out
}

/// A `Content` block description for [`stream_of`].
pub(super) enum Block<'a> {
    Text(&'a str),
    ToolUse {
        id: &'a str,
        name: &'a str,
        input: Value,
    },
}

/// Build a complete `v=1` segment: `message_start` → per-block
/// (`content_start`/`content_delta`/`content_stop`) → `usage` →
/// `finish{reason}` → `end`. The single terminal `end` makes the bytes a
/// self-delimiting attempt segment (§4.4).
pub(super) fn stream_of(reason: FinishReason, blocks: &[Block<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(line(&Event::message_start(
        Some("msg_x".into()),
        Some("claude-sonnet-5".into()),
        Role::Assistant,
    )));
    for (idx, block) in blocks.iter().enumerate() {
        let index = idx as u32;
        match block {
            Block::Text(text) => {
                out.extend(line(&Event::ContentStart {
                    index,
                    kind: ContentKind::Text {},
                }));
                out.extend(line(&Event::ContentDelta {
                    index,
                    delta: Delta::TextDelta((*text).to_string()),
                }));
            }
            Block::ToolUse { id, name, input } => {
                out.extend(line(&Event::ContentStart {
                    index,
                    kind: ContentKind::ToolUse {
                        id: (*id).to_string(),
                        name: (*name).to_string(),
                    },
                }));
                out.extend(line(&Event::ContentDelta {
                    index,
                    delta: Delta::JsonDelta(serde_json::to_string(input).unwrap()),
                }));
            }
        }
        out.extend(line(&Event::ContentStop { index }));
    }
    let mut usage = brazen::Usage::default();
    usage.input_tokens = Some(5);
    usage.output_tokens = Some(3);
    out.extend(line(&Event::Usage(usage)));
    out.extend(line(&Event::Finish { reason }));
    out.extend(line(&Event::End));
    out
}

/// A failed segment: `message_start` → `error{kind}` → `end`. brazen
/// always closes even a failed stream with one `end` (§4.4).
pub(super) fn error_stream(kind: ErrorKind, message: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(line(&Event::message_start(
        Some("msg_x".into()),
        Some("claude-sonnet-5".into()),
        Role::Assistant,
    )));
    out.extend(line(&Event::Error(CanonicalError {
        kind,
        message: message.to_string(),
        provider_detail: None,
        retry_after_seconds: None,
    })));
    out.extend(line(&Event::End));
    out
}

/// The off-the-shelf happy stream: one `"hi there"` text block, finish
/// `stop`. Tests that need *any* successful completion pull this.
pub(super) fn happy_response_bytes() -> Vec<u8> {
    stream_of(FinishReason::Stop, &[Block::Text("hi there")])
}
