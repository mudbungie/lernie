//! A selection is answered at the next leg, ahead of the fan, and the fan
//! still visits every channel (bl-1f22).

use super::super::{caught_up, tick};
use super::asking;
use crate::test_support::Scratch;
use crate::test_support::engine::Engine;
use crate::test_support::wire::{entry, flat, wired};
use crate::ui::{Aim, Model};
use serde_json::{Value, json};

/// The ops one engine was asked, in order.
fn ops(engine: &Engine) -> Vec<String> {
    engine
        .heard()
        .iter()
        .filter_map(|said| said.get("op").and_then(Value::as_str))
        .map(str::to_owned)
        .collect()
}

/// A model holding every channel under `scratch`, aimed at this box's own.
fn aimed_home(scratch: &Scratch) -> Model {
    Model {
        roster: crate::seat::channels(scratch.path()),
        aim: Some(Aim {
            channel: crate::seat::OWN.to_owned(),
            address: "home".to_owned(),
        }),
        ..Model::default()
    }
}

/// **After a select, the next request is the selected conversation's
/// transcript** — not the roster fan it used to wait behind — **and the fan
/// still visits every channel** on that same pass.
#[test]
fn a_selection_is_read_before_the_fan_and_the_fan_still_visits_every_channel() {
    let scratch = Scratch::new();
    let roster = || vec![json!({"ok": true, "kind": "workspaces", "rows": []})];
    let own = wired(
        &scratch,
        &flat(),
        vec![
            roster(),
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
            vec![json!({"ok": true, "kind": "transcript", "rows": []})],
            vec![json!({"ok": true, "kind": "refused", "why": "no such agent"})],
            roster(),
            vec![json!({"ok": true, "kind": "conversations", "rows": []})],
        ],
    );
    let far = wired(&scratch, &entry("far"), vec![roster(), roster()]);
    let mut model = aimed_home(&scratch);
    let link = asking(&model);
    tick(&link, scratch.path());
    model.select("20260830T051200Z-a1b2");
    link.settle(&mut model);
    tick(&link, scratch.path());
    assert_eq!(
        ops(&own),
        vec![
            "workspaces",
            "conversations",
            "transcript",
            "agent",
            "workspaces",
            "conversations",
        ]
    );
    assert_eq!(ops(&far), vec!["workspaces", "workspaces"]);
}

/// **A leg boundary asks only about a selection the pass has not asked about
/// yet.** One it already asked is left for the next pass; one that moved is
/// asked now, down the channel it names.
#[test]
fn a_leg_boundary_asks_only_about_a_selection_that_moved() {
    let scratch = Scratch::new();
    let own = wired(
        &scratch,
        &flat(),
        vec![
            vec![json!({"ok": true, "kind": "transcript", "rows": []})],
            vec![json!({"ok": true, "kind": "refused", "why": "no such agent"})],
        ],
    );
    let mut model = aimed_home(&scratch);
    model.select("20260830T051200Z-a1b2");
    let link = asking(&model);
    let asked = caught_up(&link, scratch.path(), None);
    assert_eq!(ops(&own), vec!["transcript", "agent"]);
    assert_eq!(
        caught_up(&link, scratch.path(), asked.clone()),
        asked,
        "the same selection again"
    );
    assert_eq!(ops(&own).len(), 2, "is not asked twice in one pass");
    model.aim = None;
    model.conversation = None;
    link.settle(&mut model);
    assert_eq!(caught_up(&link, scratch.path(), asked), None);
    assert_eq!(ops(&own).len(), 2, "and no selection asks nothing");
}
