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
//! - `response.json` — JSONL of §4.4 stream events, appended by the
//!   harness as the adapter writes them. Writer-closes-fd is the
//!   `IN_CLOSE_WRITE` end-of-stream signal (§3.5). Diagnostic-only;
//!   the harness never reads it back (§2.3).
//! - `tools/<tool-id>/` — per-tool-call records (`input.json`,
//!   `output.json`); the harness *does* read `output.json` to
//!   assemble the next step's `tool_result` blocks (§3.3).
//! - `assistant.staging.json` — the transcript entry under
//!   construction (§2.3 *The transcript writer*): the writer's own
//!   sink, not a diagnostic record, renamed out to the worktree at
//!   the model call's settling `Finish`.

use serde::{Deserialize, Serialize};

/// Top-level directory holding per-conversation step records, located
/// at the conversation-repo root outside every worktree (ARCH §2.2 /
/// §2.3). Joined onto the conv-repo path by writers, never the
/// worktree path.
pub const STEPS_DIR: &str = "steps";
/// Diagnostic snapshot of the wire request the model saw. Written
/// for audit only — harness never reads at runtime (§2.3).
pub const REQUEST_FILE: &str = "request.json";
/// JSONL of §4.4 stream events, written event-by-event by the harness
/// as the adapter emits them. End-of-stream is the writer closing the
/// fd (§3.5 IN_CLOSE_WRITE). Diagnostic-only; harness never reads it
/// back (§2.3).
pub const RESPONSE_FILE: &str = "response.json";
/// Step metadata: branch-tip sha at step-start plus timestamps
/// (§2.3). Readable by the harness — it carries the commit a
/// replay re-assembles against, which is the load-bearing piece.
pub const META_FILE: &str = "meta.json";
/// The assistant transcript entry *under construction* (ARCH §2.3 *The
/// transcript writer*). Content blocks stream here block-by-block as a
/// JSON array; segment authority (§4.4) truncates it on an `Error`
/// segment, accumulates it on `Pause`, and the final `Finish` seals it,
/// whereupon the executor renames it into the worktree as
/// `messages/NNN-assistant.json` (§2.3). The one path under `steps/`
/// that is not a diagnostic record — the writer's own sink, never read
/// back as a step record (§2.3 Diagnostic-only contract).
pub const STAGING_FILE: &str = "assistant.staging.json";

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
}
