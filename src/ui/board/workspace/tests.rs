//! The aimed workspace's section: its four emptinesses, and the two lines a ball
//! this workspace holds wears.

use super::{NOT_ANSWERED, NOTHING, UNAIMED, bound, tracking};
use crate::test_support::window::{boarded, figure, painted, seated};
use crate::ui::Model;

/// **No workspace aimed at is its own sentence**, and it is an instruction rather
/// than a fact: the remedy is one click on the roster behind the pane.
#[test]
fn an_unaimed_seat_is_told_what_to_aim_at() {
    let mut model = Model {
        lookup: Some(crate::ui::Lookup::Board),
        ..Model::default()
    };
    let glass = painted(&mut model);
    assert!(glass.contains(UNAIMED), "{glass}");
}

/// **A workspace nobody has asked about is a wait, and a workspace that answered zero is
/// a fact** — two claims, two sentences.
#[test]
fn an_unanswered_workspace_waits_and_an_empty_one_says_it_holds_none() {
    let mut waiting = Model {
        lookup: Some(crate::ui::Lookup::Board),
        ..seated()
    };
    let glass = painted(&mut waiting);
    assert!(glass.contains(NOT_ANSWERED), "{glass}");

    let mut empty = Model {
        holding: Some(Vec::new()),
        ..waiting
    };
    let glass = painted(&mut empty);
    assert!(glass.contains(NOTHING), "{glass}");
    assert!(!glass.contains(NOT_ANSWERED), "{glass}");
}

/// **The branch is a line on its own**, because it is how an operator tells a
/// workspace tracking the project's shared board from one tracking its own space.
#[test]
fn the_tracking_branch_is_said_where_the_workspace_answered_one() {
    assert_eq!(tracking("marks/alba"), "tracking tasks on marks/alba");
    let mut model = boarded();
    let glass = painted(&mut model);
    assert!(glass.contains("tracking tasks on balls/tasks"), "{glass}");
}

/// **A ball this workspace holds says its badge where the server wrote one**, and
/// nothing where the state needs none.
#[test]
fn a_bound_ball_wears_its_badge_only_where_there_is_one() {
    let badged = crate::reply::balls::BoundBall {
        id: "bl-1".to_owned(),
        badge: Some("delivered".to_owned()),
        project: "lernie".to_owned(),
        owner: "alba".to_owned(),
        state: "bound".to_owned(),
        spend: figure(None, None),
    };
    assert_eq!(
        bound(&badged),
        "bl-1  [bound]  delivered  in lernie as alba"
    );
    let bare = crate::reply::balls::BoundBall {
        badge: None,
        ..badged
    };
    assert_eq!(bound(&bare), "bl-1  [bound]  in lernie as alba");
}
