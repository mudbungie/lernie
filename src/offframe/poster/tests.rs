//! The poster's pass: what it sends, what it stamps a receipt with, and what a
//! channel that will not open costs.
//!
//! **The lost-reply contract is its own file** (`tests/doubt.rs`, REMOTE §3):
//! it is one subject with five beats and it is the only part of this pass that
//! needs an engine which hangs up, so it splits at the design-time budget on
//! the seam the ball drew.

use std::time::Duration;

use super::tick;
use crate::state::Link;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};

pub(super) fn own() -> Channel {
    Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    }
}

/// A link holding one composed gesture, aimed or not.
pub(super) fn holding(aim: Option<Aim>, envelope: Value) -> (Link, Model) {
    posting(aim, crate::ui::Posted::act(envelope))
}

/// The same, with the classification the control made spelled out.
pub(super) fn posting(aim: Option<Aim>, posted: crate::ui::Posted) -> (Link, Model) {
    let link = Link::new(Duration::from_millis(1));
    let mut model = Model {
        roster: vec![Chunk::of(own())],
        aim,
        outbox: vec![posted],
        ..Model::default()
    };
    link.settle(&mut model);
    (link, model)
}

/// **What a click composed goes out, and its receipt comes back through the
/// same door every answer does.**
#[test]
fn a_composed_gesture_is_sent_and_its_receipt_lands_as_a_frame() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![vec![json!({"ok": true, "kind": "outcome", "exit": 0,
                         "stdout": "deposited", "stderr": ""})]],
    );
    let (link, mut model) = holding(
        Some(Aim {
            channel: own().name,
            address: "home".to_owned(),
        }),
        crate::verbs::message("home".to_owned(), "a1b2".to_owned(), "ship it".to_owned()),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(
        engine
            .heard()
            .contains(&json!({"op": "message", "workspace": "home",
                              "agent": "a1b2", "content": "ship it"})),
        "{:?}",
        engine.heard()
    );
    assert_eq!(model.notice, None, "a run that landed says nothing");
}

/// A receipt that says the act did not land is told **in the child's own
/// words**, which is what makes it actionable.
#[test]
fn a_run_that_failed_is_told_in_the_child_s_own_words() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![vec![json!({"ok": true, "kind": "outcome", "exit": 1,
                         "stdout": "", "stderr": "unknown conversation"})]],
    );
    let (link, mut model) = holding(
        Some(Aim {
            channel: own().name,
            address: "home".to_owned(),
        }),
        crate::verbs::nudge("home".to_owned(), "a1b2".to_owned()),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(
        model
            .notice
            .expect("a notice")
            .line()
            .contains("unknown conversation")
    );
}

/// **A gesture composed with no aim is still sent**: the address it carries is
/// what routes it, and the address is the whole of what routing needs. And it
/// is stamped honestly, because the stamp is no longer a guess off the aim —
/// `crate::seat::route` answers the channel it chose (bl-c70d).
#[test]
fn a_gesture_with_no_aim_is_still_routed_by_the_address_it_carries() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![vec![json!({"ok": true, "kind": "nudged"})]],
    );
    let (link, mut model) = holding(
        None,
        crate::verbs::nudge("home".to_owned(), "a1b2".to_owned()),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(
        engine
            .heard()
            .contains(&json!({"op": "nudge", "workspace": "home", "agent": "a1b2"})),
        "{:?}",
        engine.heard()
    );
    assert_eq!(model.roster.len(), 1, "no chunk was invented for the stamp");
}

/// An empty outbox is a pass that does nothing, which is the ordinary case
/// between two keystrokes.
#[test]
fn an_empty_outbox_sends_nothing() {
    let scratch = Scratch::new();
    let link = Link::new(Duration::from_millis(1));
    tick(&link, scratch.path());
    let mut model = Model::default();
    link.settle(&mut model);
    assert_eq!(model.notice, None);
}

/// The lost-reply contract: what an act with no reply paints, and what it
/// never does.
mod doubt;
/// Which channel an answer is filed under — the fanned leg's and the routed
/// gesture's, neither of which is the aim.
mod stamps;

/// **An act's reply is stamped with the op it answers** (bl-b180), which is
/// what lets the model retire a start on its own refusal: the engine refuses
/// the staging act, and the start on this end is refused rather than left
/// outstanding forever with the bar saying something about nothing in
/// particular.
#[test]
fn an_act_s_refusal_reaches_the_start_that_posted_it() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![vec![json!({"ok": false, "error": "sign in first"})]],
    );
    let link = Link::new(Duration::from_millis(1));
    let mut model = Model {
        roster: vec![Chunk::of(own())],
        aim: Some(Aim {
            channel: own().name,
            address: "home".to_owned(),
        }),
        conversation: None,
        draft: "do the thing".to_owned(),
        ..Model::default()
    };
    model.stage("home");
    link.settle(&mut model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert_eq!(
        model.start.as_ref().map(|start| start.phase.clone()),
        Some(crate::ui::model::Phase::Refused("sign in first".to_owned()))
    );
    assert_eq!(model.draft, "do the thing");
    assert_eq!(model.notice, None, "the sentence is the composer's");
}
