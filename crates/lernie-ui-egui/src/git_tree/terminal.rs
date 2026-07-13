//! §4.4 terminal reading rules for a settled `response.json`.
//!
//! Once the fd is closed (the classifier reaches here only with no lock
//! held, §3.5), the latest step's `response.json` is classified by its tail
//! (§4.4):
//!
//! - *complete* — last line `end`, and the last segment carries a `finish`
//!   with no `error`.
//! - *failed* — last segment carries an `error` (retry budget exhausted or
//!   non-retryable, §2.10).
//! - *killed* — closed with no trailing `end` (writer died mid-stream,
//!   §2.9).
//!
//! Only *complete* is quiescent; *failed* and *killed* are stopped (§3.5).
//! This is a self-delimiting reader over appended attempt segments (§4.4):
//! only the **last** segment decides, because it is the settled outcome.

/// Is the payload a §4.4 *complete* model call? `false` for failed, killed,
/// and empty files. A trailing partial line (no `\n` yet) is dropped.
pub(super) fn last_segment_complete(bytes: &[u8]) -> bool {
    // Only fully-terminated lines: drop anything after the final newline.
    let terminated = match bytes.iter().rposition(|&b| b == b'\n') {
        Some(idx) => &bytes[..=idx],
        None => return false,
    };
    let lines: Vec<&[u8]> = terminated
        .split(|&b| b == b'\n')
        .filter(|l| !l.is_empty())
        .collect();
    let Some((last, rest)) = lines.split_last() else {
        return false;
    };
    if event_type(last) != Some("end") {
        return false;
    }
    // Walk the last segment backward from just before the final `end` to
    // the previous segment's `end` boundary; require a `finish`, no `error`.
    let mut saw_finish = false;
    for line in rest.iter().rev() {
        match event_type(line) {
            Some("end") => break,
            Some("error") => return false,
            Some("finish") => saw_finish = true,
            _ => {}
        }
    }
    saw_finish
}

/// The classifier-relevant `type` field of one JSONL event line, or `None`
/// if it does not parse as a JSON object with a recognized string `type`.
fn event_type(line: &[u8]) -> Option<&'static str> {
    let value: serde_json::Value = serde_json::from_slice(line).ok()?;
    match value.get("type")?.as_str()? {
        "end" => Some("end"),
        "finish" => Some("finish"),
        "error" => Some("error"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FINISH_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
    const ERROR_END: &[u8] = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"error","kind":"transport","message":"reset"}
{"type":"end"}
"#;

    #[test]
    fn finish_then_end_is_complete() {
        assert!(last_segment_complete(FINISH_END));
    }

    #[test]
    fn error_then_end_is_not_complete() {
        // A failed attempt (error+end) is *failed*, not complete (§4.4).
        assert!(!last_segment_complete(ERROR_END));
    }

    #[test]
    fn no_trailing_end_is_not_complete() {
        let jsonl = br#"{"type":"content_delta","index":0,"delta":{"text_delta":"hi"}}
"#;
        assert!(!last_segment_complete(jsonl));
    }

    #[test]
    fn empty_or_newline_only_is_not_complete() {
        assert!(!last_segment_complete(b""));
        assert!(!last_segment_complete(b"\n\n"));
    }

    #[test]
    fn trailing_partial_line_after_end_is_ignored() {
        let jsonl = b"{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n{partial";
        assert!(last_segment_complete(jsonl));
    }

    #[test]
    fn only_latest_segment_decides_complete() {
        // A prior failed attempt then a clean retry: complete.
        let jsonl = br#"{"type":"error","kind":"x"}
{"type":"end"}
{"type":"message_start","v":1}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
        assert!(last_segment_complete(jsonl));
    }

    #[test]
    fn latest_segment_error_after_earlier_finish_is_not_complete() {
        // A clean attempt then a failed retry: failed (last segment wins).
        let jsonl = br#"{"type":"finish","reason":"stop"}
{"type":"end"}
{"type":"error","kind":"x"}
{"type":"end"}
"#;
        assert!(!last_segment_complete(jsonl));
    }

    #[test]
    fn end_without_finish_or_error_is_not_complete() {
        // Defensive: an `end` with neither finish nor error in its segment
        // is not a clean completion.
        assert!(!last_segment_complete(
            b"{\"type\":\"message_start\"}\n{\"type\":\"end\"}\n"
        ));
    }

    #[test]
    fn malformed_last_line_is_not_complete() {
        assert!(!last_segment_complete(b"{\"type\":\"finish\"}\nnot json\n"));
    }
}
