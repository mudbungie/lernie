//! The policy as a value: the yield, where the two shapes meet, and the two
//! columns.

use super::{CHAT_FLOOR, CONVS, Column, SIDE_FLOOR, Shape, shape, widths};

/// **The width every column is worth together**, which is where the yield and
/// the growth cross. Derived here exactly as the policy derives it, so a
/// tuning of either constant moves this suite's subject with it.
const WORTH: f32 = CONVS + CHAT_FLOOR;

/// **The policy the window had none of** (bl-e5d2): the conversation has a
/// floor and the list pane yields to it until it reaches its own floor —
/// where nothing yields, because a pane showing nothing buys the chat pane a
/// width it still cannot use.
#[test]
fn the_list_pane_yields_to_the_conversation_s_floor_and_then_stops() {
    assert!(
        (widths(WORTH) - CONVS).abs() < 0.01,
        "every column worth exactly what it is worth"
    );
    let list = widths(600.0);
    assert!(
        (list - 180.0).abs() < 0.5,
        "the whole loss is the list's: {list}"
    );
    assert!(
        600.0 - list >= CHAT_FLOOR,
        "the conversation kept its floor"
    );
    let floored = widths(400.0);
    assert!(
        (floored - SIDE_FLOOR).abs() < f32::EPSILON,
        "past its own floor the list pane stops yielding: {floored}"
    );
}

/// **Above the width every column is worth, the yield keeps going the other
/// way** (bl-fef8). The share used to stop at 1.0, so every pixel of a large
/// display landed in the one pane whose content is already prose while the
/// navigation column stayed the width it is worth at a 740-point window
/// forever. It is the same expression rather than a second regime: the share
/// is the smaller of *this column's proportion of the window* and *what is
/// left once the conversation keeps its floor*, and the two clauses cross at
/// exactly the width every column is worth.
#[test]
fn a_window_wider_than_every_column_s_worth_grows_both_together() {
    let list = widths(1400.0);
    assert!(list > CONVS, "{list}");
    assert!(
        1400.0 - list > CHAT_FLOOR,
        "and the conversation grows too, not only the list"
    );
    // Monotone, so no width is served worse than a narrower one.
    let mut before = 0.0;
    for width in [WORTH, 900.0, 1400.0, 2560.0] {
        let list = widths(width);
        assert!(list > before, "{width}: {list} is not past {before}");
        before = list;
    }
}

/// **The two clauses cross at the width every column is worth**, which is what
/// makes the growth a continuation of the yield rather than a hinge: on either
/// side of it the answer is one expression, and at it both clauses give the
/// same number.
#[test]
fn the_yield_and_the_growth_meet_at_the_width_every_column_is_worth() {
    let below = widths(WORTH - 1.0);
    let at = widths(WORTH);
    let above = widths(WORTH + 1.0);
    assert!(below < at && at < above, "{below}, {at}, {above}");
    assert!((at - CONVS).abs() < 0.01, "{at}");
}

/// **Where the two shapes meet, and that the line is the yield itself**
/// (bl-dfda): the broad shape holds exactly as long as the pane's yield still
/// leaves the conversation its floor, and the first width it cannot is the
/// first width one column at a time is the better answer.
///
/// **The turn moved when the middle column went** (DESIGN §4.39): it used to
/// stand where two panes on their own floor could no longer keep
/// `CHAT_FLOOR`, and one pane on its floor can keep it for 140 points longer.
#[test]
fn the_shape_turns_over_at_the_width_the_yield_can_no_longer_keep_the_floor() {
    for wide in [1400.0_f32, 900.0, WORTH, SIDE_FLOOR + CHAT_FLOOR] {
        assert_eq!(
            shape(wide),
            Shape::Broad { list: widths(wide) },
            "at {wide} the conversation still gets its floor"
        );
    }
    for narrow in [SIDE_FLOOR + CHAT_FLOOR - 1.0, 500.0, 400.0, 120.0] {
        assert_eq!(
            shape(narrow),
            Shape::Narrow,
            "at {narrow} the two columns cannot stand together"
        );
    }
}

/// **Every width has a shape, which is what a policy with no floor under it
/// means.** The narrow shape has nothing competing for the window, so there is
/// no width at which this runs out of an answer — and that is what let the
/// snapshot harness drop its *is this width promised* gate rather than keep one
/// that always says yes.
#[test]
fn a_window_of_any_width_at_all_is_promised_a_shape() {
    for width in [0.0_f32, 1.0, 400.0, 4000.0] {
        assert!(
            widths(width) >= SIDE_FLOOR,
            "the yield never goes under the floor: {width}"
        );
        assert!(
            matches!(shape(width), Shape::Broad { .. } | Shape::Narrow),
            "there is an answer at {width}"
        );
    }
}

/// **A column's name is the pane's own heading**, and there is no second
/// vocabulary for the bar to drift into.
#[test]
fn each_column_is_named_by_the_pane_it_shows() {
    assert_eq!(
        Column::all().map(Column::word),
        [crate::ui::roster::HEADING, crate::ui::chat::HEADING]
    );
}

/// **A step sideways saturates at the ends**, for the reason the walk down a
/// list does: a wrap makes one keypress mean *the next one* twice and *back to
/// the start* once, with nothing on the glass to say which it will be.
#[test]
fn a_step_sideways_moves_one_column_and_stops_at_the_ends() {
    assert_eq!(Column::Engines.stepped(1), Column::Conversation);
    assert_eq!(Column::Conversation.stepped(1), Column::Conversation);
    assert_eq!(Column::Conversation.stepped(-1), Column::Engines);
    assert_eq!(Column::Engines.stepped(-1), Column::Engines);
}

/// The window opens on the engines: a seat with nothing aimed at has exactly
/// one thing to do next.
#[test]
fn the_window_opens_on_the_engines_column() {
    assert_eq!(Column::default(), Column::Engines);
}
