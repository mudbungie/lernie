//! The sixth question: the machines registered in the aimed wall's workspace,
//! which stands on the clients pane and on nothing else.
//!
//! A module of its own for [`super::login`]'s reason exactly — [`super::nesting`]
//! is at the design-time budget, and this read nests under none of the three
//! questions there: it is the aimed wall's, keyed on a pane.

use super::super::tick;
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};

/// **The sixth question is the pane's, not the aim's** — the provider table's
/// rule one noun over. And the standing buys the sharper form of what it buys
/// there: a row's presence is true only at the instant the engine answered it
/// (REMOTE §5), so a read saying a foot is connected is worth nothing unless it
/// is asked again.
#[test]
fn the_machines_read_stands_only_while_the_clients_pane_is_open() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "clients", "rows": [
                {"client": "laptop", "present": true, "tools": []}]})],
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
    model.begin_clients();
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
            "clients",
        ]
    );
    link.settle(&mut model);
    assert_eq!(
        model
            .machines
            .as_ref()
            .map(|rows| rows.iter().map(|row| row.client.clone()).collect()),
        Some(vec!["laptop".to_owned()]),
        "the answer is filed"
    );
}
