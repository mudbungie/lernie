//! The sign-in lane: the held read, the frame that arrives for a row the
//! operator has stopped following, and the two halves that can move.

use std::time::Duration;

use super::{still_on, tick};
use crate::state::Link;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Login, Model};
use serde_json::json;

fn own() -> Channel {
    Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    }
}

/// A model with the login pane open on a wall and one row being followed.
fn watching() -> Model {
    Model {
        roster: vec![Chunk::of(own())],
        aim: Some(Aim {
            channel: own().name,
            address: "home".to_owned(),
        }),
        login: Some(Login {
            following: Some("housevendor".to_owned()),
            asking: None,
        }),
        ..Model::default()
    }
}

fn linked(model: &Model) -> Link {
    let link = Link::new(Duration::from_millis(1));
    let mut model = model.clone();
    link.settle(&mut model);
    link
}

/// **Every frame of a read is absorbed, in order, onto an empty fold** (REMOTE
/// §8.3). Each carries what the run said since the previous one, so the answer
/// is this end's accumulation — a seat that took the newest frame whole would
/// paint the end of a sign-in over the URL that started it.
#[test]
fn every_frame_of_the_held_read_is_absorbed_onto_an_empty_fold() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![vec![
            json!({"ok": true, "kind": "login",
                   "lines": [{"text": "open https://provider.invalid/auth", "err": true}]}),
            json!({"ok": true, "kind": "login",
                   "lines": [{"text": "code ABCD-EFGH", "err": true}]}),
            json!({"ok": true, "kind": "login", "lines": [], "outcome": 0}),
        ]],
    );
    let mut model = watching();
    let link = linked(&model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    let run = model.signin.expect("the accumulated run");
    assert_eq!(run.lines.len(), 2, "the two lines accrete in order");
    assert_eq!(run.lines[0].text, "open https://provider.invalid/auth");
    assert_eq!(run.lines[1].text, "code ABCD-EFGH");
    assert_eq!(run.outcome, Some(0));
    assert!(run.settled());
    assert!(
        engine
            .heard()
            .contains(&json!({"op": "login-tail", "workspace": "home", "provider": "housevendor"})),
        "{:?}",
        engine.heard()
    );
}

/// **A read starts holding nothing**, so re-asking replays rather than
/// doubling: the engine's own cursor is per read and opens at zero.
#[test]
fn a_second_read_replaces_rather_than_appending() {
    let scratch = Scratch::new();
    let _engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "login",
                        "lines": [{"text": "one", "err": false}]})],
            vec![json!({"ok": true, "kind": "login",
                        "lines": [{"text": "one", "err": false},
                                  {"text": "two", "err": false}]})],
        ],
    );
    let mut model = watching();
    let link = linked(&model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert_eq!(
        model.signin.expect("the run").lines.len(),
        2,
        "the second read's first frame is the whole buffer, absorbed onto nothing"
    );
}

/// **Both halves of the subject can move**, and either ends the read: aiming at
/// another wall retires the pane outright, and starting a sign-in on another
/// row replaces the run upstream.
#[test]
fn the_read_is_about_a_wall_and_a_provider_row_together() {
    let model = watching();
    let link = linked(&model);
    let aim = model.aim.clone().expect("an aim");
    assert!(still_on(&link, &aim, "housevendor"));
    assert!(!still_on(&link, &aim, "otherhouse"));
    let elsewhere = Aim {
        channel: own().name,
        address: "other".to_owned(),
    };
    assert!(!still_on(&link, &elsewhere, "housevendor"));
    let idle = linked(&Model::default());
    assert!(!still_on(&idle, &aim, "housevendor"));
}

/// **A frame for a row the operator has stopped following is dropped at the
/// settle**, where what the pane is following is known for certain. Filing it
/// would paint one run's lines under another row's name.
#[test]
fn a_frame_for_a_row_the_operator_left_is_dropped() {
    let link = Link::new(Duration::from_millis(1));
    let stale = json!({"ok": true, "kind": "login",
                       "lines": [{"text": "stale", "err": false}]});
    let wanted = json!({"ok": true, "kind": "login",
                        "lines": [{"text": "wanted", "err": false}]});
    link.signing(&own(), "otherhouse", crate::reply::read(&stale));
    link.signing(&own(), "housevendor", crate::reply::read(&wanted));
    let mut model = watching();
    link.settle(&mut model);
    let run = model.signin.expect("the run");
    assert_eq!(run.lines.len(), 1);
    assert_eq!(run.lines[0].text, "wanted");
}

/// With no row followed there is no held read at all — the general path with no
/// subject, not a case of its own.
#[test]
fn with_no_row_followed_there_is_no_held_read() {
    let scratch = Scratch::new();
    let mut shut = watching();
    shut.login = None;
    let mut open = watching();
    open.login = Some(Login::default());
    for model in [Model::default(), shut, open] {
        let link = linked(&model);
        tick(&link, scratch.path());
        let mut settled = model.clone();
        link.settle(&mut settled);
        assert_eq!(settled.notice, None);
        assert_eq!(settled.signin, None);
    }
}

/// A far end that is not there is this seat's own sentence, on that channel's
/// own section rather than in the shell-wide bar (bl-e620).
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
    assert_eq!(settled.notice, None);
}

/// **Only the run is absorbed.** A held read can answer something that is not
/// a run — the engine refusing mid-stream — and that has nothing to accumulate
/// onto, so it crosses untouched and reaches the operator as itself.
#[test]
fn a_held_read_that_answers_something_other_than_a_run_crosses_untouched() {
    let scratch = Scratch::new();
    let _engine = wired(
        &scratch,
        &flat(),
        vec![vec![
            json!({"ok": true, "kind": "login",
                   "lines": [{"text": "said", "err": false}]}),
            json!({"ok": false, "error": "unknown provider"}),
        ]],
    );
    let mut model = watching();
    let link = linked(&model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert_eq!(
        model.notice,
        Some(crate::ui::Notice::Refused("unknown provider".to_owned())),
        "the engine's own words, not a fold"
    );
    assert_eq!(
        model.signin.expect("the run").lines.len(),
        1,
        "and what the run had already said is still standing"
    );
}
