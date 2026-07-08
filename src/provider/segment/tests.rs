//! Classification tests over both vocabularies (ARCH §4.4 reading
//! rules). Legacy v0.3 fixtures (`message_stop` / `error` terminals)
//! sit alongside brazen `v=1` fixtures (`{"type":"end"}` terminal,
//! `finish`/`error` inside the segment) so the follow-on writer swap
//! (bl-56ee) is proven green before it lands.

use super::{Outcome, classify};

// ---- Legacy v0.3 vocabulary -------------------------------------------

const LEGACY_COMPLETE: &str = concat!(
    r#"{"type":"message_start"}"#,
    "\n",
    r#"{"type":"text_delta","index":0,"text":"hi"}"#,
    "\n",
    r#"{"type":"message_stop","usage":{"input_tokens":1,"output_tokens":1},"api_calls":1}"#,
    "\n",
);

const LEGACY_FAILED: &str = concat!(
    r#"{"type":"message_start"}"#,
    "\n",
    r#"{"type":"error","kind":"fatal","message":"boom"}"#,
    "\n",
);

// ---- brazen v=1 vocabulary --------------------------------------------

const BRAZEN_COMPLETE: &str = concat!(
    r#"{"type":"message_start","v":1,"role":"assistant"}"#,
    "\n",
    r#"{"type":"content_start","index":0,"kind":{"text":{}}}"#,
    "\n",
    r#"{"type":"content_delta","index":0,"delta":{"text_delta":"Hi"}}"#,
    "\n",
    r#"{"type":"content_stop","index":0}"#,
    "\n",
    r#"{"type":"usage","input_tokens":12,"output_tokens":2}"#,
    "\n",
    r#"{"type":"finish","reason":"stop"}"#,
    "\n",
    r#"{"type":"end"}"#,
    "\n",
);

const BRAZEN_REFUSAL: &str = concat!(
    r#"{"type":"message_start","v":1,"role":"assistant"}"#,
    "\n",
    r#"{"type":"finish","reason":"refusal","category":"policy","explanation":null}"#,
    "\n",
    r#"{"type":"end"}"#,
    "\n",
);

const BRAZEN_FAILED: &str = concat!(
    r#"{"type":"message_start","v":1,"role":"assistant"}"#,
    "\n",
    r#"{"type":"error","kind":"transport","message":"connection reset"}"#,
    "\n",
    r#"{"type":"end"}"#,
    "\n",
);

// Two segments: a failed attempt (Error+End), then a clean retry
// (Finish+End). The last segment is authoritative → complete.
const BRAZEN_RETRY_THEN_CLEAN: &str = concat!(
    r#"{"type":"message_start","v":1,"role":"assistant"}"#,
    "\n",
    r#"{"type":"error","kind":"provider","message":"429"}"#,
    "\n",
    r#"{"type":"end"}"#,
    "\n",
    r#"{"type":"message_start","v":1,"role":"assistant"}"#,
    "\n",
    r#"{"type":"finish","reason":"stop"}"#,
    "\n",
    r#"{"type":"end"}"#,
    "\n",
);

// Two segments: a clean attempt, then a failed one. Last segment
// carries an Error → failed.
const BRAZEN_CLEAN_THEN_FAILED: &str = concat!(
    r#"{"type":"message_start","v":1,"role":"assistant"}"#,
    "\n",
    r#"{"type":"finish","reason":"stop"}"#,
    "\n",
    r#"{"type":"end"}"#,
    "\n",
    r#"{"type":"message_start","v":1,"role":"assistant"}"#,
    "\n",
    r#"{"type":"error","kind":"transport","message":"reset"}"#,
    "\n",
    r#"{"type":"end"}"#,
    "\n",
);

// Killed mid-stream: brazen deltas but no trailing `end`.
const BRAZEN_STOPPED: &str = concat!(
    r#"{"type":"message_start","v":1,"role":"assistant"}"#,
    "\n",
    r#"{"type":"content_delta","index":0,"delta":{"text_delta":"par"}}"#,
    "\n",
);

#[test]
fn legacy_message_stop_is_complete() {
    assert_eq!(classify(LEGACY_COMPLETE.as_bytes()), Outcome::Complete);
}

#[test]
fn legacy_error_terminal_is_failed() {
    assert_eq!(classify(LEGACY_FAILED.as_bytes()), Outcome::Failed);
}

#[test]
fn brazen_end_with_finish_is_complete() {
    assert_eq!(classify(BRAZEN_COMPLETE.as_bytes()), Outcome::Complete);
}

#[test]
fn brazen_refusal_finish_is_complete() {
    // Refusal is a Finish, never an Error (§4.4) — still complete.
    assert_eq!(classify(BRAZEN_REFUSAL.as_bytes()), Outcome::Complete);
}

#[test]
fn brazen_end_with_error_is_failed() {
    assert_eq!(classify(BRAZEN_FAILED.as_bytes()), Outcome::Failed);
}

#[test]
fn brazen_multi_segment_failed_then_clean_is_complete() {
    // The earlier segment's Error is ignored; the last segment wins.
    assert_eq!(
        classify(BRAZEN_RETRY_THEN_CLEAN.as_bytes()),
        Outcome::Complete
    );
}

#[test]
fn brazen_multi_segment_clean_then_failed_is_failed() {
    assert_eq!(
        classify(BRAZEN_CLEAN_THEN_FAILED.as_bytes()),
        Outcome::Failed
    );
}

#[test]
fn brazen_without_trailing_end_is_no_terminal() {
    assert_eq!(classify(BRAZEN_STOPPED.as_bytes()), Outcome::NoTerminal);
}

#[test]
fn brazen_trailing_finish_without_end_is_no_terminal() {
    // A Finish is not the terminator — only End is. Killed between the
    // Finish and the End reads as stopped (§4.4).
    let jsonl = concat!(
        r#"{"type":"message_start","v":1,"role":"assistant"}"#,
        "\n",
        r#"{"type":"finish","reason":"stop"}"#,
        "\n",
    );
    assert_eq!(classify(jsonl.as_bytes()), Outcome::NoTerminal);
}

#[test]
fn no_newline_at_all_is_no_terminal() {
    assert_eq!(classify(b"{\"type\":\"end\"}"), Outcome::NoTerminal);
    assert_eq!(classify(b""), Outcome::NoTerminal);
}

#[test]
fn only_blank_lines_is_no_terminal() {
    assert_eq!(classify(b"\n\n\n"), Outcome::NoTerminal);
}

#[test]
fn trailing_partial_line_after_terminal_is_ignored() {
    // The `end` line completed; a partial next line (no `\n`) is the
    // writer mid-append and must not mask the terminal before it.
    let jsonl = "{\"type\":\"finish\",\"reason\":\"stop\"}\n{\"type\":\"end\"}\n{par";
    assert_eq!(classify(jsonl.as_bytes()), Outcome::Complete);
}

#[test]
fn malformed_last_line_is_no_terminal() {
    let jsonl = b"{\"type\":\"content_delta\"}\nnot json\n";
    assert_eq!(classify(jsonl), Outcome::NoTerminal);
}
