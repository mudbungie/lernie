//! The seventh question: the config pane's lineage listing and its file read —
//! and the file read that addresses a CHANNEL rather than a workspace.
//!
//! A module of its own for [`super::login`]'s reason, and it carries the one
//! beat no other pass has: three of the five config destinations name no
//! workspace at all, so the read is asked down the aimed channel by name
//! instead of being routed by an address it does not have.

use super::super::tick;
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::wire::{entry, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use crate::verbs::Where;
use serde_json::{Value, json};

/// The seated model aimed at an ENTRY's wall, with nothing provisioned at the
/// flat root — so a gesture that fell through to this box's own engine would
/// reach nothing, which is what makes the channel-addressed read visible.
fn aimed_at_an_entry() -> Model {
    let held = Channel {
        name: "b".to_owned(),
        named_there: Some("b".to_owned()),
        dials: None,
    };
    Model {
        roster: vec![Chunk::of(held.clone())],
        aim: Some(Aim {
            channel: held.name,
            address: "b".to_owned(),
        }),
        ..Model::default()
    }
}

/// Every op one pass asked, in order.
fn asked(engine: &crate::test_support::engine::Engine) -> Vec<String> {
    engine
        .heard()
        .iter()
        .filter_map(|said| said.get("op").and_then(Value::as_str))
        .map(str::to_owned)
        .collect()
}

/// **The lineage listing stands on the pane and the file read stands on the
/// destination** — the login pane's rule with one more rung: a pane open on no
/// file asks for the listing alone, because there is nothing to read yet.
#[test]
fn the_listing_stands_on_the_pane_and_the_file_read_on_the_destination() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &entry("b"),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "lineages", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "lineages", "rows": []})],
            vec![json!({"ok": true, "kind": "config", "text": "x", "settings": []})],
        ],
    );
    let mut model = aimed_at_an_entry();
    let link = asking(&model);
    tick(&link, scratch.path());
    model.begin_configuring();
    link.settle(&mut model);
    tick(&link, scratch.path());
    assert_eq!(
        asked(&engine),
        vec![
            "workspaces",
            "conversations",
            "workspaces",
            "conversations",
            "lineages"
        ],
        "the listing stands on the pane, and no file has been picked"
    );
    model.read_config(&Where::Brazen {
        workspace: "b".to_owned(),
    });
    link.settle(&mut model);
    tick(&link, scratch.path());
    assert_eq!(asked(&engine).last().map(String::as_str), Some("config"));
    link.settle(&mut model);
    assert_eq!(
        model.config.map(|held| held.text),
        Some("x".to_owned()),
        "the answer is filed"
    );
    assert_eq!(model.lineages, Some(Vec::new()));
}

/// **A destination that names no workspace addresses the CHANNEL** (DESIGN
/// §4.30). Nothing is provisioned at the flat root here, so a gesture routed by
/// its absent address would have reached nothing at all; it reaches the entry
/// the window is aimed at, which is the one this pane is about.
#[test]
fn a_destination_naming_no_workspace_is_asked_down_the_aimed_channel() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &entry("b"),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "lineages", "rows": []})],
            vec![json!({"ok": true, "kind": "config", "text": "cadence:\n", "settings": []})],
        ],
    );
    let mut model = aimed_at_an_entry();
    model.begin_configuring();
    model.read_config(&Where::Cadence);
    let link = asking(&model);
    tick(&link, scratch.path());
    assert_eq!(
        asked(&engine),
        vec!["workspaces", "conversations", "lineages", "config"],
        "the engine the window is aimed at answered it"
    );
    link.settle(&mut model);
    assert_eq!(
        model.config.map(|held| held.text),
        Some("cadence:\n".to_owned())
    );
    assert_eq!(
        model.notice, None,
        "and nothing fell through to a flat root"
    );
}
