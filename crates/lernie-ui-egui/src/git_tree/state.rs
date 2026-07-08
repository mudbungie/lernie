//! Branch-state classifier (ARCH §2.9 / §3.5 / §7.1).
//!
//! The four states the live view renders are derived from refs and the
//! brazen `v=1` terminal event in `response.json`, not from any sidecar
//! marker (PRINCIPLES.md "Single source of truth"). For an unmerged
//! branch row the classifier reads the latest step's `response.json`
//! and returns:
//!
//! - [`BranchState::InFlight`] when no §4.4 terminal `end` event is
//!   present (the writer is still appending, or the response file is
//!   absent pre-first-event), OR a terminal `end` is present but the
//!   writer still holds the fd open — the §3.5 fd-close gate: the
//!   harness holds ONE fd across every retry attempt and the backoff
//!   sleeps between them (§4.4), so a mid-retry `end` segment stays
//!   `in_flight`, never `stopped`.
//! - [`BranchState::Stopped`] when the latest step's `response.json`
//!   carries a terminal `end` line AND no writer holds its fd open
//!   (§2.9 — kill / crash / explicit stop are indistinguishable on
//!   disk; a completed or failed step both settle here, the badge does
//!   not split them).
//!
//! [`BranchState::Merged`] and [`BranchState::Conflicted`] are part of
//! the type for renderer completeness but are not produced by this
//! classifier (see the enum docs).

use std::path::Path;

use super::STEPS_DIR;
use super::fd_probe::WriterProbe;
use super::streaming::latest_step_dir;

const RESPONSE_FILE: &str = "response.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchState {
    /// Branch is reachable from `main`'s HEAD (§2.3). Not produced by
    /// the unmerged-branch classifier; reserved for future renderers
    /// that want an explicit badge on trunk merge nodes.
    Merged,
    /// Latest step's `response.json` is still being written (no terminal
    /// `end`, or file absent), or a terminal `end` landed while a writer
    /// still holds the fd open (mid-retry, §3.5).
    InFlight,
    /// Latest step's `response.json` carries a terminal `end` and no
    /// writer holds its fd open. The chain is no longer advancing.
    Stopped,
    /// Subagent merge attempt conflicted (§2.6 step 6). Not reachable
    /// until subagent merges ship. Retained so renderers handle it.
    Conflicted,
}

/// Classify an unmerged conversation branch by reading the latest
/// step's `response.json`. Always returns `InFlight` or `Stopped` —
/// `Merged` and `Conflicted` are produced elsewhere.
pub(super) fn classify_unmerged(
    conv_repo: &Path,
    conv_id: &str,
    probe: &dyn WriterProbe,
) -> BranchState {
    let conv_steps = conv_repo.join(STEPS_DIR).join(conv_id);
    let Some(latest) = latest_step_dir(&conv_steps) else {
        return BranchState::InFlight;
    };
    let path = latest.join(RESPONSE_FILE);
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(_) => return BranchState::InFlight,
    };
    if !has_terminal_event(&bytes) {
        return BranchState::InFlight;
    }
    // Terminal `end` present — the §3.5 fd-close gate decides. A writer
    // still holding the fd open means the model call is mid-flight
    // (mid-retry, or between the terminal `end` of one attempt and the
    // next), so the segment reads in_flight.
    if probe.writer_open(&path) {
        BranchState::InFlight
    } else {
        BranchState::Stopped
    }
}

/// Last completed JSONL line is brazen's terminal `end` event (§4.4).
/// Mid-write tolerance: a trailing line with no `\n` yet is dropped, and
/// only the most recent fully-terminated line is examined.
fn has_terminal_event(bytes: &[u8]) -> bool {
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
    value.get("type").and_then(|v| v.as_str()) == Some("end")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Stub probe with a fixed answer.
    struct StubProbe(bool);
    impl WriterProbe for StubProbe {
        fn writer_open(&self, _path: &Path) -> bool {
            self.0
        }
    }
    fn closed() -> StubProbe {
        StubProbe(false)
    }
    fn open() -> StubProbe {
        StubProbe(true)
    }

    fn write(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    fn resp(dir: &Path, conv: &str, seq: &str) -> std::path::PathBuf {
        dir.join(STEPS_DIR).join(conv).join(seq).join(RESPONSE_FILE)
    }

    const FINISH_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
    const ERROR_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"transport","message":"reset"}
{"type":"end"}
"#;

    #[test]
    fn last_line_brazen_end_is_terminal() {
        assert!(has_terminal_event(FINISH_END));
    }

    #[test]
    fn brazen_content_delta_is_not_terminal() {
        let jsonl = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_delta","index":0,"delta":{"text_delta":"par"}}
"#;
        assert!(!has_terminal_event(jsonl));
    }

    #[test]
    fn legacy_message_stop_is_not_terminal() {
        // The v0.6 transition ends here: only brazen `end` is terminal.
        assert!(!has_terminal_event(b"{\"type\":\"message_stop\"}\n"));
        assert!(!has_terminal_event(
            b"{\"type\":\"error\",\"kind\":\"x\"}\n"
        ));
    }

    #[test]
    fn empty_file_is_not_terminal() {
        assert!(!has_terminal_event(b""));
        assert!(!has_terminal_event(b"\n\n"));
    }

    #[test]
    fn trailing_partial_line_is_ignored() {
        let jsonl = b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n{partial";
        assert!(has_terminal_event(jsonl));
    }

    #[test]
    fn malformed_last_line_is_not_terminal() {
        assert!(!has_terminal_event(
            b"{\"type\":\"content_delta\"}\nnot json\n"
        ));
    }

    #[test]
    fn classify_in_flight_when_steps_dir_absent() {
        let dir = tempdir().unwrap();
        assert_eq!(
            classify_unmerged(dir.path(), "no-such-conv", &closed()),
            BranchState::InFlight
        );
    }

    #[test]
    fn classify_in_flight_when_response_absent() {
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-aaaa";
        std::fs::create_dir_all(dir.path().join(STEPS_DIR).join(conv).join("001")).unwrap();
        assert_eq!(
            classify_unmerged(dir.path(), conv, &closed()),
            BranchState::InFlight
        );
    }

    #[test]
    fn classify_in_flight_when_no_terminal_event() {
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-bbbb";
        write(
            &resp(dir.path(), conv, "001"),
            b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"hi\"}}\n",
        );
        assert_eq!(
            classify_unmerged(dir.path(), conv, &closed()),
            BranchState::InFlight
        );
    }

    #[test]
    fn classify_stopped_on_terminal_end_with_fd_closed() {
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-cccc";
        write(&resp(dir.path(), conv, "001"), FINISH_END);
        assert_eq!(
            classify_unmerged(dir.path(), conv, &closed()),
            BranchState::Stopped
        );
    }

    #[test]
    fn classify_stopped_on_error_segment_with_fd_closed() {
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-dddd";
        write(&resp(dir.path(), conv, "001"), ERROR_END);
        assert_eq!(
            classify_unmerged(dir.path(), conv, &closed()),
            BranchState::Stopped
        );
    }

    #[test]
    fn mid_retry_terminal_segment_with_fd_open_is_in_flight() {
        // THE §3.5 fd-close gate: an `error`+`end` failed attempt with a
        // writer still holding the fd open is mid-retry — in_flight, not
        // stopped. Same for a `finish`+`end` between attempts.
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-eeee";
        write(&resp(dir.path(), conv, "001"), ERROR_END);
        assert_eq!(
            classify_unmerged(dir.path(), conv, &open()),
            BranchState::InFlight
        );
    }

    #[test]
    fn classify_reads_latest_step_only() {
        let dir = tempdir().unwrap();
        let conv = "20260427T140000Z-ffff";
        write(&resp(dir.path(), conv, "001"), FINISH_END);
        write(
            &resp(dir.path(), conv, "002"),
            b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"go\"}}\n",
        );
        assert_eq!(
            classify_unmerged(dir.path(), conv, &closed()),
            BranchState::InFlight
        );
    }
}
