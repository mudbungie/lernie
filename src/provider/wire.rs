//! Adapter-contract response shapes (non-streaming and streaming).
//!
//! Per `docs/ARCHITECTURE.md` §4.4 the adapter writes one of two shapes
//! to stdout, chosen by the request's `stream` field:
//!
//! - **Non-streaming**: a single JSON object conforming to the Anthropic
//!   Messages-API wire shape: `{ id, model, stop_reason, content[], usage }`.
//!   See [`Response`].
//! - **Streaming**: a JSON Lines event stream of normalized events
//!   (`message_start`, `content_block_start`, `text_delta`,
//!   `tool_use_delta`, `content_block_stop`, `message_stop`). See
//!   [`StreamEvent`]. The terminal `message_stop` carries `usage` and
//!   `api_calls`; an in-band `error` event closes a failed stream.
//!
//! These types are pinned on the harness side, independent of any
//! specific provider's client crate — so the harness never takes a
//! library dependency on a provider implementation
//! (`docs/PRINCIPLES.md` "Integrations are external binaries").
//!
//! Non-Anthropic adapters translate their provider's native response
//! into these shapes before writing; the harness does not care which
//! provider produced the bytes.

pub mod stream;

pub use stream::StreamEvent;

use serde::{Deserialize, Serialize};

/// One block of the model's output. v0.1 only handles `text`; unknown
/// block types parse into [`ContentBlock::Unknown`] so future provider
/// additions (e.g. `tool_use`, `thinking`) do not break the parse path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    #[serde(other, skip_serializing)]
    Unknown,
}

/// Usage accounting on the non-streaming response. `input_tokens` and
/// `output_tokens` are required per §4.4. Prompt-caching fields, when
/// present, MUST use Anthropic's native names
/// (`cache_creation_input_tokens`, `cache_read_input_tokens`); the
/// harness does not yet branch on them, so they are accepted and
/// ignored at parse time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Adapter-contract non-streaming response. `stop_reason` is kept as
/// the raw wire string (e.g. `"end_turn"`, `"max_tokens"`) — §2.1 lists
/// some Anthropic wire values under a banned term, and the harness
/// does not yet need to branch on them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Response {
    pub id: String,
    pub model: String,
    pub stop_reason: String,
    pub content: Vec<ContentBlock>,
    pub usage: Usage,
}

impl Response {
    /// Concatenated text from all [`ContentBlock::Text`] blocks, in
    /// order. Non-text blocks are skipped. The harness uses this when
    /// it wants the assistant's textual reply as a single string
    /// (e.g. to write to `response.json`'s `assistant_response`).
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

#[cfg(test)]
mod tests {
    use super::*;

    const HAPPY: &str = r#"{
        "id":"msg_01","model":"claude-sonnet-4-7","stop_reason":"end_turn",
        "content":[{"type":"text","text":"hi there"}],
        "usage":{"input_tokens":3,"output_tokens":2}
    }"#;

    #[test]
    fn parses_happy_shape() {
        let r: Response = serde_json::from_str(HAPPY).unwrap();
        assert_eq!(r.id, "msg_01");
        assert_eq!(r.model, "claude-sonnet-4-7");
        assert_eq!(r.stop_reason, "end_turn");
        assert_eq!(r.usage.input_tokens, 3);
        assert_eq!(r.usage.output_tokens, 2);
        assert_eq!(r.content.len(), 1);
        assert!(matches!(r.content[0], ContentBlock::Text { .. }));
    }

    #[test]
    fn text_concatenates_text_blocks_in_order_and_skips_others() {
        let body = r#"{
            "id":"x","model":"m","stop_reason":"end_turn",
            "content":[
                {"type":"text","text":"hello "},
                {"type":"tool_use","id":"t1","name":"bash","input":{}},
                {"type":"text","text":"world"}
            ],
            "usage":{"input_tokens":1,"output_tokens":1}
        }"#;
        let r: Response = serde_json::from_str(body).unwrap();
        assert_eq!(r.text(), "hello world");
        assert!(matches!(r.content[1], ContentBlock::Unknown));
    }

    #[test]
    fn unknown_cache_fields_are_accepted_and_ignored() {
        // Anthropic-native cache field names must not break parsing;
        // the harness doesn't surface them yet but adapters are
        // permitted (per §4.4) to pass them through.
        let body = r#"{
            "id":"x","model":"m","stop_reason":"end_turn",
            "content":[],
            "usage":{
                "input_tokens":1,"output_tokens":1,
                "cache_creation_input_tokens":2,"cache_read_input_tokens":3
            }
        }"#;
        let r: Response = serde_json::from_str(body).unwrap();
        assert_eq!(r.usage.input_tokens, 1);
    }

    #[test]
    fn missing_required_fields_are_rejected() {
        // One representative negative case per required top-level field
        // and usage.input_tokens; the full matrix is pinned by
        // `prompt::tests::errors_parse`.
        let cases = [
            r#"{"model":"m","stop_reason":"s","content":[],"usage":{"input_tokens":1,"output_tokens":1}}"#,
            r#"{"id":"x","stop_reason":"s","content":[],"usage":{"input_tokens":1,"output_tokens":1}}"#,
            r#"{"id":"x","model":"m","content":[],"usage":{"input_tokens":1,"output_tokens":1}}"#,
            r#"{"id":"x","model":"m","stop_reason":"s","usage":{"input_tokens":1,"output_tokens":1}}"#,
            r#"{"id":"x","model":"m","stop_reason":"s","content":[]}"#,
            r#"{"id":"x","model":"m","stop_reason":"s","content":[],"usage":{"output_tokens":1}}"#,
        ];
        for body in cases {
            assert!(
                serde_json::from_str::<Response>(body).is_err(),
                "expected parse failure for: {body}"
            );
        }
    }

    #[test]
    fn round_trips_through_serde_json() {
        let r: Response = serde_json::from_str(HAPPY).unwrap();
        let bytes = serde_json::to_vec(&r).unwrap();
        let r2: Response = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(r, r2);
    }
}
