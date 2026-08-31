//! The follow lane: the held read, the frame that arrives for a subject the
//! operator has left, and the two halves that can move.

use std::time::Duration;

use super::{still_on, tick};
use crate::state::Link;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::json;

fn own() -> Channel {
    Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    }
}

/// A model looking at one conversation on this box's own engine.
fn watching() -> Model {
    Model {
        roster: vec![Chunk::of(own())],
        aim: Some(Aim {
            channel: own().name,
            address: "home".to_owned(),
        }),
        conversation: Some("20260830T051200Z-a1b2".to_owned()),
        ..Model::default()
    }
}

fn linked(model: &Model) -> Link {
    let link = Link::new(Duration::from_millis(1));
    let mut model = model.clone();
    link.settle(&mut model);
    link
}

/// **Every frame of a read is absorbed, in order, onto an empty fold** (yog's
/// REMOTE §5.5). A frame carries what landed since the previous one, so the
/// answer is this end's accumulation and never the last frame alone — a seat
/// that took the newest frame whole would paint the tail of a sentence over
/// the front of it.
#[test]
fn every_frame_of_the_held_read_is_absorbed_onto_an_empty_fold() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![vec![
            json!({"ok": true, "kind": "follow",
                   "stream": {"text": "half a ", "delta": "text"}}),
            json!({"ok": true, "kind": "follow",
                   "stream": {"thinking": "hm. ", "delta": "thinking"}}),
            json!({"ok": true, "kind": "follow",
                   "stream": {"text": "sentence", "delta": "text"}}),
        ]],
    );
    let mut model = watching();
    let link = linked(&model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    let live = model.live.expect("the accumulated tail");
    assert_eq!(live.text.as_deref(), Some("half a sentence"));
    assert_eq!(
        live.thinking.as_deref(),
        Some("hm. "),
        "the two kinds accrete apart, and a frame carrying only one leaves the \
         other standing"
    );
    assert_eq!(
        live.last_delta,
        Some(crate::reply::stream::Delta::Text),
        "the newer delta kind wins"
    );
    assert!(
        engine
            .heard()
            .contains(&json!({"op": "follow", "workspace": "home",
                                        "agent": "20260830T051200Z-a1b2"})),
        "{:?}",
        engine.heard()
    );
}

/// **A read starts holding nothing**, so two reads of one conversation do not
/// accrete into each other. The engine's reader is minted per held connection
/// and opens the response at byte zero (REMOTE §5.5), which is why the fold's
/// whole lifetime is one read: a seat that dropped a connection mid-answer
/// re-asks and is whole on its first frame, with nothing to reconcile.
#[test]
fn a_second_read_replaces_rather_than_appending() {
    let scratch = Scratch::new();
    let _engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "follow",
                        "stream": {"text": "the whole answer", "delta": "text"}})],
            vec![json!({"ok": true, "kind": "follow",
                        "stream": {"text": "the whole answer, and more", "delta": "text"}})],
        ],
    );
    let mut model = watching();
    let link = linked(&model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert_eq!(
        model.live.and_then(|s| s.text).as_deref(),
        Some("the whole answer, and more"),
        "the second read's first frame is the whole tail, absorbed onto nothing"
    );
}

/// **Both halves of the subject can move**, and either ends the read: an
/// operator who aims at another wall has left this conversation as surely as
/// one who picks another conversation on the same wall.
#[test]
fn the_read_is_about_a_wall_and_a_conversation_together() {
    let model = watching();
    let link = linked(&model);
    let aim = model.aim.clone().expect("an aim");
    let conversation = model.conversation.clone().expect("a conversation");
    assert!(still_on(&link, &aim, &conversation));
    assert!(!still_on(&link, &aim, "somebody-else"));
    let elsewhere = Aim {
        channel: own().name,
        address: "other".to_owned(),
    };
    assert!(!still_on(&link, &elsewhere, &conversation));
    let idle = linked(&Model::default());
    assert!(!still_on(&idle, &aim, &conversation));
}

/// **A tail for a conversation the operator has left is dropped at the
/// settle**, where what is selected is known for certain. Filing it would paint
/// one conversation's words under another's name, and the lane that read it
/// cannot know: it is parked on a socket that was asked about the old one.
#[test]
fn a_tail_for_a_conversation_the_operator_left_is_dropped() {
    let link = Link::new(Duration::from_millis(1));
    link.live(
        &own(),
        "the-one-that-was-left",
        crate::reply::read(&json!({"ok": true, "kind": "follow", "stream": {"text": "stale"}})),
    );
    link.live(
        &own(),
        "20260830T051200Z-a1b2",
        crate::reply::read(&json!({"ok": true, "kind": "follow", "stream": {"text": "wanted"}})),
    );
    let mut model = watching();
    link.settle(&mut model);
    assert_eq!(
        model.live.and_then(|s| s.text),
        Some("wanted".to_owned()),
        "only the tail about what is selected is filed"
    );
}

/// With nothing selected there is no held read at all — the general path with
/// no subject, not a case of its own.
#[test]
fn with_no_conversation_there_is_no_held_read() {
    let scratch = Scratch::new();
    let mut watching = watching();
    watching.conversation = None;
    for model in [Model::default(), watching] {
        let link = linked(&model);
        tick(&link, scratch.path());
        let mut settled = model.clone();
        link.settle(&mut settled);
        assert_eq!(settled.notice, None);
        assert_eq!(settled.live, None);
    }
}

/// A far end that is not there is this seat's own sentence.
#[test]
fn a_lane_that_cannot_be_opened_says_so_as_unreachable() {
    let scratch = Scratch::new();
    let model = watching();
    let link = linked(&model);
    tick(&link, scratch.path());
    let mut settled = model.clone();
    link.settle(&mut settled);
    let crate::ui::Held::Unheld(why) = &settled.roster[0].held else {
        panic!("the lane said nothing: {:?}", settled.roster[0].held);
    };
    assert!(why.contains("no wire provisioned"), "{why}");
    assert_eq!(settled.notice, None, "not the shell-wide bar (bl-e620)");
}

/// **Only the tail is absorbed.** A held read can answer something that is not
/// a tail at all — the engine refusing mid-stream, or bytes this build cannot
/// read — and those have nothing to accumulate onto, so they cross the lane
/// untouched and reach the operator as themselves. A lane that folded
/// everything would have to invent a reading for a refusal.
#[test]
fn a_held_read_that_answers_something_other_than_a_tail_crosses_untouched() {
    let scratch = Scratch::new();
    let _engine = wired(
        &scratch,
        &flat(),
        vec![vec![
            json!({"ok": true, "kind": "follow", "stream": {"text": "said. "}}),
            json!({"ok": false, "error": "the driver is gone"}),
        ]],
    );
    let mut model = watching();
    let link = linked(&model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert_eq!(
        model.notice,
        Some(crate::ui::Notice::Refused("the driver is gone".to_owned())),
        "the engine's own words, not a fold"
    );
    assert_eq!(
        model.live.and_then(|s| s.text).as_deref(),
        Some("said. "),
        "and what the tail had already accumulated is still standing"
    );
}
