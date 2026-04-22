//! On-disk shape of a v0.1 exchange record.
//!
//! Per ARCH §2.2, `exchanges/` is the post-merge compacted history of an
//! exchange. v0.1 does not branch or compact (§12 success criterion) — one
//! `lernie prompt` invocation writes a single JSON file directly to
//! `exchanges/` and commits it to `main`. This is the explicit v0.1
//! exception to the branch invariant (§2.3); v0.2 replaces it with the
//! branch-and-merge flow.
//!
//! Field names follow ARCH §2.1 terminology: `user_message` / `assistant_response`
//! (not "prompt" / "completion"), `model_id` (the string the provider uses
//! to route), `stop_reason` (Anthropic wire value, kept raw per `provider::anthropic`).

use serde::{Deserialize, Serialize};

/// Input and output token counts for one model call. Mirrors
/// [`crate::provider::anthropic::Usage`] but lives in this module so the
/// exchange schema does not leak a provider-specific type name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// One exchange written to `exchanges/<ts>-<short-id>.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExchangeRecord {
    pub user_message: String,
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
    fn round_trips_through_json() {
        let rec = ExchangeRecord {
            user_message: "hi".into(),
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
        let back: ExchangeRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(rec, back);
        // Field names are the contract: assert they are present on the wire.
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        for key in [
            "user_message",
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
    }
}
