//! **The seventh question, and both halves of it are derived when asked** —
//! the fleet pane's two reads, which stand on the pane and on nothing else
//! (bl-a43a).
//!
//! A module of its own beside [`super::login`] and [`super::board`], on the
//! seam those files' docs draw. What is peculiar to this pair is *why* it
//! stands: neither `science` nor `work-diff` is stored anywhere, so each is a
//! statement about the moment it was asked, and a read posted once would paint
//! a moment that had passed.

use super::super::tick;
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};

/// **Nothing is asked about the agents until the pane is open, and then both
/// are** — of the aimed wall, because every one of the pane's seven ops names
/// a workspace.
#[test]
fn the_fleet_pane_s_two_reads_stand_only_while_it_is_open() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "science", "rows": []})],
            vec![json!({"ok": true, "kind": "work-diff", "rows": []})],
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
    model.begin_fleet();
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
            "science",
            "work-diff",
        ]
    );
    link.settle(&mut model);
    assert_eq!(model.attempts, Some(Vec::new()), "the attempts are filed");
    assert_eq!(model.work, Some(Vec::new()), "what changed is filed");
}
