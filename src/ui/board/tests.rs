//! The ball pane: the two emptinesses, every section it paints, the silence a
//! quiet channel gets, and the control that opens it.

use super::{BINDINGS, CLOSE, HEADING, NOT_ANSWERED, NOTHING, OPEN};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{boarded, click, own, painted, seated};
use crate::ui::{Bindings, Columns, Model};

/// **The control hangs off the roster and opens the pane**, which is the only
/// thing that says that button is wired to that door. It is offered from an
/// unaimed seat, because two of its four reads take no subject.
#[test]
fn the_roster_s_control_opens_the_pane() {
    let mut model = Model::default();
    let window = Window::new();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.boarding(), "the control opened nothing");
    let glass = painted(&mut model);
    assert!(glass.contains(HEADING), "{glass}");
    // And the strip stands down while a pane covers the conversation.
    assert!(!glass.contains(OPEN), "{glass}");
}

/// **The close puts it down**, and the roster comes back with it.
#[test]
fn the_close_puts_the_pane_down() {
    let mut model = boarded();
    let window = Window::new();
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.boarding());
}

/// **The two emptinesses are two sentences.** Nobody has answered and nothing
/// is aimed at; and every channel answered and holds nothing — the
/// conversation list's own doctrine, on a pane whose subject is two widths.
#[test]
fn each_emptiness_says_which_one_it_is() {
    let mut unheard = Model {
        lookup: Some(crate::ui::Lookup::Board),
        ..Model::default()
    };
    let glass = painted(&mut unheard);
    assert!(glass.contains(NOT_ANSWERED), "{glass}");

    let mut quiet = Model {
        lookup: Some(crate::ui::Lookup::Board),
        columns: vec![Columns {
            channel: own().channel,
            board: crate::reply::board::Board {
                rows: Vec::new(),
                fleet: Vec::new(),
            },
        }],
        bindings: vec![Bindings {
            channel: own().channel,
            rows: Vec::new(),
        }],
        ..Model::default()
    };
    let glass = painted(&mut quiet);
    assert!(glass.contains(NOTHING), "{glass}");
    assert!(!glass.contains(NOT_ANSWERED), "{glass}");
}

/// **The wall's sentence and the union's are two sentences**, painted
/// together: a seat that has aimed at a wall and heard from no channel is
/// waiting on both, and neither absence speaks for the other.
#[test]
fn an_aimed_seat_waits_on_the_wall_and_on_the_channels_separately() {
    let mut model = Model {
        lookup: Some(crate::ui::Lookup::Board),
        ..seated()
    };
    let glass = painted(&mut model);
    assert!(glass.contains(NOT_ANSWERED), "{glass}");
    assert!(glass.contains(super::wall::NOT_ANSWERED), "{glass}");
    assert!(!glass.contains(NOTHING), "{glass}");
}

/// **Every sentence the pane can say reaches the glass** — the row, its
/// placement, its holder, its gates, its drones, its two figures, the loop's
/// line, the binding table and the aimed wall's own half.
#[test]
fn every_line_the_pane_paints_reaches_the_glass() {
    let mut model = boarded();
    let glass = painted(&mut model);
    for said in [
        "bl-1",
        "claimed · bound",
        "held by alba",
        "under bl-epic",
        "gated by bl-gate",
        "worked by Cobalt",
        "$1.50",
        "under it",
        "4/4 drones",
        "ceiling: over budget",
        BINDINGS,
        "unheld",
        "tracking tasks on balls/tasks",
        "$2.50",
    ] {
        assert!(glass.contains(said), "{said:?} is on no row: {glass}");
    }
}

/// **A channel that answered nothing is silent, not a header over a blank** —
/// the queue's rule, on the pane that shares its shape, and it is asked of
/// both of this pane's unions.
#[test]
fn a_channel_holding_nothing_gets_no_header() {
    let mut model = boarded();
    let quiet = crate::ui::Channel {
        name: "elsewhere".to_owned(),
        ..own().channel
    };
    model.columns.push(Columns {
        channel: quiet.clone(),
        board: crate::reply::board::Board {
            rows: Vec::new(),
            fleet: Vec::new(),
        },
    });
    model.bindings.push(Bindings {
        channel: quiet,
        rows: Vec::new(),
    });
    let glass = painted(&mut model);
    assert!(!glass.contains("elsewhere"), "{glass}");
}

/// **A loop with no rows is still a section**, because a fleet running in a
/// wall whose balls are all closed is exactly the thing an operator opens this
/// pane to see.
#[test]
fn a_channel_with_a_loop_and_no_rows_still_says_so() {
    let mut model = boarded();
    model.columns[0].board.rows.clear();
    let glass = painted(&mut model);
    assert!(glass.contains("4/4 drones"), "{glass}");
}
