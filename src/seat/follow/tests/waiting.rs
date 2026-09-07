//! **The two answers that mis-reported** (bl-87ab, bl-3ecd): a rest that was
//! only a rest for an instant, and a terminator that was prose on a stream of
//! envelopes. Split from [`super`] at the cap on the ball's own seam — that
//! file is the loop, this is what entitles the loop to stop and what it says
//! when it does.

use serde_json::{Value, json};

use super::{agent, asked, formed, inbox, tail, watched};
use crate::render::Form;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};

/// **Mail nobody has taken yet is not rest** (bl-87ab item 2, bl-3ecd). The
/// pair every operator types is *say something, then watch it*, and the deposit
/// lands before any driver holds the lock — so the state read is right about
/// the instant and wrong about the question. The watch holds instead of
/// answering "nothing more will arrive", and says once that it is waiting.
#[test]
fn a_deposit_not_yet_taken_holds_the_watch_instead_of_ending_it() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![agent("stopped")],
            vec![inbox(1)],
            Vec::new(),
            vec![agent("in-flight")],
            vec![tail("on it")],
            vec![agent("quiescent")],
            vec![inbox(0)],
        ],
    );
    let (verdict, said) = watched(&scratch);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert!(
        verdict.text.contains("is at rest (quiescent)"),
        "it ended on the real rest: {}",
        verdict.text
    );
    assert!(said.iter().any(|l| l.contains("on it")), "{said:?}");
    let waiting: Vec<&String> = said
        .iter()
        .filter(|l| l.contains("not yet taken"))
        .collect();
    assert_eq!(waiting.len(), 1, "said once, not once a look: {said:?}");
    let line = waiting[0];
    assert!(line.contains("1 deposit"), "{line}");
    assert!(line.contains("at rest (stopped)"), "{line}");
    assert!(line.contains("Ctrl-C"), "{line}");
    assert!(line.starts_with("┊ "), "the seat's own narration: {line}");
    assert_eq!(asked(&engine).len(), 7, "{:?}", asked(&engine));
}

/// **An inbox this seat could not READ is no mail.** Every way the probe can
/// fail leaves the watch with exactly the reading the state read already gave
/// it, because holding somebody's connection open forever on the strength of an
/// unanswered question is worse than the sentence this exists to fix.
#[test]
fn an_inbox_that_cannot_be_read_ends_the_watch_on_the_state() {
    for answer in [
        crate::test_support::engine::Answer::Frames(vec![
            json!({"ok": false, "error": "no such conversation"}),
        ]),
        crate::test_support::engine::Answer::Hangup,
    ] {
        let scratch = Scratch::new();
        wired(
            &scratch,
            &flat(),
            vec![
                crate::test_support::engine::Answer::Frames(vec![agent("quiescent")]),
                answer,
            ],
        );
        let (verdict, said) = watched(&scratch);
        assert_eq!(verdict.code, 0, "{}", verdict.text);
        assert!(
            verdict.text.contains("is at rest (quiescent)"),
            "{}",
            verdict.text
        );
        assert!(said.is_empty(), "{said:?}");
    }
}

/// **`--json` narrates nothing, on the one line that used to** (bl-87ab item
/// 1). `--help` promises the frames exactly as they crossed, one envelope per
/// line; the terminator was prose, so a reader calling `json.loads` on each
/// line died on it. What ends the watch is now the `agent` frame that ended it.
#[test]
fn the_json_form_ends_on_the_frame_that_ended_the_watch() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![
            vec![agent("in-flight")],
            vec![tail("The answer")],
            vec![agent("quiescent")],
            vec![inbox(0)],
        ],
    );
    let (verdict, said) = formed(&scratch, Form::Json);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    for line in said.iter().chain(std::iter::once(&verdict.text)) {
        let frame: Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("{line:?} is not an envelope: {e}"));
        assert!(frame.get("kind").is_some(), "{line}");
    }
    assert_eq!(
        serde_json::from_str::<Value>(&verdict.text)
            .expect("the terminator is an envelope")
            .get("state")
            .and_then(Value::as_str),
        Some("quiescent"),
        "the state a script wanted from the sentence: {}",
        verdict.text
    );
}

/// The waiting line is a rendering, so `--json` does not say it either — a
/// script gets frames and nothing else, on both of the two lines this word
/// writes for itself.
#[test]
fn the_json_form_says_nothing_while_it_waits_for_a_driver() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![
            vec![agent("stopped")],
            vec![inbox(2)],
            Vec::new(),
            vec![agent("quiescent")],
            vec![inbox(0)],
        ],
    );
    let (verdict, said) = formed(&scratch, Form::Json);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert!(said.is_empty(), "{said:?}");
    assert!(
        serde_json::from_str::<Value>(&verdict.text).is_ok(),
        "{}",
        verdict.text
    );
}

/// A held conversation's own row: at rest by its state, and parked by its
/// mark — which is the shape the defect lived in, since the two are read off
/// the same object one field apart.
fn holding() -> Value {
    let mut row = agent("quiescent");
    row["held"] = json!({"tool": "box2_service_status", "tool_use_id": "toolu_02",
                         "reason": "classified opaque"});
    row
}

/// **A HELD conversation is not at rest, and the ending says so** (bl-3a1f;
/// yog bl-58bb). The capability control parks the call *before* the executor
/// is entered, so the state read says `quiescent` and the ordinary ending is
/// false in both halves: something more WILL arrive, and neither *nudge* nor
/// *message* is what makes it. The line names the call, the control's own
/// reason and `answer`, which is the gesture that lifts it — spelled with the
/// address the watch was aimed at, because a remedy an operator has to
/// complete from memory is one they complete wrongly.
#[test]
fn a_held_conversation_ends_the_watch_naming_the_call_and_the_answer() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![vec![holding()]]);
    let (verdict, _) = watched(&scratch);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    for needle in [
        "box2_service_status",
        "toolu_02",
        "classified opaque",
        "for your answer",
        "lernie answer home PelicanQuiet pass|refuse",
    ] {
        assert!(verdict.text.contains(needle), "{needle}: {}", verdict.text);
    }
    assert!(
        !verdict.text.contains("nudged or messaged"),
        "the two remedies are both wrong for a park: {}",
        verdict.text
    );
}

/// **It asks no inbox, because mail behind a park is still behind the park.**
/// The probe that tells a rest about to end from one that is not has nothing
/// to add here: what the conversation is waiting for is the operator, and it
/// will go on waiting whatever is in the inbox.
#[test]
fn a_park_ends_the_watch_without_asking_what_is_waiting() {
    let scratch = Scratch::new();
    let engine = wired(&scratch, &flat(), vec![vec![holding()]]);
    let (verdict, _) = watched(&scratch);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    let ops: Vec<String> = asked(&engine)
        .iter()
        .map(|frame| frame["op"].as_str().unwrap_or_default().to_owned())
        .collect();
    assert_eq!(ops, vec!["agent"], "{ops:?}");
}

/// **And `--json` says nothing of this seat's own here either** (bl-87ab's
/// rule, applied to the third sentence this word can write): what ends the
/// watch is the `agent` frame that ended it, printed as it crossed — and that
/// frame already carries the hold mark the rendered sentence is made of.
#[test]
fn the_json_form_ends_a_park_with_the_frame_that_ended_it() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![vec![holding()]]);
    let (verdict, said) = formed(&scratch, Form::Json);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert!(said.is_empty(), "{said:?}");
    let frame: Value = serde_json::from_str(verdict.text.trim()).expect("one envelope");
    assert_eq!(frame["held"]["tool"], json!("box2_service_status"));
}
