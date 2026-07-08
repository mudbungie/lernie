//! Response-segment classification (ARCH §4.4 "On-disk response shape:
//! appended attempt segments"; §2.9 Stopped branches; §2.10 Retries and
//! failures; §3.5 Branch-state classification).
//!
//! `response.json` is JSONL: one canonical event per line, appended
//! across one or more attempt **segments**, each terminated by its
//! `{"type":"end"}` line (§4.4). brazen guarantees every stream —
//! success, refusal, or failure — ends with exactly one `End`, so the
//! file is a sequence of self-delimiting segments and the last segment
//! is authoritative; earlier segments are the audit trail of failed
//! attempts. This module is the single seam that reads that framing
//! tail and classifies a **closed** response file's last segment. It
//! reads only framing — the terminal `End` line and whether the last
//! segment carries an `Error` — never event content, honoring the §2.3
//! diagnostic-only contract (framing-yes / content-no).
//!
//! The vocabulary is brazen's `v=1` [`brazen::Event`] type — its single
//! source of truth — so no `"end"`/`"error"` string literal duplicates
//! it here (the v0.6 legacy-vocabulary seam is retired: bl-56ee).
//!
//! Terminal classification is evaluated ONLY on a closed file. The
//! `in_flight` state (fd still open) is observed elsewhere — the §3.5
//! `IN_CLOSE_WRITE` watch and the §2.9 `/proc` writer scan — never
//! derived from these bytes.

use brazen::Event;

/// Outcome of the last attempt segment of a **closed** `response.json`
/// (the §4.4 reading rules). `in_flight` is deliberately not a variant:
/// it is the fd-open observation the caller makes, not a property of the
/// bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Trailing `End` line; the last segment carries a `Finish` (any
    /// reason, including refusal) and no `Error` — the model call
    /// completed (§4.4 *complete*).
    Complete,
    /// Trailing `End` line; the last segment carries an `Error`
    /// (retry budget exhausted or non-retryable, §2.10) — the model
    /// call failed (§4.4 *failed*). The branch is flagged.
    Failed,
    /// No trailing `End` line. On a closed file this is the §2.9
    /// kill signature (*stopped/killed*); while the fd is still open it
    /// is an in-flight mid-append. The bytes alone cannot tell the two
    /// apart — the caller disambiguates by the fd-open observation.
    NoTerminal,
}

/// Classify the last segment of a `response.json` payload. `bytes` is
/// the whole file; only complete (`\n`-terminated) lines are examined,
/// so a partially-written trailing line is ignored — the writer may be
/// mid-append (§3.5 mid-write tolerance).
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
    // brazen's single terminator is `End`; anything else on the last
    // complete line is a writer that died mid-stream (§2.9).
    if is_end(last) {
        classify_segment(head)
    } else {
        Outcome::NoTerminal
    }
}

/// A brazen segment ends with `Event::End`; its outcome is decided by
/// whether the **last** segment (the lines after the previous `End`)
/// carries an `Event::Error`. A refusal is a `Finish`, never an
/// `Error`, so it reads *complete* (§4.4). Earlier segments — the audit
/// trail of failed attempts before a retry succeeded — are skipped by
/// starting the scan after the previous `End`.
fn classify_segment(head: &[&[u8]]) -> Outcome {
    let seg_start = head.iter().rposition(|l| is_end(l)).map_or(0, |i| i + 1);
    if head[seg_start..].iter().any(|l| is_error(l)) {
        Outcome::Failed
    } else {
        Outcome::Complete
    }
}

fn is_end(line: &[u8]) -> bool {
    matches!(serde_json::from_slice::<Event>(line), Ok(Event::End))
}

fn is_error(line: &[u8]) -> bool {
    matches!(serde_json::from_slice::<Event>(line), Ok(Event::Error(_)))
}

#[cfg(test)]
mod tests;
