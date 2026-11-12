//! The pending offers: the aimed workspace's, standing on the offers page
//! and on nothing else (DESIGN §4.51) — [`super::clients`]'s shape one noun
//! over.

use super::super::tick;
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::wire::{flat, wired};
use crate::ui::{Aim, Channel, Chunk, Model};
use serde_json::{Value, json};

/// **`offers` is asked while the page stands and never before**, and its
/// answer is the page's rows — so a device that claims and dials moves its
/// row with nothing clicked here.
#[test]
fn the_offers_read_stands_only_while_the_offers_page_is_open() {
    let scratch = Scratch::new();
    let quiet = |kind: &str| vec![json!({"ok": true, "kind": kind, "rows": []})];
    let server = wired(
        &scratch,
        &flat(),
        vec![
            quiet("workspaces"),
            quiet("conversations"),
            quiet("roles"),
            quiet("providers"),
            quiet("workspaces"),
            quiet("conversations"),
            quiet("roles"),
            quiet("providers"),
            vec![json!({"ok": true, "kind": "offers", "rows": [
                {"name": "builder", "grade": "foot", "state": "proved",
                 "expires": 1_700_000_600, "word": "2LFTUR"}]})],
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
    model.begin_offers();
    link.settle(&mut model);
    tick(&link, scratch.path());
    let ops: Vec<String> = server
        .heard()
        .iter()
        .filter_map(|said| said.get("op").and_then(Value::as_str))
        .map(str::to_owned)
        .collect();
    assert_eq!(ops.last().map(String::as_str), Some("offers"));
    assert_eq!(ops.iter().filter(|op| *op == "offers").count(), 1);
    link.settle(&mut model);
    let rows = model.pending.clone().expect("the answer is filed");
    assert_eq!(rows.len(), 1);
    assert!(rows.iter().all(crate::reply::offers::OfferRow::proved));
    // The two verdicts' receipts are the bar's, a sentence each.
    for kind in ["approved", "declined"] {
        model.receipt(
            &own,
            kind,
            crate::reply::read(&json!({"ok": true, "kind": kind})),
        );
        assert!(model.notice.is_some(), "{kind}");
    }
}
