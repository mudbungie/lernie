//! The union across channels: every channel asked for its own roster, each
//! answer stamped with the channel it came down, and a channel that will not
//! answer costing only itself.

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
    let notice = settled.notice.expect("the hollow entry said so");
    assert!(
        notice.line().contains("could not reach"),
        "{}",
        notice.line()
    );
    assert!(notice.line().contains("empty entry"), "{}", notice.line());
}
