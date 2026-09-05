//! **Which channel an answer is filed under**, which is the whole of what this
//! pass stamps: a fanned leg's is the channel that leg opened, and a routed
//! gesture's is the channel `crate::seat::route` chose (bl-c70d). Neither is
//! the aim, which is only where the gesture was composed.
//!
//! Split from [`super`] at the design-time budget on the seam the subject
//! already has: [`super`] is what a pass sends and what a receipt does to the
//! model, and this is where each answer lands.

use super::{own, posting, tick};
use crate::state::Link;
use crate::test_support::Scratch;
use crate::test_support::wire::{entry, flat, wired, yes};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};
use std::time::Duration;

/// A link whose roster holds this box's own engine **and one entry beside
/// it** — the two-channel arrangement a stamp taken off the aim is only
/// visible on.
fn beside(aim: Option<Aim>, posted: crate::ui::Posted) -> (Link, Model) {
    let link = Link::new(Duration::from_millis(1));
    let mut model = Model {
        roster: vec![
            Chunk::of(own()),
            Chunk::of(Channel {
                name: "b".to_owned(),
                named_there: Some("b".to_owned()),
                dials: None,
            }),
        ],
        aim,
        outbox: vec![posted],
        ..Model::default()
    };
    link.settle(&mut model);
    (link, model)
}

/// One conversation still asking, which is what `seen` answers with: the queue
/// that REMAINS, filed per channel.
fn asking() -> Value {
    json!({"ok": true, "kind": "attention", "rows": [
        {"workspace": "b", "agent": "c-2", "display": "Dun", "state": "live",
         "uncertain": false, "preview": "", "age_secs": 0, "pending": 0,
         "signals": [], "failure": null, "flag": null, "held": null}]})
}

/// **A gesture naming no workspace is FANNED, not routed** (bl-40ec): it has
/// no way to name a channel, so its subject is every channel the standing set
/// holds — and each answer is stamped with the channel it came down rather
/// than with the aim.
///
/// Routed instead, it would have gone to the flat root alone and said nothing
/// about the rest, which is bl-0d54's defect one surface over.
#[test]
fn a_gesture_naming_no_workspace_is_asked_of_every_channel() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![vec![json!({"ok": true, "kind": "help", "rows": [
            {"verb": "scan", "usage": "/scan", "summary": "sweep",
             "detail": "one sweep", "surface": "control"}]})]],
    );
    let (link, mut model) = posting(None, crate::ui::Posted::read(crate::verbs::window::help()));
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(
        engine.heard().contains(&json!({"op": "help"})),
        "{:?}",
        engine.heard()
    );
    assert_eq!(
        model.pages.len(),
        1,
        "the answer landed under the channel it came down"
    );
    assert_eq!(model.pages[0].channel.name, own().name);
}

/// And a channel that will not open costs only itself — reported against that
/// channel's own section rather than against the aim.
#[test]
fn a_fanned_leg_that_reached_nothing_is_that_channels_own_sentence() {
    let scratch = Scratch::new();
    let (link, mut model) = posting(None, crate::ui::Posted::read(crate::verbs::workspaces()));
    tick(&link, scratch.path());
    link.settle(&mut model);
    let crate::ui::Held::Unheld(why) = &model.roster[0].held else {
        panic!(
            "the leg's failure lands on its own section: {:?}",
            model.roster[0].held
        );
    };
    assert!(!why.is_empty(), "{why}");
    assert_eq!(model.notice, None, "and not in the shell-wide bar");
}

/// **A receipt is stamped with the channel the gesture was ROUTED down, never
/// with the one it was aimed from** (bl-c70d).
///
/// An aim is where a gesture was *composed*. An operator may compose one aimed
/// at a wall on one channel while a control fires at a row on another — the
/// decision queue is the pane where that is ordinary, since its rows come from
/// every channel at once — and `seen` answers with the queue that REMAINS,
/// which is filed per channel. Stamped with the aim, one channel's slice
/// replaces another's.
#[test]
fn a_receipt_is_stamped_with_the_channel_it_crossed_and_not_the_aim() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![vec![yes()]]);
    wired(&scratch, &entry("b"), vec![vec![asking()]]);
    let (link, mut model) = beside(
        Some(Aim {
            channel: own().name,
            address: "home".to_owned(),
        }),
        crate::ui::Posted::act(crate::verbs::seen("b".to_owned(), "c-1".to_owned())),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert_eq!(
        model
            .waiting
            .iter()
            .map(|held| held.channel.name.clone())
            .collect::<Vec<String>>(),
        vec!["b".to_owned()],
        "the slice belongs to the channel that answered it"
    );
}

/// And a leg that reached nothing is filed the same way: [`crate::seat::Routed`]
/// answers the seat-side name whether or not anything opened, so a read that
/// could not be routed lands on the section it would have crossed on rather
/// than on the aim's.
#[test]
fn a_routed_leg_that_reached_nothing_is_that_channels_own_sentence() {
    let scratch = Scratch::new();
    wired(&scratch, &flat(), vec![vec![yes()]]);
    std::fs::create_dir_all(scratch.path().join(entry("b"))).expect("mkdir");
    let (link, mut model) = beside(
        Some(Aim {
            channel: own().name,
            address: "home".to_owned(),
        }),
        crate::ui::Posted::read(crate::verbs::models("b".to_owned(), "vendor".to_owned())),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(
        matches!(model.roster[1].held, crate::ui::Held::Unheld(_)),
        "the hollow entry's own section says so: {:?}",
        model.roster[1].held
    );
    assert!(
        matches!(model.roster[0].held, crate::ui::Held::Unheard),
        "and the aim's is untouched: {:?}",
        model.roster[0].held
    );
}

/// **A gesture naming no workspace that names a CHANNEL goes down that channel
/// alone** (bl-4855) — the fan is what *addressed to no channel in particular*
/// means, and it was never what every workspace-less gesture means.
///
/// `config` on one engine's own `cadence.yaml` names no workspace and is not
/// about every engine. Fanned, it would have written this operator's text onto
/// every engine this box is a client of.
#[test]
fn an_addressed_gesture_goes_down_the_one_channel_and_no_other() {
    let scratch = Scratch::new();
    let own_engine = wired(&scratch, &flat(), vec![vec![yes()]]);
    let other = wired(&scratch, &entry("b"), vec![vec![yes()]]);
    let addressed = Channel {
        name: "b".to_owned(),
        named_there: Some("b".to_owned()),
        dials: None,
    };
    let (link, mut model) = beside(
        Some(Aim {
            channel: own().name,
            address: "home".to_owned(),
        }),
        crate::ui::Posted::act(crate::verbs::write(
            &crate::verbs::Where::Cadence,
            "beat: 1\n".to_owned(),
        ))
        .down(addressed),
    );
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(
        other
            .heard()
            .contains(&json!({"op": "config", "target": {"file": "cadence"},
                              "text": "beat: 1\n"})),
        "the channel it named heard it: {:?}",
        other.heard()
    );
    assert!(
        !own_engine
            .heard()
            .iter()
            .any(|said| said.get("op").is_some()),
        "and no other engine was told anything at all: {:?}",
        own_engine.heard()
    );
}
