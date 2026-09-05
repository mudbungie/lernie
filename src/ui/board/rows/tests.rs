//! The words a board row wears: every line, and the absence that is silence.

use super::{binding, cost, gated, headline, held, placed, running, worked};
use crate::test_support::window::{column, figure};

/// One board row with everything on it.
fn full() -> crate::reply::board::BoardRow {
    crate::reply::board::BoardRow {
        state: "bound".to_owned(),
        workspace: Some("home".to_owned()),
        claimant: Some("alba".to_owned()),
        parent: Some("bl-epic".to_owned()),
        gates: vec![crate::reply::board::Gate {
            id: "bl-gate".to_owned(),
            title: "the gate".to_owned(),
            mints: "close".to_owned(),
        }],
        drones: vec![crate::reply::board::Drone {
            root_id: "c-1".to_owned(),
            name: "Cobalt".to_owned(),
        }],
        ..column("bl-1", "claimed")
    }
}

/// **The headline is which ball and what it is called** — the one line every
/// row gets, whatever else is true of it.
#[test]
fn the_headline_names_the_ball_and_says_what_it_is_for() {
    assert_eq!(headline(&full()), "bl-1  what bl-1 is for");
}

/// **The column and the state are two facts and both are said**, each
/// verbatim: a column this build has never seen paints as itself.
#[test]
fn the_placement_says_the_column_the_state_the_rank_and_the_project() {
    assert_eq!(placed(&full()), "claimed · bound  priority 2  in lernie");
    let strange = crate::reply::board::BoardRow {
        column: "quarantined".to_owned(),
        ..column("bl-9", "x")
    };
    assert!(placed(&strange).contains("quarantined"));
}

/// Whose it is, where it runs and what it is part of, read as one line — and
/// nothing at all for a ball nobody has claimed.
#[test]
fn the_holder_line_is_absent_on_a_ball_nobody_holds() {
    assert_eq!(
        held(&full()).expect("a held row says so"),
        "held by alba  on home  under bl-epic"
    );
    assert!(held(&column("bl-2", "ready")).is_none());
}

/// The gates are the balls whose close would open this one, and a row with
/// none says nothing rather than saying *no gates*.
#[test]
fn the_gates_are_named_and_a_row_with_none_is_silent() {
    assert_eq!(
        gated(&full()).expect("a gated row says so"),
        "gated by bl-gate the gate (close)"
    );
    assert!(gated(&column("bl-2", "ready")).is_none());
}

/// The conversations working it, on the same terms.
#[test]
fn the_drones_are_named_and_a_row_with_none_is_silent() {
    assert_eq!(
        worked(&full()).expect("a worked row says so"),
        "worked by Cobalt (c-1)"
    );
    assert!(worked(&column("bl-2", "ready")).is_none());
}

/// **The money is upstream's own string**, and a figure no rate priced says
/// its tokens and its attribution and no money at all.
#[test]
fn a_figure_says_its_money_where_there_is_one_and_never_computes_it() {
    assert_eq!(
        cost(&figure(Some("$1.50"), Some("over 2 conversations"))),
        "$1.50  99 tokens  over 2 conversations"
    );
    assert_eq!(cost(&figure(None, None)), "99 tokens  conversations");
}

/// **The loop's line is the engine's**, with the two facts it does not carry
/// hung off it — and neither is said where it is not true.
#[test]
fn a_loop_says_the_engine_s_own_line_plus_the_room_and_the_ceiling() {
    let full = crate::reply::board::Fleet {
        workspace: "home".to_owned(),
        project: "lernie".to_owned(),
        cap: 4,
        count: 4,
        room: false,
        ceiling: Some("over budget".to_owned()),
        label: "4/4 drones".to_owned(),
    };
    assert_eq!(
        running(&full),
        "running lernie in home: 4/4 drones  full  ceiling: over budget"
    );
    let easy = crate::reply::board::Fleet {
        room: true,
        ceiling: None,
        ..full
    };
    assert_eq!(running(&easy), "running lernie in home: 4/4 drones");
}

/// **A binding says its three absences as words**, because the pane is a list
/// of one-line facts and a blank in one would read as the row above it.
#[test]
fn a_binding_says_unheld_where_nobody_holds_it() {
    let row = crate::reply::balls::BallRow {
        ball_id: "bl-1".to_owned(),
        project: "p".to_owned(),
        state: "bound".to_owned(),
        title: Some("t".to_owned()),
        claimant: Some("alba".to_owned()),
        workspace: Some("home".to_owned()),
    };
    assert_eq!(binding(&row), "bl-1  t  [bound]  in p  alba on home");
    let unwalled = crate::reply::balls::BallRow {
        workspace: None,
        title: None,
        ..row.clone()
    };
    assert_eq!(binding(&unwalled), "bl-1  [bound]  in p  held by alba");
    let unheld = crate::reply::balls::BallRow {
        claimant: None,
        workspace: None,
        ..row
    };
    assert_eq!(binding(&unheld), "bl-1  t  [bound]  in p  unheld");
}
