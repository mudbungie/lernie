//! **The sixth question, and it is two widths at once** — the ball pane's four
//! reads, which stand on the pane and on nothing else (bl-d2af).
//!
//! A module of its own beside [`super::login`], on the seam that file's doc
//! draws: [`super::nesting`] is *how the three questions nest*, and these are
//! reads that nest under none of them. What is peculiar to this pane is that
//! two of its four fan over every channel and two are asked of the aimed wall,
//! from one standing — so the pass has to put both up together and both down
//! together, and that is what is asserted here.

use super::super::tick;
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};

/// **Nothing is asked about the balls until the pane is open, and then all
/// four are** — the two that name no workspace off the channel loop, and the
/// two that name one off the aim.
#[test]
fn the_ball_pane_s_four_reads_stand_only_while_it_is_open() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "balls", "rows": []})],
            vec![json!({"ok": true, "kind": "board", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "workspace-balls", "rows": []})],
            vec![json!({"ok": true, "kind": "marks", "branch": "balls/tasks"})],
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
    model.begin_board();
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
            "balls",
            "board",
            "conversations",
            "workspace-balls",
            "marks",
        ]
    );
    link.settle(&mut model);
    assert_eq!(model.columns.len(), 1, "the board is filed");
    assert_eq!(model.bindings.len(), 1, "the binding table is filed");
    assert_eq!(
        model.holding,
        Some(Vec::new()),
        "the wall's balls are filed"
    );
    assert_eq!(model.marks.as_deref(), Some("balls/tasks"));
}
