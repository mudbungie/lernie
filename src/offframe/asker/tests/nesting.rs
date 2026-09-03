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

/// **The fourth question is the pane's, not the aim's.** A wall this seat is
/// aimed at is asked nothing about its roles until the tuning pane is open on
/// it — the read is cheap, and a standing question nobody has a use for is one
/// the engine answers on every beat forever.
#[test]
fn the_roles_read_stands_only_while_the_tuning_pane_is_open() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "roles", "rows": []})],
        ],
    );
    let own = Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    };
    let mut model = Model {
        roster: vec![Chunk::of(own.clone())],
        aim: Some(Aim {
            channel: own.name.clone(),
            address: "home".to_owned(),
        }),
        ..Model::default()
    };
    let link = asking(&model);
    tick(&link, scratch.path());
    model.begin_tuning();
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
            "conversations",
            "workspaces",
            "conversations",
            "roles",
        ]
    );
    link.settle(&mut model);
    assert_eq!(model.roles, Some(Vec::new()), "the answer is filed");
}

/// **The queue's read stands on its pane and fans with the roster** (bl-f0ef).
/// It is the one standing question that is nobody's focus — `attention` names
/// no workspace — so it rides beside each channel's roster read rather than
/// under the aim, and a seat with the pane shut asks none of them.
#[test]
fn the_queue_read_stands_only_while_its_pane_is_open_and_goes_down_every_channel() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "attention", "rows": []})],
        ],
    );
    let own = Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    };
    let mut model = Model {
        roster: vec![Chunk::of(own)],
        ..Model::default()
    };
    let link = asking(&model);
    tick(&link, scratch.path());
    // No aim and no selection: the pane opens on a seat that has picked
    // nothing, which is the seat most likely to be asking the question.
    model.begin_queue();
    link.settle(&mut model);
    tick(&link, scratch.path());
    let ops: Vec<String> = engine
        .heard()
        .iter()
        .filter_map(|said| said.get("op").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    assert_eq!(ops, vec!["workspaces", "workspaces", "attention"]);
    link.settle(&mut model);
    assert_eq!(model.waiting.len(), 1, "the answer is filed by channel");
}

/// **The records pair stands on its pane exactly as the roles read does**
/// (bl-2cf7): the selected conversation is asked what its loop did and what
/// its worktree holds only while somebody is looking — and both answers are
/// filed through the one door.
#[test]
fn the_records_reads_stand_only_while_the_records_pane_is_open() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "transcript", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "transcript", "rows": []})],
            vec![json!({"ok": true, "kind": "steps", "rows": [], "orphan": "none"})],
            vec![json!({"ok": true, "kind": "files", "worktree": false})],
        ],
    );
    let own = Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    };
    let mut model = Model {
        roster: vec![Chunk::of(own.clone())],
        aim: Some(Aim {
            channel: own.name.clone(),
            address: "home".to_owned(),
        }),
        conversation: Some("20260830T051200Z-a1b2".to_owned()),
        ..Model::default()
    };
    let link = asking(&model);
    tick(&link, scratch.path());
    model.begin_records();
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
            "conversations",
            "transcript",
            "workspaces",
            "conversations",
            "transcript",
            "steps",
            "files",
        ]
    );
    link.settle(&mut model);
    assert!(
        model.steps.is_some() && model.files.is_some(),
        "both answers are filed"
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
