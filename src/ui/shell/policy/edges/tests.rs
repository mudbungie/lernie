//! What a drag is worth: that no drag is the yield exactly, that a drag is the
//! operator's, that the clamp narrows without forgetting, and that the
//! composer's rows are bounded at both ends.

use super::super::{CHAT_FLOOR, SIDE_FLOOR, widths};
use super::{TAIL_FLOOR, rows, shown, span};

/// One line of body type, as the composer's own paint measures it, and the
/// space the panel holds besides the field's lines.
const LINE: f32 = 16.0;
const CHROME: f32 = 40.0;

/// **With nothing dragged the policy is the whole of the answer**, at every
/// width — which is what makes the drag an addition rather than a second
/// regime: bl-fef8's yield is untouched where nobody has touched an edge.
#[test]
fn a_seat_that_has_dragged_nothing_is_shown_exactly_what_the_yield_gives_it() {
    for window in [400.0, 700.0, 900.0, 1020.0, 1440.0, 2560.0_f32] {
        assert_eq!(
            shown(window, (None, None)),
            widths(window),
            "at {window} points"
        );
    }
}

/// **A dragged width is the operator's and the policy stands aside** — the
/// half of bl-fef8 §4.39 reverses. The other pane keeps the policy's answer,
/// because one edge moved is one fact.
#[test]
fn an_edge_the_operator_dragged_is_the_width_the_pane_is_shown_at() {
    let (_, convs) = widths(1440.0);
    assert_eq!(shown(1440.0, (Some(200.0), None)), (200.0, convs));
}

/// **A window narrower than the drag CLAMPS it and never overwrites it.** The
/// shown width is what the window can afford; what the seat holds is what the
/// operator set, so the pane comes back to it when the window does.
#[test]
fn a_window_too_narrow_for_the_drag_narrows_the_pane_and_not_the_fact() {
    let held = (Some(600.0), Some(SIDE_FLOOR));
    let (roster, _) = shown(900.0, held);
    assert!(roster < 600.0, "the window cannot afford 600: {roster}");
    let whole = shown(1440.0, held).0;
    assert!(
        (whole - 600.0).abs() < f32::EPSILON,
        "and the window that can shows it whole: {whole}"
    );
}

/// **The conversation keeps its floor against both drags together**, which is
/// the one thing the policy still owns: an edge dragged to the frame would
/// leave the pane the window exists for with nothing.
#[test]
fn two_edges_dragged_to_the_frame_still_leave_the_conversation_its_floor() {
    let (roster, convs) = shown(1440.0, (Some(2000.0), Some(2000.0)));
    assert!(
        1440.0 - roster - convs >= CHAT_FLOOR,
        "the conversation is under its floor: {roster} + {convs} of 1440"
    );
}

/// **And under the floor nothing yields**, exactly as the yield itself stops:
/// two panes showing nothing buys the chat pane a width it still cannot use,
/// so [`SIDE_FLOOR`] is the one thing the cap gives way to.
#[test]
fn a_window_with_no_room_at_all_holds_both_panes_at_their_own_floor() {
    assert_eq!(span(200.0, 400.0), (SIDE_FLOOR, SIDE_FLOOR));
    assert_eq!(
        shown(200.0, (Some(50.0), Some(50.0))),
        (SIDE_FLOOR, SIDE_FLOOR)
    );
}

/// **A cap is what is left once the conversation and the other pane are
/// paid**, and it is stated once so a test reads it back rather than looking
/// at a window.
#[test]
fn the_cap_on_one_pane_is_what_the_other_pane_and_the_conversation_leave() {
    assert_eq!(
        span(1440.0, 320.0),
        (SIDE_FLOOR, 1440.0 - CHAT_FLOOR - 320.0)
    );
}

/// **The composer's rows are the pointer's, rounded to the nearest line.**
/// Half a line of grace, so the edge lands on the row an operator aimed at
/// rather than the one below it.
#[test]
fn the_rows_are_the_nearest_whole_line_to_where_the_edge_was_dropped() {
    let window = 900.0;
    assert_eq!(rows(CHROME + 3.0 * LINE, CHROME, LINE, window), 3);
    assert_eq!(rows(CHROME + 3.4 * LINE, CHROME, LINE, window), 3);
    assert_eq!(rows(CHROME + 3.6 * LINE, CHROME, LINE, window), 4);
}

/// **One row is the floor**, whatever the edge was dragged to: a field of no
/// rows is a composer with no box, which is a control an operator cannot get
/// back by dragging.
#[test]
fn an_edge_dragged_to_the_foot_of_the_window_still_leaves_one_row() {
    for asked in [CHROME, 0.0, -400.0_f32] {
        assert_eq!(rows(asked, CHROME, LINE, 900.0), 1, "asked {asked}");
    }
}

/// **And the tail keeps a floor**, which is [`CHAT_FLOOR`] one axis over: a
/// composer dragged over the whole window is written down, so the next run
/// would open on a seat whose transcript is gone.
#[test]
fn an_edge_dragged_over_the_whole_window_leaves_the_tail_its_floor() {
    let window = 900.0;
    let most = rows(window * 2.0, CHROME, LINE, window);
    assert!(
        CHROME + f32::from(most) * LINE <= window - TAIL_FLOOR,
        "{most} rows is past the tail's floor"
    );
    assert!(most > 1, "and there is room for more than one: {most}");
}

/// **A window with no room for the tail's floor still answers one row**, which
/// is the rule that makes this total: there is no width or height this has
/// nothing to say for.
#[test]
fn a_window_smaller_than_its_own_floors_still_answers_one_row() {
    assert_eq!(rows(400.0, CHROME, LINE, TAIL_FLOOR), 1);
}
