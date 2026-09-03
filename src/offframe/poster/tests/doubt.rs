//! **The lost-reply contract, on the send path** (yog's `docs/REMOTE.md` §3):
//! *"A lost reply leaves an act IN DOUBT, and the recovery is a read — never a
//! resend … Asks are the opposite case and re-ask freely."*
//!
//! Five beats, and each is one clause of that sentence. Every one of them needs
//! a far end that takes the request and says nothing — the window yog's own
//! bl-d1f1 recorded having no test seam for — which is
//! [`crate::test_support::engine::Answer::Hangup`].

use super::{holding, own, posting};
use crate::offframe::poster::tick;
use crate::test_support::Scratch;
use crate::test_support::engine::Answer;
use crate::test_support::wire::{flat, wired};
use crate::ui::Aim;
use serde_json::json;

/// **An act that never left this box says exactly that, and says the remedy**
/// (REMOTE §3, bl-3969). Nothing crossed, so nothing happened, so doing it
/// again is safe — which is the opposite of what the sibling below is told, and
/// is why the two are not one sentence.
#[test]
fn an_act_that_never_left_this_box_says_so_and_says_it_is_safe_to_repeat() {
    let scratch = Scratch::new();
    let (link, mut model) = holding(
        None,
        crate::verbs::nudge("home".to_owned(), "a1b2".to_owned()),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    let said = model.notice.expect("a notice").line();
    assert!(said.contains("`nudge` was not sent"), "{said}");
    assert!(said.contains("nothing happened"), "{said}");
    assert!(said.contains("safe to do it again"), "{said}");
    assert!(!said.contains("DOUBT"), "nothing crossed: {said}");
}

/// **An act that crossed and was not answered is IN DOUBT** — REMOTE §3's whole
/// contract, on the one path that can produce it: the engine takes the request
/// and hangs up without a frame or a terminator.
///
/// Three things are asserted and each is the contract rather than the wording.
/// The sentence names the OP, because one bar serves the whole window and a
/// sentence about an unnamed act is one nobody can act on. It says the effect
/// may have run, which is the fact. And it sends the operator to LOOK rather
/// than to repeat, because *"an act is not idempotent — two clicks of Nudge are
/// two nudges"*.
#[test]
fn an_act_that_crossed_with_no_answer_is_painted_in_doubt() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![Answer::Hangup]);
    let (link, mut model) = holding(
        Some(Aim {
            channel: own().name,
            address: "home".to_owned(),
        }),
        crate::verbs::message("home".to_owned(), "a1b2".to_owned(), "ship it".to_owned()),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    let said = model.notice.expect("a notice").line();
    assert!(said.contains("`message` is IN DOUBT"), "{said}");
    assert!(said.contains("it may have run"), "{said}");
    assert!(said.contains("never resends an act"), "{said}");
    assert!(
        said.contains("the world is the record"),
        "the recovery is a read: {said}"
    );
}

/// **And it is never resent.** The next pass sends nothing, because the queue
/// is a take with no arm that puts one back: yog's disk bus refuses a re-deposit
/// mechanically (bl-d1f1) and the wire has no slot to refuse from, so *sent
/// exactly once per operator gesture* is a property of this queue or of nothing.
///
/// The engine is scripted to hang up once and to answer the second dial. If any
/// arm re-queued the act, that second answer would be spent on it — so the
/// assertion is that the far end heard the gesture exactly once, across both.
#[test]
fn an_act_in_doubt_is_never_sent_again() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            Answer::Hangup,
            Answer::Frames(vec![json!({"ok": true, "kind": "nudged"})]),
        ],
    );
    let (link, mut model) = holding(
        None,
        crate::verbs::nudge("home".to_owned(), "a1b2".to_owned()),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    let gesture = json!({"op": "nudge", "workspace": "home", "agent": "a1b2"});
    assert_eq!(
        engine
            .heard()
            .iter()
            .filter(|said| **said == gesture)
            .count(),
        1,
        "an act in doubt was sent a second time: {:?}",
        engine.heard()
    );
}

/// **The sentence outlives the beat that would have erased it.**
///
/// An act's failure used to be written into the channel's roster section, which
/// is the slot a `workspaces` answer overwrites — and the asker answers
/// `workspaces` on every beat. So the one fact on this window that nothing will
/// ever say again survived until the next successful read, which is under a
/// second. Here the roster answers cleanly *after* the act failed and the
/// sentence still stands.
#[test]
fn an_acts_sentence_survives_a_roster_answer() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![Answer::Hangup]);
    let (link, mut model) = holding(
        None,
        crate::verbs::nudge("home".to_owned(), "a1b2".to_owned()),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(model.notice.is_some(), "the act said something");
    model.absorb(
        &own(),
        crate::reply::read(&json!({"ok": true, "kind": "workspaces", "rows": []})),
    );
    let said = model.notice.expect("still there").line();
    assert!(said.contains("IN DOUBT"), "{said}");
    assert!(
        matches!(model.roster[0].held, crate::ui::Held::Heard),
        "and the channel is not painted down over an act's failure"
    );
}

/// **A posted READ keeps the read arm** (REMOTE §3: *"Asks are the opposite
/// case and re-ask freely"*). §4.21's three window-level reads come down this
/// same queue, and a lost answer to one costs nothing — so it lands on the
/// channel's own section like every other read and never claims a doubt.
#[test]
fn a_posted_read_that_went_unanswered_is_not_in_doubt() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![Answer::Hangup]);
    let (link, mut model) = posting(None, crate::ui::Posted::read(crate::verbs::workspaces()));
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert_eq!(model.notice, None, "a read is not an exchange in doubt");
    assert!(
        matches!(model.roster[0].held, crate::ui::Held::Unheld(_)),
        "it is the channel's own relationship: {:?}",
        model.roster[0].held
    );
}
