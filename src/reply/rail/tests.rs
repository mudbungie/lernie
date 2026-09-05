//! The spine's reading: both absences read as absences, the label derived off
//! the commit that stores it, and the strictness that names a field.

use serde_json::json;

use super::{NO_COMMIT, card, notch, rail};
use crate::reply::convs::AgentState;

/// **A whole notch reads, and both of its optional pairs are pairs.** The
/// commit is there, so it is operable and wears a clipped label; the seat is
/// there, so both of its keys are.
#[test]
fn an_operable_notch_carries_its_commit_its_seat_and_its_rollup() {
    let read = notch(&json!({
        "seq": "001", "budget": 120, "commit": "abcdef1234567890",
        "short": "abcdef1", "row": "003-claude.json", "cut": 2
    }))
    .expect("a whole notch reads");
    assert_eq!(read.seq, "001");
    assert_eq!(read.budget, 120);
    assert!(read.operable());
    assert_eq!(read.short(), "abcdef1");
    let seat = read.seat.expect("a seated notch carries its place");
    assert_eq!(seat.row, "003-claude.json");
    assert_eq!(seat.cut, 2);
}

/// **A notch with neither is not empty, it is unreachable.** No commit means
/// no ref to fork off, and the label is the engine's own word for it.
#[test]
fn a_notch_with_no_commit_is_not_operable_and_says_so() {
    let read = notch(&json!({"seq": "002", "budget": 120})).expect("a bare notch reads");
    assert!(!read.operable());
    assert!(read.seat.is_none());
    assert_eq!(read.short(), NO_COMMIT);
}

/// A commit shorter than the clip width is its own label — the `get` is a
/// reading and not a guard against a case that cannot arise.
#[test]
fn a_commit_shorter_than_the_clip_is_its_own_label() {
    let read = notch(&json!({"seq": "003", "budget": 0, "commit": "abc"})).expect("reads");
    assert_eq!(read.short(), "abc");
}

/// A card carries the child, its fork-point sentence verbatim, its state on
/// the conversation list's own vocabulary, and the tail it may not have.
#[test]
fn a_card_carries_the_child_and_the_words_the_engine_wrote_about_it() {
    let read = card(&json!({
        "agent": "c-1-a", "name": "Cobalt", "fork": "from here",
        "state": "live", "tokens": 9, "notch": 0, "tail": "working"
    }))
    .expect("a whole card reads");
    assert_eq!(read.agent, "c-1-a");
    assert_eq!(read.fork, "from here");
    assert_eq!(read.state, AgentState::Live);
    assert_eq!(read.tokens, 9);
    assert_eq!(read.notch, 0);
    assert_eq!(read.tail.as_deref(), Some("working"));
    let quiet = card(&json!({
        "agent": "c-1-b", "name": "Dun", "fork": "from config/main",
        "state": "sideways", "tokens": 0, "notch": 1
    }))
    .expect("a card with no tail reads");
    assert!(quiet.tail.is_none());
    // Rung 3: a state this build has never seen paints as itself.
    assert_eq!(quiet.state, AgentState::Unknown("sideways".to_owned()));
}

/// Rung 1: a missing or mistyped field refuses, and names itself.
#[test]
fn a_malformed_spine_refuses_and_names_the_field() {
    let why = notch(&json!({"seq": "001"})).expect_err("a notch with no budget refuses");
    assert!(why.contains("budget"), "{why}");
    let why = notch(&json!({"seq": "001", "budget": 0, "row": "r"}))
        .expect_err("a seat with no cut refuses");
    assert!(why.contains("cut"), "{why}");
    let why = notch(&json!("a string")).expect_err("a notch that is not an object refuses");
    assert!(why.contains("not an object"), "{why}");
    let why = card(&json!("a string")).expect_err("a card that is not an object refuses");
    assert!(why.contains("not an object"), "{why}");
    let why = rail(json!({"rows": []}).as_object().expect("an object"))
        .expect_err("a spine with no cards refuses");
    assert!(why.contains("cards"), "{why}");
}

/// The whole answer: two lists, and the cards name their notches by index.
#[test]
fn the_answer_is_two_lists_and_the_cards_name_their_notches() {
    let read = rail(
        json!({
            "rows": [{"seq": "001", "budget": 1, "commit": "aaaaaaaaaa"}],
            "cards": [{"agent": "c", "name": "n", "fork": "from here",
                       "state": "stopped", "tokens": 0, "notch": 0}]
        })
        .as_object()
        .expect("an object"),
    )
    .expect("the whole answer reads");
    assert_eq!(read.notches.len(), 1);
    assert_eq!(read.cards.len(), 1);
    assert_eq!(read.cards[0].notch, 0);
}
