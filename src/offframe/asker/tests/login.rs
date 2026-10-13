//! The fifth question: the provider table, which stands on the aim (bl-7a77)
//! and so under the login pane too.
//!
//! A module of its own rather than a member of [`super::nesting`], which is at
//! the design-time budget — and the split is on the seam that file's own doc
//! draws: that one is *how the three questions nest*, and this is a read that
//! nests under none of them. It is the aimed wall's, keyed on a pane, and its
//! answer's other half is a held lane on a different thread entirely
//! (`crate::offframe::signin`).

use super::super::tick;
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};

/// **The fifth question is the aim's** (bl-7a77) — the roles read's rule
/// one noun over, because the start's hint reads the worker's row in it
/// before any pane is open — and opening the login pane asks nothing more.
/// The standing still buys what it bought there: a credential lands on the
/// ENGINE while the operator is looking at the table, so a row that said
/// *no credential* says otherwise on the next beat with nothing asked again.
#[test]
fn the_provider_table_stands_on_the_aim() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "roles", "rows": []})],
            vec![json!({"ok": true, "kind": "providers", "rows": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "roles", "rows": []})],
            vec![json!({"ok": true, "kind": "providers", "rows": []})],
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
    model.begin_login();
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
            "roles",
            "providers",
            "workspaces",
            "conversations",
            "roles",
            "providers",
        ]
    );
    link.settle(&mut model);
    assert_eq!(model.providers, Some(Vec::new()), "the answer is filed");
}
