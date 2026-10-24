//! The records pane's reads stand on their pane (bl-2cf7) — split from
//! [`super::nesting`] at the design-time budget (bl-1d79).

use super::super::tick;
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};

/// **The records pair stands on its pane exactly as the roles read does**
/// (bl-2cf7): the selected conversation is asked what its loop did and what
/// its worktree holds only while somebody is looking — and both answers are
/// filed through the one door.
#[test]
fn the_records_reads_stand_only_while_the_records_pane_is_open() {
    let scratch = Scratch::new();
    let server = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "transcript", "rows": []})],
            vec![json!({"ok": true, "kind": "refused", "why": "no such agent"})],
            vec![json!({"ok": true, "kind": "rail", "rows": [], "cards": []})],
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "roles", "rows": []})],
            vec![json!({"ok": true, "kind": "providers", "rows": []})],
            vec![json!({"ok": true, "kind": "transcript", "rows": []})],
            vec![json!({"ok": true, "kind": "refused", "why": "no such agent"})],
            vec![json!({"ok": true, "kind": "rail", "rows": [], "cards": []})],
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
    let ops: Vec<String> = server
        .heard()
        .iter()
        .filter_map(|said| said.get("op").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    // **`agent` rides with the transcript on both passes** (bl-7b03): the
    // conversation pane's header needs the row whether or not this pane is
    // open, so it is the selection's read and no longer this pane's.
    assert_eq!(
        ops,
        vec![
            "transcript",
            "agent",
            "rail",
            "workspaces",
            "conversations",
            "roles",
            "providers",
            "transcript",
            "agent",
            "rail",
            "steps",
            "files",
        ]
    );
    link.settle(&mut model);
    assert!(
        model.records.steps.is_some() && model.records.files.is_some(),
        "both answers are filed"
    );
}
