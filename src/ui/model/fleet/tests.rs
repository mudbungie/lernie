//! The fleet pane between frames: the gate on opening it, the three words, the
//! receipt read by its op, and the retirement that goes with the wall.

use crate::reply::{Read, Reply};
use crate::test_support::window::{attempt, diff, fleeting, own, seated};
use crate::ui::Model;

/// **Every one of its seven ops carries a workspace**, so a pane opened with
/// nothing aimed at would be controls with nowhere to fire. The aim is the
/// gate, exactly as it is the tuning pane's.
#[test]
fn the_pane_opens_only_on_an_aimed_wall_and_closes_again() {
    let mut unaimed = Model::default();
    unaimed.begin_fleet();
    assert!(
        unaimed.fleet.is_none(),
        "a pane opened with nowhere to fire"
    );
    let mut model = seated();
    model.begin_fleet();
    assert!(model.fleet.is_some());
    assert!(model.covered(), "the pane covers the conversation");
    model.close_fleet();
    assert!(model.fleet.is_none());
}

/// **The cap opens at one, never at zero.** A cap of zero is a loop that
/// spawns nothing and still reaps, which upstream refuses to spell as a cap —
/// `disband` is that — so it is not a value this box can send. The floor
/// itself is the pane's, and its own suite holds that beat.
#[test]
fn the_cap_opens_at_one() {
    let mut model = seated();
    model.begin_fleet();
    assert_eq!(model.fleet.as_ref().expect("open").cap, 1);
}

/// **The receipt is filed under the OP and never under a family read off the
/// reply** — one `armed` kind spans the fleet loop and the alignment monitor,
/// and only the poster still knows which was sent.
#[test]
fn the_shared_receipt_is_filed_under_the_op_that_earned_it() {
    let mut model = fleeting();
    model.receipt(&own().channel, "disarm", Read::Answer(Reply::Armed(false)));
    let said = model
        .fleet
        .as_ref()
        .and_then(|held| held.said.clone())
        .expect("a receipt");
    assert_eq!(said.op, "disarm");
    assert!(!said.armed);
}

/// **A frame that reached the plain door carries no op**, because no standing
/// read answers this kind — so it is filed under no name rather than under a
/// guess.
#[test]
fn an_unstamped_armed_frame_is_filed_under_no_name() {
    let mut model = fleeting();
    model.absorb(&own().channel, Read::Answer(Reply::Armed(true)));
    assert_eq!(
        model
            .fleet
            .as_ref()
            .and_then(|held| held.said.clone())
            .expect("a receipt")
            .op,
        ""
    );
}

/// The two reads are filed whether or not the pane is open, on the roles'
/// terms: a frame after the close is the last one in flight.
#[test]
fn the_two_reads_are_filed_off_the_pane() {
    let mut model = seated();
    model.absorb(
        &own().channel,
        Read::Answer(Reply::Science(vec![attempt("bl-1", "pending")])),
    );
    model.absorb(
        &own().channel,
        Read::Answer(Reply::Work(vec![diff("bl-1", "unreadable")])),
    );
    assert_eq!(model.attempts.as_deref().expect("answered").len(), 1);
    assert_eq!(model.work.as_deref().expect("answered").len(), 1);
}

/// **The pane and both its answers go with the wall.** Its acts run drones and
/// spend money in ONE workspace, so a pane left standing over a new aim would
/// offer to start a fleet somewhere the operator is no longer looking.
#[test]
fn an_aim_retires_the_pane_and_both_its_answers() {
    let mut model = fleeting();
    model.select("c-2");
    assert!(
        model.fleet.is_some(),
        "a selection retired a pane it is not about"
    );
    let aim = model.aim.clone().expect("the fixture is aimed");
    model.aim_at(&aim.channel, &aim.address);
    assert!(model.fleet.is_none());
    assert!(model.attempts.is_none());
    assert!(model.work.is_none());
}

/// An act goes out as an ACT, which is what makes a lost reply IN DOUBT rather
/// than a thing to re-ask.
#[test]
fn an_act_is_posted_as_one() {
    let mut model = fleeting();
    model.post_fleet(crate::verbs::disband("home".to_owned()));
    let posted = model.outbox.first().expect("a gesture");
    assert!(posted.act);
    assert_eq!(posted.envelope["op"], "disband");
}
