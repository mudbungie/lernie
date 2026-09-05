//! The union across channels: every channel asked for its own roster, each
//! answer stamped with the channel it came down, a channel that will not
//! answer costing only itself, and the trail — the other read whose subject is
//! every channel and which nests under nothing.

use super::super::tick;
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::wire::{entry, flat, wired};
use crate::ui::Model;
use serde_json::json;

/// **The roster is a union across channels, composed here.** Every channel is
/// asked for its own, each answer is stamped with the channel it came down, and
/// no origin crosses the wire.
#[test]
fn every_channel_is_asked_for_its_own_roster_and_stamped_with_it() {
    let scratch = Scratch::new();
    let roster = |name: &str| {
        json!({"ok": true, "kind": "workspaces",
               "rows": [{"workspace": name, "kind": "named", "attention": 0,
                         "agents": 1, "running": false}]})
    };
    wired(&scratch, &flat(), vec![vec![roster("here")]]);
    wired(&scratch, &entry("home"), vec![vec![roster("personal")]]);
    let model = Model {
        roster: crate::seat::channels(scratch.path()),
        ..Model::default()
    };
    let link = asking(&model);
    tick(&link, scratch.path());
    let mut settled = model.clone();
    link.settle(&mut settled);
    let seen: Vec<(String, String)> = settled
        .roster
        .iter()
        .map(|chunk| {
            (
                chunk.channel.name.clone(),
                chunk
                    .walls
                    .first()
                    .map(|row| row.workspace.clone())
                    .unwrap_or_default(),
            )
        })
        .collect();
    assert_eq!(
        seen,
        vec![
            (crate::seat::OWN.to_owned(), "here".to_owned()),
            ("home".to_owned(), "personal".to_owned()),
        ]
    );
}

/// **A channel that will not answer costs only itself.** A box holding two with
/// one engine down still gets the one that is up.
#[test]
fn a_channel_that_will_not_answer_leaves_the_others_standing() {
    let scratch = Scratch::new();
    wired(
        &scratch,
        &flat(),
        vec![vec![json!({"ok": true, "kind": "workspaces", "rows": []})]],
    );
    // An entry directory with nothing in it: a stated intent with no material.
    std::fs::create_dir_all(scratch.path().join(entry("hollow"))).expect("mkdir");
    let model = Model {
        roster: crate::seat::channels(scratch.path()),
        ..Model::default()
    };
    let link = asking(&model);
    tick(&link, scratch.path());
    let mut settled = model.clone();
    link.settle(&mut settled);
    assert_eq!(settled.roster.len(), 2);
    // **The one that failed says so on its OWN section**, and the one that
    // answered is untouched beside it (bl-e620).
    let hollow = settled
        .roster
        .iter()
        .find(|chunk| chunk.channel.name == "hollow")
        .expect("the entry has a section");
    let crate::ui::Held::Unheld(why) = &hollow.held else {
        panic!("the hollow entry said nothing: {:?}", hollow.held);
    };
    assert!(why.contains("empty entry"), "{why}");
    assert_eq!(settled.notice, None, "and nothing reached the shell's bar");
}

/// **The trail's read stands on its pane and goes down every channel**
/// (bl-4c48). `ops` names no workspace, so it rides beside each channel's
/// roster read rather than under the aim — and a seat with the pane shut asks
/// none of them, because a standing question nobody has a use for is one the
/// engine answers on every beat forever.
///
/// It is here rather than beside the nesting tests because it nests under
/// nothing: this is the union's own shape, one noun over from the queue's.
#[test]
fn the_trail_read_stands_only_while_its_pane_is_open_and_goes_down_every_channel() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "ops", "rows": []})],
        ],
    );
    let own = crate::ui::Channel {
        name: crate::seat::OWN.to_owned(),
        named_there: None,
        dials: None,
    };
    let mut model = Model {
        roster: vec![crate::ui::Chunk::of(own)],
        ..Model::default()
    };
    let link = asking(&model);
    tick(&link, scratch.path());
    // No aim and no selection: the pane opens on a seat that has picked
    // nothing, which is the seat most likely to be asking the question.
    model.begin_trail();
    link.settle(&mut model);
    tick(&link, scratch.path());
    let asked: Vec<serde_json::Value> = engine.heard();
    let ops: Vec<String> = asked
        .iter()
        .filter_map(|said| said.get("op").and_then(serde_json::Value::as_str))
        .map(str::to_owned)
        .collect();
    assert_eq!(ops, vec!["workspaces", "workspaces", "ops"]);
    // **The depth is on the envelope**, because the wire requires it: yog's
    // *"defaults to the last 50"* is its own line grammar's, on a surface this
    // seat is not.
    let read = asked
        .iter()
        .find(|said| said.get("op").and_then(serde_json::Value::as_str) == Some("ops"))
        .expect("the trail was asked for");
    assert_eq!(
        read.get("max").and_then(serde_json::Value::as_u64),
        Some(crate::verbs::DEPTH)
    );
    link.settle(&mut model);
    assert_eq!(model.trails.len(), 1, "the answer is filed by channel");
}
