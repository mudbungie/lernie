//! The live tail: the three absences that are readings, and the one place
//! this reader is deliberately looser than the engine's own.

use super::{Delta, Stream, follow};
use serde_json::{Value, json};

fn read(v: &Value) -> Result<Stream, String> {
    follow(v.as_object().expect("an object"))
}

/// A frame carrying both halves and saying which one moved last.
#[test]
fn a_frame_carries_the_whole_accumulated_fold() {
    assert_eq!(
        read(&json!({"ok": true, "kind": "follow", "stream": {
            "thinking": "weighing the two seams",
            "text": "the seam is real",
            "delta": "text",
        }}))
        .expect("a fold"),
        Stream {
            text: Some("the seam is real".to_owned()),
            thinking: Some("weighing the two seams".to_owned()),
            last_delta: Some(Delta::Text),
        }
    );
}

/// **All three absences are readings.** An empty fold is *waiting for the
/// API*, not a model that answered nothing — and a seat that read the two
/// alike would paint silence over anticipation.
#[test]
fn an_empty_fold_is_waiting_and_not_an_empty_answer() {
    assert_eq!(
        read(&json!({"stream": {}})).expect("a fold"),
        Stream::default()
    );
    let thinking_only =
        read(&json!({"stream": {"thinking": "…", "delta": "thinking"}})).expect("a fold");
    assert_eq!(thinking_only.text, None);
    assert_eq!(thinking_only.last_delta, Some(Delta::Thinking));
}

/// **The deliberate divergence.** The engine's own reader refuses an
/// unrecognised delta token; this one keeps the word. The two readers are not
/// doing the same job — refusing here throws away a whole accumulated turn to
/// avoid painting one word, while the operator is watching the tail move.
#[test]
fn an_unknown_delta_keeps_the_turn_rather_than_the_strictness() {
    let read = read(&json!({"stream": {"text": "so far", "delta": "summary"}})).expect("a fold");
    assert_eq!(read.text.as_deref(), Some("so far"));
    assert_eq!(
        read.last_delta,
        Some(Delta::Unknown("summary".to_owned())),
        "the turn survives the word"
    );
    for (delta, word) in [
        (Delta::Text, "text"),
        (Delta::Thinking, "thinking"),
        (Delta::Unknown("summary".to_owned()), "summary"),
    ] {
        assert_eq!(delta.label(), word);
    }
}

/// Rung 1 still holds around it: the fold must be there and must be an object,
/// and a half that is present and mistyped refuses by name.
#[test]
fn a_frame_with_no_fold_in_it_refuses() {
    for (frame, said) in [
        (json!({"ok": true, "kind": "follow"}), "\"stream\""),
        (json!({"stream": []}), "\"stream\""),
        (json!({"stream": {"text": 7}}), "\"text\""),
    ] {
        let refusal = read(&frame).expect_err("refused");
        assert!(refusal.contains(said), "{frame}: {refusal}");
    }
}
