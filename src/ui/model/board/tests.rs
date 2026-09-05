//! The ball pane between frames: the door, the two unions it files into, and
//! the wall half that goes with the wall.

use crate::reply::{Read, Reply};
use crate::test_support::window::{boarded, column, figure, own, seated};
use crate::ui::{Channel, Model};

/// One binding row, quiet.
fn binding(id: &str) -> crate::reply::balls::BallRow {
    crate::reply::balls::BallRow {
        ball_id: id.to_owned(),
        project: "p".to_owned(),
        state: "ready".to_owned(),
        title: None,
        claimant: None,
        workspace: None,
    }
}

/// The pane takes no subject: two of its four reads name no workspace, so
/// *what is on the board* is answerable from an unaimed seat.
#[test]
fn the_pane_opens_with_nothing_aimed_at_and_closes_again() {
    let mut model = Model::default();
    assert!(!model.boarding());
    model.begin_board();
    assert!(model.boarding());
    assert!(model.covered(), "the pane covers the conversation");
    model.escape();
    assert!(!model.boarding());
}

/// **It stands in the same field the other channel-wide panes do**, which is
/// what makes *two of them open at once* unrepresentable rather than merely
/// unreachable.
#[test]
fn opening_another_channel_wide_pane_stands_the_board_down() {
    let mut model = Model::default();
    model.begin_board();
    model.begin_trail();
    assert!(!model.boarding());
    assert!(model.trailing());
}

/// **The pane survives an aim and a selection and its WALL HALF does not.**
/// Two of its reads are about every channel, so nothing on the glass is their
/// subject; two are about the wall, and the old wall's answer is not the new
/// wall's.
#[test]
fn an_aim_retires_the_wall_s_answers_and_leaves_the_pane_standing() {
    let mut model = boarded();
    model.select("c-2");
    assert!(
        model.boarding(),
        "a selection retired a pane it is not about"
    );
    assert!(
        model.holding.is_some(),
        "a selection retired the wall's balls"
    );
    let aim = model.aim.clone().expect("the fixture is aimed");
    model.aim_at(&aim.channel, &aim.address);
    assert!(model.boarding(), "an aim retired a pane it is not about");
    assert!(
        model.holding.is_none(),
        "the old wall's balls outlived the aim"
    );
    assert!(
        model.marks.is_none(),
        "the old wall's branch outlived the aim"
    );
}

/// **One channel's board replaces its own section and no other**, which is the
/// roster's rule and the queue's before it.
#[test]
fn a_board_answer_replaces_its_own_channel_and_leaves_the_others_standing() {
    let mut model = seated();
    let other = Channel {
        name: "elsewhere".to_owned(),
        ..own().channel
    };
    let full = crate::reply::board::Board {
        rows: vec![column("bl-1", "ready")],
        fleet: Vec::new(),
    };
    model.absorb(&own().channel, Read::Answer(Reply::Board(full.clone())));
    model.absorb(&other, Read::Answer(Reply::Board(full)));
    assert_eq!(model.columns.len(), 2);
    model.absorb(
        &own().channel,
        Read::Answer(Reply::Board(crate::reply::board::Board {
            rows: Vec::new(),
            fleet: Vec::new(),
        })),
    );
    assert_eq!(
        model.columns.len(),
        2,
        "a section was added rather than replaced"
    );
    assert!(
        model.columns[0].board.rows.is_empty(),
        "the answer did not replace"
    );
    assert_eq!(
        model.columns[1].board.rows.len(),
        1,
        "the other channel was disturbed"
    );
}

/// The binding table files on exactly the same terms.
#[test]
fn a_binding_answer_replaces_its_own_channel_and_leaves_the_others_standing() {
    let mut model = seated();
    let other = Channel {
        name: "elsewhere".to_owned(),
        ..own().channel
    };
    model.absorb(
        &own().channel,
        Read::Answer(Reply::Balls(vec![binding("bl-1")])),
    );
    model.absorb(&other, Read::Answer(Reply::Balls(vec![binding("bl-2")])));
    assert_eq!(model.bindings.len(), 2);
    model.absorb(&own().channel, Read::Answer(Reply::Balls(Vec::new())));
    assert_eq!(
        model.bindings.len(),
        2,
        "a section was added rather than replaced"
    );
    assert!(
        model.bindings[0].rows.is_empty(),
        "the answer did not replace"
    );
    assert_eq!(
        model.bindings[1].rows.len(),
        1,
        "the other channel was disturbed"
    );
}

/// **The wall's two answers are filed whether or not the pane is open**, on
/// the roles' terms: a frame arriving after it closed is the last one in
/// flight rather than a thing to drop.
#[test]
fn the_wall_s_two_answers_are_filed_off_the_pane() {
    let mut model = seated();
    model.absorb(
        &own().channel,
        Read::Answer(Reply::WorkspaceBalls(vec![
            crate::reply::balls::BoundBall {
                id: "bl-1".to_owned(),
                badge: None,
                project: "p".to_owned(),
                owner: "alba".to_owned(),
                state: "bound".to_owned(),
                spend: figure(None, None),
            },
        ])),
    );
    model.absorb(
        &own().channel,
        Read::Answer(Reply::Marks {
            branch: "balls/tasks".to_owned(),
        }),
    );
    assert_eq!(
        model.holding.as_deref().expect("the wall answered").len(),
        1
    );
    assert_eq!(model.marks.as_deref(), Some("balls/tasks"));
}

/// The answers outlive the close, exactly as the trail's rows do.
#[test]
fn closing_the_pane_keeps_what_the_channels_said() {
    let mut model = boarded();
    model.close_lookup();
    assert_eq!(model.columns.len(), 1);
    assert_eq!(model.bindings.len(), 1);
    assert!(model.holding.is_some());
}
