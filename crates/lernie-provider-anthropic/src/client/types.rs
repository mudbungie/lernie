//! Anthropic Messages-API wire types.
//!
//! Lives in its own file so [`super`] (the HTTP-client surface) stays
//! under the repo's 300-line code-file cap. The split is structural:
//! these types model the wire shape (request, response, content blocks,
//! tool declarations) and are independent of the [`reqwest`]-based
//! transport in [`super`]. Tests for the types live alongside the
//! transport tests in `super::tests` since the request fixture is
//! shared.

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Author of a [`Message`] in a request.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// Polymorphic message-content shape. Anthropic's Messages API accepts
/// either a bare string (the v0.1/v0.2 shape — text-only message) or an
/// array of typed content blocks (v0.3 — required to mix `text` with
/// `tool_use` / `tool_result`). The untagged enum round-trips both.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

/// One message in the conversation history sent to the model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: MessageContent,
}

/// One entry in a request's `tools: [...]` array. Matches Anthropic's
/// Messages-API tool-declaration shape: name, free-text description (the
/// associated `SKILL.md`'s frontmatter description per ARCH §3.3), and
/// the JSON Schema for `tool_use.input`. The schema is held verbatim as
/// [`serde_json::Value`] — adapters do not interpret it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDecl {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Input to one model call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Request {
    pub model: String,
    pub max_tokens: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    pub messages: Vec<Message>,
    /// Tools the model is allowed to invoke this call. Omitted when the
    /// caller declares no tools — Anthropic rejects an empty `tools: []`,
    /// and `Option::is_none` skip-serialization keeps the wire clean.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDecl>>,
}

/// Usage accounting returned by the provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// One block of an assistant message or a user message's content. v0.3
/// adds `tool_use` (model emission) and `tool_result` (next-step
/// feedback), per ARCH §3.3. Unknown block types surface as
/// [`ContentBlock::Unknown`] so future provider-side additions do not
/// fail parsing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    /// Model's request to invoke a tool (TAXONOMY.md §2 maps this term
    /// to "tool call" in vendor-neutral prose). `id` is the wire id the
    /// matching `tool_result` echoes back; `input` is held as
    /// [`serde_json::Value`] so the harness does not interpret it.
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// User-side payload feeding tool output back to the model on the
    /// next step. `tool_use_id` matches the emission's `id`. The `content`
    /// field is a string in v0.3 (the harness wraps the tool's stdout as
    /// a string per ARCH §3.3 stdio contract); Anthropic also accepts an
    /// array shape, deferred to a later milestone if multimodal tool
    /// output lands. `is_error` defaults to `false` and is the exit-code
    /// projection per ARCH §3.3.
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default, skip_serializing_if = "is_false")]
        is_error: bool,
    },
    #[serde(other, skip_serializing)]
    Unknown,
}

/// `skip_serializing_if` predicate — keeps `is_error: false` off the
/// wire so `tool_result` blocks default-clean on success.
fn is_false(b: &bool) -> bool {
    !*b
}

/// Parsed Messages API response. `stop_reason` is kept as the raw wire
/// string (e.g. `"end_turn"`, `"max_tokens"`) rather than enumified here:
/// see `docs/ARCHITECTURE.md` §2.1 — one of Anthropic's wire values uses a
/// banned term, and the harness does not yet need to branch on it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
