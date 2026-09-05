//! The five acts between frames: the name they stamp, what each composes, and
//! the two refusals that are facts rather than policies.

use serde_json::json;

use crate::test_support::window::{boarded, own, panes::board::column, seated};
use crate::ui::{Aim, Channel, Model};

/// The block the fixture's wall opens on its one held ball.
fn amending() -> Model {
    let mut model = boarded();
    let ball = model
        .holding
        .clone()
        .and_then(|rows| rows.first().cloned())
        .expect("the boarded fixture holds one ball");
    model.begin_amending(&ball);
    model
}

/// **The stamp is the wall's name as its ENGINE spells it.** A channel that
/// renames answers the host's name and not this box's leaf, because the
/// rewrite `crate::seat::route` performs is on a `workspace` field none of
/// these five has.
#[test]
fn the_stamp_is_the_engines_own_spelling_of_the_aimed_wall() {
    assert_eq!(boarded().stamp(), Some("home".to_owned()));

    let renaming = Model {
        roster: vec![crate::ui::Chunk {
            channel: Channel {
                name: "leaf".to_owned(),
                named_there: Some("their-name".to_owned()),
                dials: None,
            },
            ..own()
        }],
        aim: Some(Aim {
            channel: "leaf".to_owned(),
            address: "leaf".to_owned(),
        }),
        ..Model::default()
    };
    assert_eq!(renaming.stamp(), Some("their-name".to_owned()));
}

/// **Nothing aimed at is no stamp**, and so no act — and an aim on a channel
/// this box no longer holds is the same answer for the same reason.
#[test]
fn an_unaimed_seat_stamps_nothing_and_opens_no_block() {
    let mut nowhere = Model::default();
    assert_eq!(nowhere.stamp(), None);
    nowhere.begin_filing();
    assert!(nowhere.authoring.is_none());

    let mut gone = Model {
        aim: Some(Aim {
            channel: "vanished".to_owned(),
            address: "home".to_owned(),
        }),
        ..Model::default()
    };
    assert_eq!(gone.stamp(), None);
    gone.begin_amending(&crate::test_support::window::panes::board::bound("bl-1"));
    assert!(gone.authoring.is_none());
}

/// **The block opens on the ball's own project** and closes changing nothing.
#[test]
fn the_block_opens_on_the_balls_project_and_the_way_out_changes_nothing() {
    let mut model = amending();
    assert_eq!(
        model.authoring.as_ref().map(|block| block.project.clone()),
        Some("lernie".to_owned())
    );
    model.close_authoring();
    assert!(model.authoring.is_none());
}

/// **An act is addressed down one channel and never fanned**, which is the
/// whole reason these five needed a ball of their own.
#[test]
fn an_act_carries_the_channel_its_row_came_down() {
    let mut model = amending();
    let down = model.channel().expect("the fixture is aimed");
    model.post_ball(&down, json!({ "op": "release" }));
    let posted = model.outbox.first().expect("one gesture");
    assert!(posted.act);
    assert_eq!(
        posted.channel.as_ref().map(|held| held.name.clone()),
        Some(down.name)
    );
}

/// **Two facts refuse a claim and neither is a policy.** A ball somebody holds
/// is not one to claim, and a ball on one engine cannot be claimed by a wall
/// on another.
#[test]
fn a_claim_needs_an_unheld_row_on_the_aimed_channel() {
    let model = boarded();
    let down = model.channel().expect("the fixture is aimed");
    assert_eq!(
        model.claiming(&down, &column("bl-3", "ready")),
        Some(json!({
            "op": "assign", "project": "lernie", "id": "bl-3", "name": "home"
        }))
    );

    let held = crate::reply::board::BoardRow {
        claimant: Some("somebody".to_owned()),
        ..column("bl-3", "claimed")
    };
    assert_eq!(model.claiming(&down, &held), None);

    let elsewhere = Channel {
        name: "another engine".to_owned(),
        named_there: None,
        dials: None,
    };
    assert_eq!(model.claiming(&elsewhere, &column("bl-3", "ready")), None);

    let unaimed = Model {
        aim: None,
        ..seated()
    };
    assert_eq!(unaimed.claiming(&down, &column("bl-3", "ready")), None);
}
