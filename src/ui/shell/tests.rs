//! The whole window in one frame: that every pane is on it, that a notice
//! stands where content would have been, and that it can be put down.

use super::{DISMISS, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, painted, seated};
use crate::ui::{Model, Notice};

/// **One frame paints every pane.** The smoke test the whole ball is about: a
/// window that opens and shows the roster, the list, the conversation and the
/// composer, all from a snapshot and with nothing dialled.
#[test]
fn one_frame_paints_the_roster_the_list_the_conversation_and_the_composer() {
    let mut model = seated();
    let shown = painted(&mut model);
    for expected in [
        "channels",
        "(this box's own engine)",
        "home  (named)  2 conversations",
        "conversations",
        "port the paint probe  [quiescent]  42s",
        "conversation",
        "op",
        "port it",
        crate::ui::composer::SEND,
    ] {
        assert!(
            shown.contains(expected),
            "{expected:?} is not on the glass:\n{shown}"
        );
    }
}

/// An empty window is not a blank one: every pane says what it is waiting for.
#[test]
fn an_empty_window_says_what_each_pane_is_waiting_for() {
    let mut model = Model::default();
    let shown = painted(&mut model);
    for expected in [
        crate::ui::roster::NO_CHANNEL,
        crate::ui::convs::NO_WALL,
        crate::ui::chat::NO_CONVERSATION,
        crate::ui::composer::NOWHERE,
    ] {
        assert!(
            shown.contains(expected),
            "{expected:?} is not on the glass:\n{shown}"
        );
    }
}

/// **A refusal and an unreadable answer are both visible, and they read
/// differently** — one is the engine's sentence, the other is a statement about
/// this seat, and only the second is fixed by an upgrade. Neither is a silent
/// drop, which is the reply vocabulary's own policy on the glass.
#[test]
fn a_notice_stands_where_the_content_would_have_been_and_says_whose_it_is() {
    for (notice, expected) in [
        (
            Notice::Refused("unknown workspace \"hoem\"".to_owned()),
            "the engine refused: unknown workspace \"hoem\"",
        ),
        (
            Notice::Unreadable("cannot paint a \"board\" answer".to_owned()),
            "this seat could not read the answer: cannot paint a \"board\" answer",
        ),
    ] {
        let mut model = Model {
            notice: Some(notice),
            ..seated()
        };
        let shown = painted(&mut model);
        assert!(shown.contains(expected), "{expected:?}:\n{shown}");
        assert!(
            shown.contains("home  (named)  2 conversations"),
            "a refusal about one pane does not stop the others:\n{shown}"
        );
    }
}

/// **It is a bar, not a modal**, and it can be put down: an operator who has
/// read a refusal should not have to wait for the next answer to clear it.
#[test]
fn a_notice_can_be_put_down() {
    let mut model = Model {
        notice: Some(Notice::Refused("no".to_owned())),
        ..seated()
    };
    let window = Window::new();
    click(&window, DISMISS, |ctx| render(ctx, &mut model));
    assert_eq!(model.notice, None);
}
