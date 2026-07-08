//! Live-streaming text accumulation for in-flight steps.
//!
//! Per ARCH §2.3 / §3.5 / §4.4: the harness writes
//! `<conv-repo>/steps/<conv-id>/<NNN>/response.json` as a JSONL stream
//! of §4.4 events as the model's adapter emits them, then closes the fd
//! at completion. The frontend tails this file from disk on every tick
//! (§3.5: re-read, no in-memory accumulator that could drift) and folds
//! `text_delta` events into the displayed text.
//!
//! Functions here are pure over an on-disk `<conv-repo>/steps/` tree.
//! The only events we look at for text rendering are the text deltas
//! (§4.4); other events (`message_start`, tool-argument deltas, etc.)
//! are ignored at this layer — pulsing tool indicators (bl-23d9) and
//! branch-state badges (bl-de6b) read their own signals.
//!
//! Text fragments are read from brazen's `v=1` `content_delta` carrying
//! a `text_delta` `Delta` (§4.4). The v0.6 legacy vocabulary is retired
//! (bl-56ee).

use std::path::Path;

use super::STEPS_DIR;

/// Width of the zero-padded step sequence in on-disk paths
/// (`steps/<conv-id>/001`, `…/002`, ...). Mirrors
/// `src/prompt/step::STEP_SEQ_WIDTH` — duplicated here to keep the UI
/// crate free of a dep on the harness binary.
const STEP_SEQ_WIDTH: usize = 3;

const RESPONSE_FILE: &str = "response.json";

/// Read the latest in-flight step's accumulated streaming text from
/// disk. Returns `Some(text)` when at least one `text_delta` event has
/// landed under the conversation's highest-numbered step's
/// `response.json`; `None` when the file is absent, empty, or contains
/// only non-text events (e.g. a `message_start` followed by tool-use
/// deltas).
///
/// "Latest step" is the highest `<NNN>` directory under
/// `<conv-repo>/steps/<conv-id>/`. Re-derived on every call from the
/// directory listing so the view-model has no in-memory state to drift
/// out of sync with disk (§3.5).
pub(super) fn streaming_text_from_disk(conv_repo: &Path, conv_id: &str) -> Option<String> {
    let conv_steps = conv_repo.join(STEPS_DIR).join(conv_id);
    let latest = latest_step_dir(&conv_steps)?;
    let bytes = std::fs::read(latest.join(RESPONSE_FILE)).ok()?;
    accumulate_text_deltas(&bytes)
}

/// Find the highest-numbered `<NNN>/` directory under `conv_steps`.
/// Entries that don't match the zero-padded step shape are ignored —
/// `tools/` lives one level deeper, so it's structurally not at risk
/// here, but staying strict keeps us robust to stray files.
pub(super) fn latest_step_dir(conv_steps: &Path) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(conv_steps).ok()?;
    let mut best: Option<(u32, std::path::PathBuf)> = None;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        if name_str.len() != STEP_SEQ_WIDTH {
            continue;
        }
        let Ok(seq) = name_str.parse::<u32>() else {
            continue;
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if best.as_ref().is_none_or(|(s, _)| seq > *s) {
            best = Some((seq, path));
        }
    }
    best.map(|(_, p)| p)
}

/// Fold a JSONL `response.json` payload into accumulated text. Each
/// line is a §4.4 stream event; we collect text fragments in stream
/// order across all block indices. Lines that fail to parse, or events
/// that carry no display text, are skipped — partial-write tolerance is
/// structural (the harness may be mid-line on disk).
fn accumulate_text_deltas(bytes: &[u8]) -> Option<String> {
    let mut text = String::new();
    for line in bytes.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        let Ok(value): Result<serde_json::Value, _> = serde_json::from_slice(line) else {
            continue;
        };
        if let Some(fragment) = text_fragment(&value) {
            text.push_str(fragment);
        }
    }
    if text.is_empty() { None } else { Some(text) }
}

/// The brazen `v=1` text seam:
/// `{"type":"content_delta","delta":{"text_delta":"…"}}` — the text arm
/// of the externally-tagged `Delta`; a `json_delta` (tool arguments) or
/// `thinking_delta` carries no display text and yields `None`.
fn text_fragment(value: &serde_json::Value) -> Option<&str> {
    match value.get("type").and_then(|v| v.as_str())? {
        "content_delta" => value
            .get("delta")
            .and_then(|d| d.get("text_delta"))
            .and_then(|v| v.as_str()),
        _ => None,
    }
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
    fn accumulates_text_in_order_across_indices() {
        let jsonl = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_delta","index":0,"delta":{"text_delta":"hel"}}
{"type":"content_delta","index":0,"delta":{"text_delta":"lo"}}
{"type":"content_delta","index":0,"delta":{"text_delta":" world"}}
"#;
        assert_eq!(
            accumulate_text_deltas(jsonl).as_deref(),
            Some("hello world")
        );
    }

    #[test]
    fn accumulates_brazen_content_delta_text() {
        // brazen v=1: text rides `content_delta.delta.text_delta`
        // (bl-507a dual vocabulary).
        let jsonl = br#"{"type":"message_start","v":1,"role":"assistant"}
{"type":"content_start","index":0,"kind":{"text":{}}}
{"type":"content_delta","index":0,"delta":{"text_delta":"Hel"}}
{"type":"content_delta","index":0,"delta":{"text_delta":"lo"}}
{"type":"finish","reason":"stop"}
{"type":"end"}
"#;
        assert_eq!(accumulate_text_deltas(jsonl).as_deref(), Some("Hello"));
    }

    #[test]
    fn ignores_brazen_non_text_deltas() {
        // Tool-argument (`json_delta`) and `thinking_delta` fragments
        // carry no display text.
        let jsonl = br#"{"type":"content_delta","index":1,"delta":{"json_delta":"{\"a\":"}}
{"type":"content_delta","index":0,"delta":{"thinking_delta":"hmm"}}
"#;
        assert!(accumulate_text_deltas(jsonl).is_none());
    }

    #[test]
    fn ignores_non_text_events() {
        let jsonl = br#"{"type":"message_start"}
{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}
{"type":"tool_use_delta","index":1,"partial_json":"{}"}
{"type":"content_block_stop","index":0}
"#;
        assert!(accumulate_text_deltas(jsonl).is_none());
    }

    #[test]
    fn tolerates_malformed_lines_without_aborting() {
        let jsonl =
            b"not json\n{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"hi\"}}\n{partial";
        assert_eq!(accumulate_text_deltas(jsonl).as_deref(), Some("hi"));
    }

    #[test]
    fn empty_payload_returns_none() {
        assert!(accumulate_text_deltas(b"").is_none());
        assert!(accumulate_text_deltas(b"\n\n").is_none());
    }

    #[test]
    fn content_delta_without_text_delta_is_skipped() {
        let jsonl = br#"{"type":"content_delta","index":0,"delta":{"json_delta":"{}"}}
{"type":"content_delta","index":0,"delta":{"text_delta":"x"}}
"#;
        assert_eq!(accumulate_text_deltas(jsonl).as_deref(), Some("x"));
    }

    #[test]
    fn streaming_text_from_disk_reads_latest_step_response() {
        let dir = tempdir().unwrap();
        let conv = "20260427T120000Z-aaaa";
        let steps = dir.path().join(STEPS_DIR).join(conv);
        write(
            &steps.join("001").join(RESPONSE_FILE),
            b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"first\"}}\n",
        );
        write(
            &steps.join("002").join(RESPONSE_FILE),
            b"{\"type\":\"content_delta\",\"index\":0,\"delta\":{\"text_delta\":\"second\"}}\n",
        );
        assert_eq!(
            streaming_text_from_disk(dir.path(), conv).as_deref(),
            Some("second")
        );
    }

    #[test]
    fn streaming_text_returns_none_when_steps_dir_absent() {
        let dir = tempdir().unwrap();
        assert!(streaming_text_from_disk(dir.path(), "no-such-conv").is_none());
    }

    #[test]
    fn streaming_text_returns_none_when_response_absent() {
        let dir = tempdir().unwrap();
        let conv = "20260427T120000Z-bbbb";
        std::fs::create_dir_all(dir.path().join(STEPS_DIR).join(conv).join("001")).unwrap();
        assert!(streaming_text_from_disk(dir.path(), conv).is_none());
    }

    #[test]
    fn latest_step_dir_skips_non_step_entries() {
        // Stray file at the conv-id level (e.g. an editor backup) and a
        // dir that doesn't match `<NNN>` shape must both be ignored.
        let dir = tempdir().unwrap();
        let conv = "20260427T120000Z-cccc";
        let conv_steps = dir.path().join(STEPS_DIR).join(conv);
        std::fs::create_dir_all(conv_steps.join("001")).unwrap();
        std::fs::create_dir_all(conv_steps.join("notes")).unwrap();
        std::fs::write(conv_steps.join(".keep"), b"").unwrap();
        std::fs::write(conv_steps.join("01a"), b"").unwrap();
        let latest = latest_step_dir(&conv_steps).unwrap();
        assert!(latest.ends_with("001"));
    }
}
