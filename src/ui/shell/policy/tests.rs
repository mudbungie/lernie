//! The policy as a value: the yield, where the two shapes meet, and the three
//! columns.

use super::{CHAT_FLOOR, Column, SIDE_FLOOR, Shape, shape, widths};
use crate::ui::Pane;

/// **The policy the window had none of** (bl-e5d2): the conversation has a
/// floor and the two list panes yield to it, together and in proportion, until
/// they reach their own floor — where nothing yields, because two panes showing
/// nothing buys the chat pane a width it still cannot use.
#[test]
fn the_list_panes_yield_to_the_conversation_s_floor_and_then_stop() {
    assert_eq!(widths(1200.0), (280.0, 320.0), "wide enough for both");
    assert_eq!(
        widths(1020.0),
        (280.0, 320.0),
        "exactly enough is still enough"
    );
    let (roster, convs) = widths(900.0);
    assert!(
        (roster - 224.0).abs() < 0.5 && (convs - 256.0).abs() < 0.5,
        "the loss is shared in proportion: {roster}, {convs}"
    );
    assert!(
        900.0 - roster - convs >= CHAT_FLOOR,
        "the conversation kept its floor"
    );
    assert_eq!(
        widths(400.0),
        (SIDE_FLOOR, SIDE_FLOOR),
        "past their own floor the list panes stop yielding"
    );
}

/// **Where the two shapes meet, and that the line is the yield itself**
/// (bl-dfda): the broad shape holds exactly as long as the panes' yield still
/// leaves the conversation its floor, and the first width it cannot is the
/// first width one column at a time is the better answer.
#[test]
fn the_shape_turns_over_at_the_width_the_yield_can_no_longer_keep_the_floor() {
    for wide in [1400.0_f32, 900.0, 720.0] {
        let (roster, convs) = widths(wide);
        assert_eq!(
            shape(wide),
            Shape::Broad { roster, convs },
            "at {wide} the conversation still gets its floor"
        );
    }
    for narrow in [719.0_f32, 700.0, 400.0, 120.0] {
        assert_eq!(
            shape(narrow),
            Shape::Narrow,
            "at {narrow} the three columns cannot stand together"
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
        let (roster, convs) = widths(width);
        assert!(
            roster >= SIDE_FLOOR && convs >= SIDE_FLOOR,
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
        [
            crate::ui::roster::HEADING,
            crate::ui::convs::HEADING,
            crate::ui::chat::HEADING,
        ]
    );
}

/// **The arrows belong to the column on the glass**, and the conversation
/// column answers the conversation list: the row a walk lands on is exactly
/// what the chat pane is showing.
#[test]
fn each_column_says_which_list_the_arrows_walk() {
    assert_eq!(Column::Channels.arrows(), Pane::Roster);
    assert_eq!(Column::Conversations.arrows(), Pane::Conversations);
    assert_eq!(Column::Conversation.arrows(), Pane::Conversations);
}

/// **A step sideways saturates at the ends**, for the reason the walk down a
/// list does: a wrap makes one keypress mean *the next one* twice and *back to
/// the start* once, with nothing on the glass to say which it will be.
#[test]
fn a_step_sideways_moves_one_column_and_stops_at_the_ends() {
    assert_eq!(Column::Channels.stepped(1), Column::Conversations);
    assert_eq!(Column::Conversations.stepped(1), Column::Conversation);
    assert_eq!(Column::Conversation.stepped(1), Column::Conversation);
    assert_eq!(Column::Conversation.stepped(-1), Column::Conversations);
    assert_eq!(Column::Conversations.stepped(-1), Column::Channels);
    assert_eq!(Column::Channels.stepped(-1), Column::Channels);
}

/// The window opens on the roster: a seat with nothing aimed at has exactly one
/// thing to do next.
#[test]
fn the_window_opens_on_the_channels_column() {
    assert_eq!(Column::default(), Column::Channels);
}
