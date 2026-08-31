//! The three questions, and how each waits for the last to have an answer —
//! plus the aimed read that could not be routed at all.

use super::super::tick;
use super::{asking, reported};
use crate::state::Standing;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};

/// **The three questions nest.** With no aim only the rosters are asked; with
/// an aim the conversations follow; with a conversation the transcript does.
#[test]
fn the_questions_nest_and_each_one_waits_for_the_last_to_have_an_answer() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "transcript", "rows": []})],
        ],
    );
    let own = Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    };
    let mut model = Model {
        roster: vec![Chunk::of(own.clone())],
        ..Model::default()
    };
    let link = asking(&model);
    tick(&link, scratch.path());
    model.aim = Some(Aim {
        channel: own.name.clone(),
        address: "home".to_owned(),
    });
    link.settle(&mut model);
    tick(&link, scratch.path());
    model.conversation = Some("20260830T051200Z-a1b2".to_owned());
    link.settle(&mut model);
    tick(&link, scratch.path());
    let ops: Vec<String> = engine
        .heard()
        .iter()
        .filter_map(|said| said.get("op").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    assert_eq!(
        ops,
        vec![
            "workspaces",
            "workspaces",
            "conversations",
            "workspaces",
            "conversations",
            "transcript",
        ]
    );
}

/// A standing set with nothing in it asks nothing at all — the general path
/// with no input, not a case of its own.
#[test]
fn an_empty_standing_set_asks_nothing() {
    let scratch = Scratch::new();
    let link = asking(&Model::default());
    tick(&link, scratch.path());
    assert!(reported(&link).is_empty());
    assert_eq!(Standing::default().aimed(), None);
}

/// **An aimed read that cannot be routed is this seat's own sentence**, and it
/// is a different one from a refusal: the remedy is this box's files or the far
/// end being up, not typing something else.
#[test]
fn an_aimed_read_that_cannot_be_routed_says_so_as_unreachable() {
    let scratch = Scratch::new();
    let own = Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    };
    let mut model = Model {
        roster: vec![Chunk::of(own.clone())],
        aim: Some(Aim {
            channel: own.name,
            address: "home".to_owned(),
        }),
        ..Model::default()
    };
    let link = asking(&model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    // **On that channel's own section**, which is where a fact about a
    // relationship goes (bl-e620).
    let crate::ui::Held::Unheld(why) = &model.roster[0].held else {
        panic!("the leg said nothing: {:?}", model.roster[0].held);
    };
    assert!(why.contains("no wire provisioned"), "{why}");
    assert_eq!(model.notice, None);
}
