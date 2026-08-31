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

/// **The fold's contract**, and the reason the seat needs no second parser:
/// absorbing a frame's append is reading the concatenation whole. Proven over
/// a split rather than asserted, because that equality is the only thing
/// standing between an append lane and a seat that paints half a sentence.
#[test]
fn absorbing_the_append_is_reading_the_whole() {
    let split = [
        json!({"stream": {"thinking": "weighing ", "delta": "thinking"}}),
        json!({"stream": {"text": "the answer ", "delta": "text"}}),
        json!({"stream": {"text": "so far.", "delta": "text"}}),
    ];
    let mut fold = Stream::default();
    for frame in &split {
        fold.absorb(read(frame).expect("a fold"));
    }
    assert_eq!(
        fold,
        read(&json!({"stream": {"thinking": "weighing ",
                                "text": "the answer so far.", "delta": "text"}}))
        .expect("a fold"),
        "fold(a).absorb(fold(b)) == fold(a ++ b)"
    );
}

/// **Absent stays absent, in both directions.** A frame that appended nothing
/// leaves the accumulation exactly as it was — including its delta kind, which
/// is the last kind *seen* and not the last frame's — and absorbing onto an
/// empty fold is how every read starts, so the first frame is simply the whole
/// of it.
#[test]
fn a_frame_that_appended_nothing_leaves_the_fold_standing() {
    let mut fold = Stream::default();
    fold.absorb(read(&json!({"stream": {"text": "said", "delta": "text"}})).expect("a fold"));
    let whole = fold.clone();
    fold.absorb(read(&json!({"stream": {}})).expect("a fold"));
    assert_eq!(fold, whole, "an empty append is not a reset");
    assert_eq!(fold.last_delta, Some(Delta::Text));
    assert_eq!(fold.thinking, None, "and silence is not an empty string");
}
