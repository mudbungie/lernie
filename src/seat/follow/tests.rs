//! The watch: the state read that opens it, the reads it holds, and the ways
//! it ends.

use super::follow;
use crate::cli::Stream;
use crate::render::Form;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use serde_json::{Value, json};

/// A conversation's own row, as thin as the reader allows, in the state named.
fn agent(state: &str) -> Value {
    json!({"ok": true, "kind": "agent", "agent": "c-1", "root": "c-1",
           "display": "PelicanQuiet", "display_only": false, "tip": "abc",
           "state": state, "refused": false, "present": true,
           "nudgeable": true, "stoppable": false, "stop_children": false,
           "spend": {"tokens": {"input": 0, "output": 0, "cache_read": 0,
                                "cache_write": 0, "total": 0},
                     "attribution": {"kind": "own"}}})
}

/// One frame of a live tail.
fn tail(text: &str) -> Value {
    json!({"ok": true, "kind": "follow", "tools": [], "stream": {"delta": "text", "text": text}})
}

/// The gestures an engine was asked, without the version preface each
/// connection opens with.
fn asked(engine: &crate::test_support::engine::Engine) -> Vec<Value> {
    engine
        .heard()
        .into_iter()
        .filter(|frame| frame.get("op").is_some())
        .collect()
}

/// Run a watch and hand back its verdict beside everything it printed.
fn watched(scratch: &Scratch) -> (crate::cli::Verdict, Vec<String>) {
    let mut said: Vec<String> = Vec::new();
    let verdict = follow(
        scratch.path(),
        "home",
        "PelicanQuiet",
        Form::Rendered,
        &mut |line| said.push(line.to_owned()),
    );
    (verdict, said)
}

/// **A conversation already at rest is told so at once, and exits 0**
/// (bl-3dca). It opens no held connection at all, so the half-minute of
/// silence is not shortened — it never happens — and the failure code that was
/// indistinguishable from a dead engine is gone with it.
#[test]
fn a_quiescent_conversation_answers_at_once_and_succeeds() {
    let scratch = Scratch::new();
    let engine = wired(&scratch, &flat(), vec![vec![agent("quiescent")]]);
    let (verdict, said) = watched(&scratch);
    assert_eq!(verdict.code, 0);
    assert_eq!(verdict.stream, Stream::Out);
    assert!(
        verdict.text.contains("is at rest (quiescent)"),
        "{}",
        verdict.text
    );
    assert!(
        verdict.text.contains("nudged or messaged"),
        "{}",
        verdict.text
    );
    // **The closing line is the seat's own, so it wears the gutter and
    // carries how long the watch held** (bl-293d) — a watch that answers no
    // duration leaves "what did that take" a question nobody can answer.
    assert!(verdict.text.starts_with("┊ "), "{}", verdict.text);
    assert!(verdict.text.contains(" after 0s "), "{}", verdict.text);
    assert!(said.is_empty(), "no tail was held: {said:?}");
    assert_eq!(
        asked(&engine),
        vec![json!({"op": "agent", "workspace": "home", "agent": "PelicanQuiet"})],
        "it asked what the conversation was doing and nothing else"
    );
}

/// **The watch outlives the engine's step boundary** (bl-f076). A held read
/// ends when a step commits; the conversation is still working, so the line is
/// taken up again — and the operator sees one continuous tail rather than
/// having to invent a loop.
#[test]
fn a_read_that_ends_at_a_step_boundary_is_taken_up_again() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![agent("in-flight")],
            vec![tail("The"), tail(" answer")],
            vec![agent("in-flight")],
            vec![tail(" is 391.")],
            vec![agent("quiescent")],
        ],
    );
    let (verdict, said) = watched(&scratch);
    assert_eq!(verdict.code, 0);
    assert!(
        verdict.text.contains("is at rest (quiescent)"),
        "{}",
        verdict.text
    );
    assert_eq!(said.len(), 3, "one printing per frame: {said:?}");
    assert!(said.concat().contains(" is 391."), "{said:?}");
    assert_eq!(
        asked(&engine).len(),
        5,
        "three state reads and two held reads: {:?}",
        asked(&engine)
    );
}

/// **A read that brought nothing settles before asking again.** The engine
/// paces this loop on its own, so the pause is insurance against one that
/// closes an empty read at once — the alternative being a busy loop on
/// somebody else's socket.
#[test]
fn a_read_that_brought_nothing_waits_before_asking_again() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![vec![agent("live")], Vec::new(), vec![agent("stopped")]],
    );
    let began = std::time::Instant::now();
    let (verdict, said) = watched(&scratch);
    assert!(began.elapsed() >= super::SETTLE, "it did not settle");
    assert_eq!(verdict.code, 0);
    assert!(
        verdict.text.contains("is at rest (stopped)"),
        "{}",
        verdict.text
    );
    assert!(said.is_empty(), "{said:?}");
}

/// **A state this build has no word for ends the watch rather than looping on
/// it.** The reply vocabulary paints an unknown token as itself (DESIGN §4.9
/// rung 3), and the safe reading is *this seat cannot say that it is working*.
#[test]
fn a_state_this_seat_does_not_know_ends_the_watch_and_says_the_word() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![vec![agent("hibernating")]]);
    let (verdict, _) = watched(&scratch);
    assert_eq!(verdict.code, 0);
    assert!(verdict.text.contains("(hibernating)"), "{}", verdict.text);
}

/// **An answer no state can be read out of is the product**, and the exit code
/// says the watch never started: a refusal about a workspace that is not there
/// must not read as a conversation that came to rest.
#[test]
fn an_answer_with_no_state_in_it_is_printed_and_fails() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![vec![
            json!({"ok": false, "error": "unknown workspace \"hoem\""}),
        ]],
    );
    let (verdict, said) = watched(&scratch);
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Out);
    assert!(
        verdict.text.contains("unknown workspace"),
        "{}",
        verdict.text
    );
    assert!(said.is_empty(), "{said:?}");
}

/// **A channel that will not answer is this box's fact, not the
/// conversation's**, and it earns the sentence alone — on either of the two
/// reads the watch makes.
#[test]
fn a_channel_that_will_not_answer_says_so_on_either_read() {
    let scratch = Scratch::new();
    let (verdict, _) = watched(&scratch);
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(
        verdict.text.contains("no wire provisioned"),
        "{}",
        verdict.text
    );

    let held = Scratch::new();
    wired(
        &held,
        &flat(),
        vec![
            crate::test_support::engine::Answer::Frames(vec![agent("in-flight")]),
            crate::test_support::engine::Answer::Hangup,
        ],
    );
    let (verdict, _) = watched(&held);
    assert_eq!(verdict.code, 1);
    assert_eq!(verdict.stream, Stream::Err);
}
