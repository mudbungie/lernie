//! Branch-state classifier (ARCH §2.9 / §3.5 / §7.1).
//!
//! The four states the live view renders are derived from refs and the
//! JSONL terminal event in `response.json`, not from any sidecar marker
//! (PRINCIPLES.md "Single source of truth"). For an unmerged branch row
//! (the in-flight section's rows), the classifier reads the latest
//! step's `response.json` and returns:
//!
//! - [`BranchState::InFlight`] when no §4.4 terminal event is present —
//!   the writer is still appending, or the response file is absent
//!   pre-first-event.
//! - [`BranchState::Stopped`] when the last JSONL line is a §4.4
//!   terminal event. Per ARCH §2.9 (post bl-de6b amendment), kill /
//!   crash / explicit user stop are indistinguishable on disk; for root
//!   conversations (which do not merge back per §2.3 step 5) `Stopped`
//!   is the natural terminal state regardless of how the chain ended. A
//!   *complete* and a *failed* step both settle here (§3.5 — a failed
//!   step renders as stopped with the error surfaced); the badge does
//!   not split them, so this classifier only asks "is there a terminal
//!   line?".
//!
//! **Dual vocabulary (v0.6 transition, bl-507a).** A terminal line is
//! recognized in either vocabulary: the legacy v0.3 `message_stop` /
//! `error`, or brazen's `v=1` terminator `{"type":"end"}` (§4.4). The
//! follow-on ball (bl-56ee) drops the legacy tokens; until then both are
//! accepted so the writer swap merges green.
//!
//! [`BranchState::Merged`] and [`BranchState::Conflicted`] are part of
//! the type for renderer completeness but are not produced by this
//! classifier — `Merged` shows up as a `--no-ff` merge node on `main`'s
//! first-parent log (§2.3, rendered by the existing trunk path), and
//! `Conflicted` requires a subagent merge attempt which doesn't ship
//! until v0.4 (§2.6 step 6).

use std::path::Path;

use super::STEPS_DIR;
use super::streaming::latest_step_dir;

const RESPONSE_FILE: &str = "response.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchState {
    /// Branch is reachable from `main`'s HEAD (§2.3). Not produced by
    /// the unmerged-branch classifier; reserved for future renderers
    /// that want an explicit badge on trunk merge nodes.
    Merged,
    /// Latest step's `response.json` is still being written: file is
    /// absent pre-first-event, or its last JSONL line is not a §4.4
    /// terminal event.
    InFlight,
    /// Latest step's `response.json` ended in a §4.4 terminal event
    /// (`message_stop` or `error`). The chain is no longer advancing.
    Stopped,
    /// Subagent merge attempt conflicted (§2.6 step 6). Not reachable
    /// in v0.5 (no subagent merges ship until v0.4). Retained so
    /// renderers handle the variant once it arrives.
    Conflicted,
}

/// Classify an unmerged conversation branch by reading the latest
/// step's `response.json` from disk. Always returns `InFlight` or
/// `Stopped` — `Merged` and `Conflicted` are produced elsewhere (or
/// not at all, in v0.5).
pub(super) fn classify_unmerged(conv_repo: &Path, conv_id: &str) -> BranchState {
    let conv_steps = conv_repo.join(STEPS_DIR).join(conv_id);
    let Some(latest) = latest_step_dir(&conv_steps) else {
        return BranchState::InFlight;
    };
    let bytes = match std::fs::read(latest.join(RESPONSE_FILE)) {
        Ok(b) => b,
        Err(_) => return BranchState::InFlight,
    };
    if has_terminal_event(&bytes) {
        BranchState::Stopped
    } else {
        BranchState::InFlight
    }
}

/// Last completed JSONL line is a §4.4 terminal event. Mid-write
/// tolerance: a trailing line with no `\n` yet is dropped (the writer is
/// still appending it), and only the most recent fully-terminated line
/// is examined — mirroring the streaming accumulator's stance.
fn has_terminal_event(bytes: &[u8]) -> bool {
    // Trim a trailing partial line: drop everything after the last `\n`.
    // A buffer ending in `\n` is left intact; one ending mid-line is
    // truncated to the last newline boundary.
    let terminated = match bytes.iter().rposition(|&b| b == b'\n') {
        Some(idx) => &bytes[..=idx],
        None => return false,
    };
    let Some(line) = terminated
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .next_back()
    else {
        return false;
    };
    let Ok(value): Result<serde_json::Value, _> = serde_json::from_slice(line) else {
        return false;
    };
    // Dual-vocabulary terminal set (bl-507a): brazen's `v=1` terminator
    // is `end`; the legacy `message_stop` / `error` tokens are the seam
    // bl-56ee deletes once the writer emits only brazen events.
    matches!(
        value.get("type").and_then(|v| v.as_str()),
        Some("end") | Some("message_stop") | Some("error")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn last_line_message_stop_is_terminal() {
        let jsonl = br#"{"type":"message_start"}
{"type":"text_delta","text":"hi"}
{"type":"message_stop","usage":{"input_tokens":1,"output_tokens":1},"api_calls":1}
"#;
        assert!(has_terminal_event(jsonl));
    }

    #[test]
    fn last_line_error_is_terminal() {
        let jsonl = br#"{"type":"message_start"}
{"type":"error","kind":"fatal","message":"boom"}
"#;
        assert!(has_terminal_event(jsonl));
    }

    #[test]
    fn last_line_brazen_end_is_terminal() {
        // brazen v=1 terminator (bl-507a dual vocabulary).
        let jsonl = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_delta","index":0,"delta":{"text_delta":"hi"}}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
        assert!(has_terminal_event(jsonl));
    }

    #[test]
    fn last_line_brazen_content_delta_is_not_terminal() {
        // brazen stream killed mid-append: no trailing `end`.
        let jsonl = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_delta","index":0,"delta":{"text_delta":"par"}}
"#;
        assert!(!has_terminal_event(jsonl));
    }

    #[test]
    fn last_line_text_delta_is_not_terminal() {
        let jsonl = br#"{"type":"message_start"}
{"type":"text_delta","text":"partial"}
"#;
        assert!(!has_terminal_event(jsonl));
    }

    #[test]
    fn empty_file_is_not_terminal() {
        assert!(!has_terminal_event(b""));
        assert!(!has_terminal_event(b"\n\n"));
    }

    #[test]
    fn trailing_partial_line_is_ignored() {
        // Mid-write race: a line was started but `\n` hasn't landed yet.
        // The completed terminal event before it must still classify.
        let jsonl =
            b"{\"type\":\"message_start\"}\n{\"type\":\"message_stop\",\"api_calls\":1}\n{partial";
        assert!(has_terminal_event(jsonl));
    }

    #[test]
    fn malformed_last_line_is_not_terminal() {
        let jsonl = b"{\"type\":\"text_delta\",\"text\":\"x\"}\nnot json\n";
        assert!(!has_terminal_event(jsonl));
    }

    #[test]
    fn classify_returns_in_flight_when_steps_dir_absent() {
        let dir = tempdir().unwrap();
        assert_eq!(
            classify_unmerged(dir.path(), "no-such-conv"),
            BranchState::InFlight
        );
    }

    #[test]
    fn classify_returns_in_flight_when_response_absent() {
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-aaaa";
        std::fs::create_dir_all(dir.path().join(STEPS_DIR).join(conv).join("001")).unwrap();
        assert_eq!(classify_unmerged(dir.path(), conv), BranchState::InFlight);
    }

    #[test]
    fn classify_returns_in_flight_when_no_terminal_event() {
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-bbbb";
        let path = dir
            .path()
            .join(STEPS_DIR)
            .join(conv)
            .join("001")
            .join(RESPONSE_FILE);
        write(&path, b"{\"type\":\"text_delta\",\"text\":\"hi\"}\n");
        assert_eq!(classify_unmerged(dir.path(), conv), BranchState::InFlight);
    }

    #[test]
    fn classify_returns_stopped_on_message_stop() {
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-cccc";
        let path = dir
            .path()
            .join(STEPS_DIR)
            .join(conv)
            .join("001")
            .join(RESPONSE_FILE);
        write(
            &path,
            b"{\"type\":\"message_start\"}\n{\"type\":\"message_stop\",\"api_calls\":1}\n",
        );
        assert_eq!(classify_unmerged(dir.path(), conv), BranchState::Stopped);
    }

    #[test]
    fn classify_returns_stopped_on_error_event() {
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-dddd";
        let path = dir
            .path()
            .join(STEPS_DIR)
            .join(conv)
            .join("001")
            .join(RESPONSE_FILE);
        write(
            &path,
            b"{\"type\":\"error\",\"kind\":\"fatal\",\"message\":\"x\"}\n",
        );
        assert_eq!(classify_unmerged(dir.path(), conv), BranchState::Stopped);
    }

    #[test]
    fn classify_returns_stopped_on_brazen_end() {
        // brazen v=1 terminal drives the same Stopped classification as
        // the legacy tokens (bl-507a dual vocabulary).
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-bzen";
        let path = dir
            .path()
            .join(STEPS_DIR)
            .join(conv)
            .join("001")
            .join(RESPONSE_FILE);
        write(
            &path,
            b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n",
        );
        assert_eq!(classify_unmerged(dir.path(), conv), BranchState::Stopped);
    }

    #[test]
    fn classify_reads_latest_step_only() {
        // Earlier step terminated cleanly; latest step is mid-stream.
        // The classifier must read the latest step, returning InFlight.
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-eeee";
        let steps = dir.path().join(STEPS_DIR).join(conv);
        write(
            &steps.join("001").join(RESPONSE_FILE),
            b"{\"type\":\"message_stop\",\"api_calls\":1}\n",
        );
        write(
            &steps.join("002").join(RESPONSE_FILE),
            b"{\"type\":\"text_delta\",\"text\":\"still going\"}\n",
        );
        assert_eq!(classify_unmerged(dir.path(), conv), BranchState::InFlight);
    }
}
