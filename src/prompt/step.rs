//! On-disk layout for a step (ARCH §2.3 and §2.10).
//!
//! Within an exchange branch, each step lives in its own directory under
//! `exchanges/<exchange-id>/steps/<NNN>/`, zero-padded 3-digit and
//! 1-indexed. Two files land per step in v0.2:
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
//! v0.2 has exactly one step per exchange (no tools yet — §12). v0.3
//! extends the step dir with `tool_calls/<tool-id>/…` without moving
//! `request.json` / `response.json`, so this layout generalizes.

use serde::{Deserialize, Serialize};

/// Top-level directory holding per-exchange working areas on a branch
/// and compacted summaries on `main` post-merge (ARCH §2.2).
pub const EXCHANGES_DIR: &str = "exchanges";
/// Sub-directory inside an exchange that holds each step's on-disk
/// layout. The name is not a term of art — it mirrors §2.3's
/// "steps are linear commits" phrasing.
pub const STEPS_DIR: &str = "steps";
/// Model call input, committed BEFORE the model call (§2.10).
pub const REQUEST_FILE: &str = "request.json";
/// Model call output, committed AFTER the model call on the same branch.
pub const RESPONSE_FILE: &str = "response.json";

/// Width of the zero-padded step sequence in on-disk paths
/// (`steps/001`, `steps/002`, ...). Three digits gives comfortable
/// headroom for any realistic exchange while keeping directories
/// lexically sortable.
const STEP_SEQ_WIDTH: usize = 3;

/// The branch-relative directory for step `seq` within exchange
/// `exchange_id`. `seq` is 1-indexed; v0.2 always passes `1`.
pub fn step_dir_rel(exchange_id: &str, seq: u32) -> String {
    format!(
        "{EXCHANGES_DIR}/{exchange_id}/{STEPS_DIR}/{seq:0width$}",
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
/// long-term record — the compactor (later v0.2 work) reads this as a
/// stable contract.
///
/// `assistant_response` is the concatenated text of the response's text
/// blocks; per ARCH §2.1 `stop_reason` stays as the raw provider wire
/// string (one of Anthropic's values uses a banned term, and the
/// harness does not yet branch on it).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepResponse {
    pub assistant_response: String,
    pub model_id: String,
    pub provider: String,
    pub usage: Usage,
    pub stop_reason: String,
    pub started_at: String,
    pub ended_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_dir_rel_zero_pads_seq() {
        assert_eq!(
            step_dir_rel("20260422T000000Z-deadbeef", 1),
            "exchanges/20260422T000000Z-deadbeef/steps/001"
        );
        assert_eq!(step_dir_rel("id", 42), "exchanges/id/steps/042");
    }

    #[test]
    fn step_response_round_trips_and_publishes_stable_keys() {
        let rec = StepResponse {
            assistant_response: "hello".into(),
            model_id: "claude-sonnet-4-7".into(),
            provider: "anthropic".into(),
            usage: Usage {
                input_tokens: 3,
                output_tokens: 2,
            },
            stop_reason: "end_turn".into(),
            started_at: "2026-04-22T06:54:32Z".into(),
            ended_at: "2026-04-22T06:54:35Z".into(),
        };
        let json = serde_json::to_string(&rec).unwrap();
        let back: StepResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(rec, back);
        // Field names are the on-disk contract — assert they survive.
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "assistant_response",
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
    }
}
