//! On-disk layout for a step (ARCH §2.3 and §2.10).
//!
//! Each step lives in its own directory under
//! `<conv-repo>/steps/<conv-id>/<NNN>/`, zero-padded 3-digit and
//! 1-indexed. The tree is at the conversation-repo root, *outside
//! every worktree* (§2.2 / §2.3), so context assembly (§3.5, §5)
//! cannot see step records as model context. Namespacing by
//! conversation id is what lets every conversation in the tree
//! (root and every subagent) write into a single shared `steps/`
//! tree without filename collision.
//!
//! Per-step files in v0.3.1+:
//!
//! - `meta.json` — `{commit, started_at, ended_at}`. The `commit`
//!   field is the sha of the branch tip at step-start; replay
//!   reproduces the wire input by re-running the context assembler
//!   (§5) against this commit's tree (§2.10).
//! - `request.json` — diagnostic snapshot of the wire request the
//!   model saw. Written for audit / human inspection only; the
//!   harness never reads it at runtime (§2.3 Diagnostic-only contract).
//! - `response.json` — the parsed model-call output. Written for the
//!   same diagnostic reasons; not read at runtime by the harness.
//!   v0.3.1 P3 reshapes this to a JSONL stream of §4.4 events.
//! - `tools/<tool-id>/` — per-tool-call records (`input.json`,
//!   `output.json`); the harness *does* read `output.json` to
//!   assemble the next step's `tool_result` blocks (§3.3).

use crate::provider::wire::ContentBlock;
use serde::{Deserialize, Serialize};

/// Top-level directory holding per-conversation step records, located
/// at the conversation-repo root outside every worktree (ARCH §2.2 /
/// §2.3). Joined onto the conv-repo path by writers, never the
/// worktree path.
pub const STEPS_DIR: &str = "steps";
/// Diagnostic snapshot of the wire request the model saw. Written
/// for audit only — harness never reads at runtime (§2.3).
pub const REQUEST_FILE: &str = "request.json";
/// Diagnostic snapshot of the parsed model-call output. Written for
/// audit only — harness never reads at runtime (§2.3). v0.3.1 P3
/// reshapes this to JSONL stream events.
pub const RESPONSE_FILE: &str = "response.json";
/// Step metadata: branch-tip sha at step-start plus timestamps
/// (§2.3). Readable by the harness — it carries the commit a
/// replay re-assembles against, which is the load-bearing piece.
pub const META_FILE: &str = "meta.json";

/// Width of the zero-padded step sequence in on-disk paths
/// (`steps/<conv-id>/001`, `…/002`, ...). Three digits gives comfortable
/// headroom for any realistic conversation while keeping directories
/// lexically sortable.
const STEP_SEQ_WIDTH: usize = 3;

/// The conv-repo-relative directory for step `seq` within conversation
/// `conv_id`. `seq` is 1-indexed. Joined onto the conv-repo root
/// (not any worktree) — step records live outside every worktree
/// per ARCH §2.2 / §2.3.
pub fn step_dir_rel(conv_id: &str, seq: u32) -> String {
    format!(
        "{STEPS_DIR}/{conv_id}/{seq:0width$}",
        width = STEP_SEQ_WIDTH
    )
}

/// On-disk shape of `meta.json`. The `commit` field is the branch
/// tip's sha at step-start — the read state for the model call
/// (§2.10). `started_at` / `ended_at` bookend the call's wall-clock
/// duration. Replay tooling reads `commit` to locate the tree state
/// the request was assembled against.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepMeta {
    pub commit: String,
    pub started_at: String,
    pub ended_at: String,
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
/// long-term record. Diagnostic-only per §2.3 — the harness does not
/// read this back at runtime.
///
/// `content` is the structured assistant message — text + `tool_use`
/// blocks per ARCH §3.3. Storing it structurally (rather than
/// concatenated text) preserves per-block fidelity for human
/// inspection.
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
    /// `content` for inspection, but is not part of the prose.
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
    fn step_meta_round_trips_and_publishes_stable_keys() {
        let m = StepMeta {
            commit: "0123456789abcdef0123456789abcdef01234567".into(),
            started_at: "2026-04-22T06:54:32Z".into(),
            ended_at: "2026-04-22T06:54:35Z".into(),
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: StepMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in ["commit", "started_at", "ended_at"] {
            assert!(v.get(key).is_some(), "missing key: {key}");
        }
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
        // tool_use blocks survive structurally.
        assert_eq!(v["content"][1]["type"], "tool_use");
        assert_eq!(v["content"][1]["name"], "bash");
        // .text() folds over text blocks only.
        assert_eq!(rec.text(), "hello ");
    }
}
