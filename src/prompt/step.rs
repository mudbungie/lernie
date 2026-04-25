//! On-disk layout for a step (ARCH §2.3 and §2.10).
//!
//! Each step lives in its own directory under
//! `steps/<conv-id>/<NNN>/`, zero-padded 3-digit and 1-indexed.
//! Namespacing by conversation id (§2.2) is what lets a subagent's step
//! tree merge into its parent's worktree without filename collision.
//! Two files land per step in v0.3:
//!
//! - `request.json` — the model call's input, written and committed BEFORE
//!   the model call. Per §2.10 ("commit before model call"), this
//!   commit's tree is the exact snapshot the model call was derived
//!   from; retry replays the snapshot without drift.
//! - `response.json` — the model call's parsed output (text, usage,
//!   stop_reason, timing), written and committed AFTER the model call as
//!   a follow-up commit on the same branch. A follow-up commit (vs
//!   amending) is chosen so the snapshot commit's tree continues to
//!   reflect pre-model-call state, preserving §2.10's replay property.
//!
//! v0.3 ball #3 turns the step seq from a constant 1 into a loop
//! counter (§2.5): a step whose response carries `stop_reason:
//! "tool_use"` is followed by another step on the same branch under
//! `steps/<conv-id>/002/`, `…/003/`, etc. The loop terminates when
//! `stop_reason` is anything else.
//!
//! v0.4+ extends each step dir with `tools/<tool-id>/…` for the
//! per-call tool records (ball #4); the layout already accommodates
//! this without moving `request.json` / `response.json`.

use crate::provider::wire::ContentBlock;
use serde::{Deserialize, Serialize};

/// Top-level directory holding per-conversation step records on a branch
/// (ARCH §2.2).
pub const STEPS_DIR: &str = "steps";
/// Model call input, committed BEFORE the model call (§2.10).
pub const REQUEST_FILE: &str = "request.json";
/// Model call output, committed AFTER the model call on the same branch.
pub const RESPONSE_FILE: &str = "response.json";

/// Width of the zero-padded step sequence in on-disk paths
/// (`steps/<conv-id>/001`, `…/002`, ...). Three digits gives comfortable
/// headroom for any realistic conversation while keeping directories
/// lexically sortable.
const STEP_SEQ_WIDTH: usize = 3;

/// The branch-relative directory for step `seq` within conversation
/// `conv_id`. `seq` is 1-indexed; v0.3 always passes `1`.
pub fn step_dir_rel(conv_id: &str, seq: u32) -> String {
    format!(
        "{STEPS_DIR}/{conv_id}/{seq:0width$}",
        width = STEP_SEQ_WIDTH
    )
}

/// Input-token / output-token pair returned by the provider. Lives
/// alongside the step types because it is only ever read in the same
/// frame as a [`StepResponse`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// On-disk shape of `response.json`. Normalized to a harness-owned
/// schema so provider-specific wire fields do not leak into the
/// long-term record — the compactor reads this as a stable contract.
///
/// `content` is the structured assistant message — text + `tool_use`
/// blocks per ARCH §3.3. Storing it structurally (rather than
/// concatenated text) is what lets the next step's request assembly
/// surface tool-use emissions to the loop without re-deriving them
/// from prose. Use [`StepResponse::text`] when only the prose is
/// needed (e.g. the v0.3 stub compactor).
///
/// Per ARCH §2.1 `stop_reason` stays as the raw provider wire string
/// (one of Anthropic's values uses a banned term, and the harness does
/// not yet branch on it).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepResponse {
    pub content: Vec<ContentBlock>,
    pub model_id: String,
    pub provider: String,
    pub usage: Usage,
    pub stop_reason: String,
    pub started_at: String,
    pub ended_at: String,
}

impl StepResponse {
    /// Concatenated text of the response's [`ContentBlock::Text`] blocks,
    /// in order. Non-text blocks are skipped — `tool_use` survives in
    /// `content` for the loop to inspect, but is not part of the prose.
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

    #[test]
    fn step_dir_rel_zero_pads_seq() {
        assert_eq!(
            step_dir_rel("20260422T000000Z-deadbeef", 1),
            "steps/20260422T000000Z-deadbeef/001"
        );
        assert_eq!(step_dir_rel("id", 42), "steps/id/042");
    }

    #[test]
    fn step_response_round_trips_and_publishes_stable_keys() {
        let rec = StepResponse {
            content: vec![
                ContentBlock::Text {
                    text: "hello ".into(),
                },
                ContentBlock::ToolUse {
                    id: "toolu_01".into(),
                    name: "bash".into(),
                    input: serde_json::json!({"cmd": "ls"}),
                },
            ],
            model_id: "claude-sonnet-4-7".into(),
            provider: "anthropic".into(),
            usage: Usage {
                input_tokens: 3,
                output_tokens: 2,
            },
            stop_reason: "tool_use".into(),
            started_at: "2026-04-22T06:54:32Z".into(),
            ended_at: "2026-04-22T06:54:35Z".into(),
        };
        let json = serde_json::to_string(&rec).unwrap();
        let back: StepResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(rec, back);
        // Field names are the on-disk contract — assert they survive.
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "content",
            "model_id",
            "provider",
            "usage",
            "stop_reason",
            "started_at",
            "ended_at",
        ] {
            assert!(v.get(key).is_some(), "missing key: {key}");
        }
        assert_eq!(v["usage"]["input_tokens"], 3);
        // tool_use blocks survive structurally — the loop reads them
        // from response.json without re-parsing prose.
        assert_eq!(v["content"][1]["type"], "tool_use");
        assert_eq!(v["content"][1]["name"], "bash");
        // .text() folds over text blocks only.
        assert_eq!(rec.text(), "hello ");
    }
}
