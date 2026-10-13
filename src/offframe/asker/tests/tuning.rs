//! The tuning pane's offerings: each provider row its roles name is asked what
//! it offers on the roles read's standing, and each answer is filed under the
//! row it was asked of (bl-b7d9).

use super::super::tick;
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::window::{role, tuned};
use crate::test_support::wire::{flat, wired};
use crate::ui::Model;
use serde_json::{Value, json};

/// **The rows the roles name are asked, and the answer is stamped with the
/// row** — the reply names none, so nothing else could say whose it is.
#[test]
fn each_provider_row_the_roles_name_is_asked_and_its_answer_filed_under_it() {
    let scratch = Scratch::new();
    let engine = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "workspaces", "rows": []})],
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "roles", "rows": []})],
            vec![json!({"ok": true, "kind": "models", "rows": ["house-model-2"]})],
        ],
    );
    // No conversation selected, so the pass asks the wall's questions alone.
    let mut model = Model {
        conversation: None,
        ..tuned()
    };
    let link = asking(&model);
    tick(&link, scratch.path());
    let heard = engine.heard();
    let ops: Vec<&str> = heard
        .iter()
        .filter_map(|said| said.get("op").and_then(Value::as_str))
        .collect();
    assert_eq!(ops, ["workspaces", "conversations", "roles", "models"]);
    let asked = heard.iter().find(|said| said["op"] == "models");
    assert_eq!(
        asked.map(|said| &said["provider"]),
        Some(&json!("housevendor"))
    );
    link.settle(&mut model);
    assert_eq!(
        model.unoffered(&role("worker")).as_deref(),
        Some("housevendor does not offer house-model-1")
    );
}

/// **A row that cannot be asked is the channel's own sentence** — the aimed
/// read's rule — and files no listing.
#[test]
fn an_offering_that_cannot_be_routed_files_nothing() {
    let scratch = Scratch::new();
    let mut model = tuned();
    let link = asking(&model);
    tick(&link, scratch.path());
    link.settle(&mut model);
    assert!(model.offers.is_empty());
    assert!(matches!(model.roster[0].held, crate::ui::Held::Unheld(_)));
}
