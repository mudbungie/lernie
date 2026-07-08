//! Response-segment classification (ARCH §4.4 "On-disk response shape:
//! appended attempt segments"; §2.9 Stopped branches; §2.10 Retries and
//! failures; §3.5 Branch-state classification).
//!
//! `response.json` is JSONL: one canonical event per line, appended
//! across one or more attempt **segments**, each terminated by its
//! terminal line (§4.4). The last segment is authoritative; earlier
//! segments are the audit trail of failed attempts. This module is the
//! single seam that reads that framing tail and classifies a **closed**
//! response file's last segment. It reads only framing — the terminal
//! line and whether the last segment carries an `Error` — never event
//! content, honoring the §2.3 diagnostic-only contract (framing-yes /
//! content-no).
//!
//! **Dual vocabulary — the v0.6 transition (bl-507a).** Two event
//! vocabularies coexist while the provider layer folds into brazen
//! (§4.4): the **legacy** v0.3 stream (terminal `message_stop`, failure
//! terminal `error`) and **brazen**'s `v=1` stream (terminal
//! `Event::End` — the line `{"type":"end"}` — with `Finish`/`Error`
//! carried *inside* the segment, ahead of the `End`). Every
//! `response.json` reader accepts both so the writer swap in the
//! follow-on ball (bl-56ee) merges green. The brazen arm is expressed
//! through the linked `brazen::Event` type — the vocabulary's single
//! source of truth — so no `"end"`/`"error"` string literal duplicates
//! it here. [`legacy_terminal`] is the sole legacy seam and is the one
//! piece the follow-on ball deletes.
//!
//! Terminal classification is evaluated ONLY on a closed file. The
//! `in_flight` state (fd still open) is observed elsewhere — the §3.5
//! `IN_CLOSE_WRITE` watch and the §2.9 `/proc` writer scan — never
//! derived from these bytes.

use brazen::Event;
use serde_json::Value;

/// Outcome of the last attempt segment of a **closed** `response.json`
/// (the §4.4 reading rules). `in_flight` is deliberately not a variant:
/// it is the fd-open observation the caller makes, not a property of the
/// bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Trailing terminal line; the last segment carries a `Finish` (any
    /// reason, including refusal) and no `Error` — the model call
    /// completed (§4.4 *complete*).
    Complete,
    /// Trailing terminal line; the last segment carries an `Error`
    /// (retry budget exhausted or non-retryable, §2.10) — the model
    /// call failed (§4.4 *failed*). The branch is flagged.
    Failed,
    /// No trailing terminal line. On a closed file this is the §2.9
    /// kill signature (*stopped/killed*); while the fd is still open it
    /// is an in-flight mid-append. The bytes alone cannot tell the two
    /// apart — the caller disambiguates by the fd-open observation.
    NoTerminal,
}

/// Classify the last segment of a `response.json` payload. `bytes` is
/// the whole file; only complete (`\n`-terminated) lines are examined,
/// so a partially-written trailing line is ignored — the writer may be
/// mid-append (§3.5 mid-write tolerance). See the module docs for the
/// dual-vocabulary contract.
pub fn classify(bytes: &[u8]) -> Outcome {
    // Drop any trailing partial line: keep only through the last `\n`.
    // No `\n` at all means no completed line to classify.
    let Some(nl) = bytes.iter().rposition(|&b| b == b'\n') else {
        return Outcome::NoTerminal;
    };
    let lines: Vec<&[u8]> = bytes[..=nl]
        .split(|&b| b == b'\n')
        .filter(|line| !line.is_empty())
        .collect();
    let Some((&last, head)) = lines.split_last() else {
        return Outcome::NoTerminal;
    };
    match terminal(last) {
        Terminal::None => Outcome::NoTerminal,
        Terminal::Complete => Outcome::Complete,
        Terminal::Failed => Outcome::Failed,
        Terminal::End => classify_brazen_segment(head),
    }
}

/// A brazen segment ends with `Event::End`; its outcome is decided by
/// whether the **last** segment (the lines after the previous `End`)
/// carries an `Event::Error`. A refusal is a `Finish`, never an
/// `Error`, so it reads *complete* (§4.4). Earlier segments — the audit
/// trail of failed attempts before a retry succeeded — are skipped by
/// starting the scan after the previous `End`.
fn classify_brazen_segment(head: &[&[u8]]) -> Outcome {
    let seg_start = head.iter().rposition(|l| is_end(l)).map_or(0, |i| i + 1);
    if head[seg_start..].iter().any(|l| is_error(l)) {
        Outcome::Failed
    } else {
        Outcome::Complete
    }
}

/// How the last complete line terminates its segment.
enum Terminal {
    /// Not a terminal line — the writer stopped mid-stream.
    None,
    /// Legacy `message_stop`: a clean step finish.
    Complete,
    /// A trailing `Event::Error`. In brazen an `Error` is always
    /// followed by `End`, so a *last-line* error is the legacy failure
    /// terminal (or the vanishing brazen kill-after-error window, which
    /// is equally not-complete) — either way, failed.
    Failed,
    /// Brazen `Event::End`: the provider-agnostic terminator; the
    /// segment's outcome is read from its `Finish`/`Error` content.
    End,
}

fn terminal(line: &[u8]) -> Terminal {
    match serde_json::from_slice::<Event>(line) {
        Ok(Event::End) => Terminal::End,
        Ok(Event::Error(_)) => Terminal::Failed,
        _ => legacy_terminal(line),
    }
}

/// The legacy v0.3 vocabulary seam — deleted by bl-56ee once the writer
/// emits only brazen events. A legacy stream terminates a successful
/// step with `message_stop`; its failure terminal `error` is already
/// caught above as `brazen::Event::Error`.
fn legacy_terminal(line: &[u8]) -> Terminal {
    if json_type_is(line, "message_stop") {
        Terminal::Complete
    } else {
        Terminal::None
    }
}

fn is_end(line: &[u8]) -> bool {
    matches!(serde_json::from_slice::<Event>(line), Ok(Event::End))
}

fn is_error(line: &[u8]) -> bool {
    matches!(serde_json::from_slice::<Event>(line), Ok(Event::Error(_)))
}

/// True when `line` is a JSON object whose `type` equals `expected`.
/// Malformed JSON is not that type, so it returns `false`.
fn json_type_is(line: &[u8], expected: &str) -> bool {
    let Ok(value) = serde_json::from_slice::<Value>(line) else {
        return false;
    };
    value.get("type").and_then(Value::as_str) == Some(expected)
}

#[cfg(test)]
mod tests;
