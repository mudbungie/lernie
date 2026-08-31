//! The poster's pass: what it sends, what it stamps a receipt with, and what a
//! channel that will not open costs.

use std::time::Duration;

use super::tick;
use crate::state::Link;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};

fn own() -> Channel {
    Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    }
}

/// A link holding one composed gesture, aimed or not.
fn holding(aim: Option<Aim>, envelope: Value) -> (Link, Model) {
    let link = Link::new(Duration::from_millis(1));
    let mut model = Model {
        roster: vec![Chunk::of(own())],
        aim,
        outbox: vec![envelope],
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
/// what routes it, and the address is the whole of what routing needs. What it
/// cannot have is an honest channel stamp, so it gets one that names nothing
/// rather than one this seat invented.
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

/// A far end that is not there is this seat's own sentence, not the engine's.
#[test]
fn an_act_that_could_not_be_sent_says_so_as_unreachable() {
    let scratch = Scratch::new();
    let (link, mut model) = holding(
        None,
        crate::verbs::nudge("home".to_owned(), "a1b2".to_owned()),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(
        model
            .notice
            .expect("a notice")
            .line()
            .contains("could not reach")
    );
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
