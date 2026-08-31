//! The model: the address a channel resolves, the door a reply comes in
//! through, and the notice that stands where content would have been.

use super::{Aim, Channel, Chunk, Model, Notice};
use crate::reply::roster::Workspaces;
use crate::reply::{Outcome, Read, Reply, read};
use crate::test_support::window::{own, seated, wall};
use serde_json::json;

/// **The three addressing cases, and the third is a real one.** This box's own
/// engine rewrites nothing; an entry resolves by its leaf and by nothing else;
/// and an entry's engine may answer a row the entry does not name, which no
/// envelope this seat can write will reach.
#[test]
fn a_channel_says_what_a_gesture_must_carry_or_that_it_cannot_say() {
    let flat = Channel {
        name: "(this box's own engine)".to_owned(),
        named_there: None,
        dials: None,
    };
    assert_eq!(flat.address(&wall("home")), Some("home".to_owned()));
    let entry = Channel {
        name: "home".to_owned(),
        named_there: Some("personal".to_owned()),
        dials: None,
    };
    assert_eq!(entry.address(&wall("personal")), Some("home".to_owned()));
    assert_eq!(entry.address(&wall("somebody-elses")), None);
}

/// **Nothing that arrives is dropped.** An answer is filed; a refusal and an
/// unreadable frame become the notice the shell paints where that content would
/// have been, and they read differently because only one of them is fixed by an
/// upgrade.
#[test]
fn every_frame_becomes_content_or_a_visible_notice() {
    let mut model = Model::default();
    let flat = own().channel;
    model.absorb(
        &flat,
        read(&json!({"ok": false, "error": "unknown workspace"})),
    );
    assert_eq!(
        model.notice,
        Some(Notice::Refused("unknown workspace".to_owned()))
    );
    model.absorb(
        &flat,
        read(&json!({"ok": true, "kind": "board", "rows": []})),
    );
    let Some(Notice::Unreadable(why)) = &model.notice else {
        panic!("an unpainted kind is this seat's own sentence");
    };
    assert!(why.contains("\"board\""), "{why}");
    assert!(
        model
            .notice
            .as_ref()
            .expect("a notice")
            .line()
            .contains("this seat")
    );
}

/// A roster answer replaces **its own channel's** rows and leaves every other
/// channel standing — a box serving three engines does not lose the two that
/// are fine.
#[test]
fn a_roster_answer_replaces_one_channel_and_leaves_the_others() {
    let mut model = Model {
        roster: vec![own()],
        ..Model::default()
    };
    let other = Channel {
        name: "elsewhere".to_owned(),
        named_there: Some("elsewhere".to_owned()),
        dials: None,
    };
    model.absorb(
        &other,
        Read::Answer(Reply::Workspaces(Workspaces {
            rows: vec![wall("theirs")],
            stale: Some("4m behind".to_owned()),
            growth: None,
        })),
    );
    assert_eq!(model.roster.len(), 2);
    assert_eq!(model.roster[0], own(), "the first channel is untouched");
    assert_eq!(model.roster[1].stale.as_deref(), Some("4m behind"));
    // A second answer down the same channel replaces rather than piles up.
    model.absorb(
        &other,
        Read::Answer(Reply::Workspaces(Workspaces::default())),
    );
    assert_eq!(model.roster.len(), 2);
    assert!(model.roster[1].walls.is_empty());
    assert_eq!(model.roster[1].stale, None);
}

/// The four content kinds land in the four panes.
#[test]
fn each_kind_lands_in_the_pane_that_paints_it() {
    let mut model = Model::default();
    let flat = own().channel;
    model.absorb(
        &flat,
        read(&json!({"ok": true, "kind": "conversations", "rows": [
            {"root_id": "a", "display": "one", "state": "live", "uncertain": false, "preview": "",
             "age_secs": 0, "attention": 0, "members": 1, "depth": 0, "tone": "live"}]})),
    );
    assert_eq!(model.convs.len(), 1);
    model.absorb(
        &flat,
        read(&json!({"ok": true, "kind": "transcript", "rows": [
            {"name": "001-op.md", "raw": "hi", "kind": "delivered",
             "sender": "op", "body": "hi"}]})),
    );
    assert_eq!(model.transcript.entries.len(), 1);
    model.absorb(
        &flat,
        read(&json!({"ok": true, "kind": "follow", "stream": {"text": "so far"}})),
    );
    assert_eq!(
        model.live.as_ref().and_then(|s| s.text.clone()),
        Some("so far".to_owned())
    );
}

/// **The two receipts, and a captured run that failed is told in the child's
/// own words.** A receipt with nothing to say clears the last notice; one that
/// says the act did not land replaces it.
#[test]
fn a_receipt_either_clears_the_notice_or_carries_the_child_s_own_refusal() {
    let mut model = Model {
        notice: Some(Notice::Refused("stale".to_owned())),
        ..Model::default()
    };
    let flat = own().channel;
    model.absorb(&flat, Read::Answer(Reply::Nudged));
    assert_eq!(model.notice, None);
    model.absorb(
        &flat,
        Read::Answer(Reply::Outcome(Outcome {
            exit: 1,
            stdout: String::new(),
            stderr: "the gate said no".to_owned(),
        })),
    );
    assert_eq!(
        model.notice,
        Some(Notice::Refused("the gate said no".to_owned()))
    );
    model.absorb(
        &flat,
        Read::Answer(Reply::Outcome(Outcome {
            exit: 0,
            stdout: "deposited".to_owned(),
            stderr: String::new(),
        })),
    );
    assert_eq!(model.notice, None);
}

/// A channel with nothing behind it yet is what the roster holds before any
/// answer has come down it — which is what a box that has never been asked
/// shows, and it is not the same as a channel that answered nothing.
#[test]
fn a_channel_can_be_held_before_it_has_answered() {
    let held = Chunk::of(own().channel);
    assert!(held.walls.is_empty());
    assert_eq!(held.stale, None);
}

/// The aim is a pair, because two channels may both hold a wall called `home`
/// and a seat that matched on the name alone would highlight the wrong row.
#[test]
fn the_aim_is_a_channel_and_an_address_together() {
    let model = Model {
        aim: Some(Aim {
            channel: "one".to_owned(),
            address: "home".to_owned(),
        }),
        ..seated()
    };
    assert!(model.aimed_at("one", Some(&"home".to_owned())));
    assert!(!model.aimed_at("two", Some(&"home".to_owned())));
    assert!(!model.aimed_at("one", Some(&"other".to_owned())));
    assert!(
        !model.aimed_at("one", None),
        "an unreachable row is never aimed at"
    );
    assert!(!Model::default().aimed_at("one", Some(&"home".to_owned())));
}
