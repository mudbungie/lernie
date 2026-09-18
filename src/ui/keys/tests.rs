//! **Every act, from the keyboard, driven into a real window.**
//!
//! Each beat presses real key events into a persistent context and reads back
//! the glyphs on the glass — the same discipline the pointer beats hold, and
//! for the same reason: a galley reports the string that went in, so an
//! assertion made against the input is blind to what the toolkit did with it.

/// The focused thing is visibly the focused thing, and Tab agrees with the
/// arrows.
mod focus;
/// **The one cursor track**, walked by the arrows: where a step lands and what
/// landing there means. Split from this file at the 300-line cap on the seam
/// the module itself has (DESIGN §4.39) — the walk is one subject, and what
/// stays here is everything that is NOT a walk: the keys that stand it down,
/// the binding beside it, and the cursor's own arithmetic.
mod walk;

use super::moved;
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{conv, own, seated, wall};
use crate::ui::{Channel, Chunk, Model, Notice, roster};

/// A window on two channels holding three addressable walls and one that no
/// envelope can reach, with two conversations under the aimed one.
pub(super) fn stocked() -> Model {
    Model {
        roster: vec![
            own(),
            Chunk {
                channel: Channel {
                    name: "elsewhere".to_owned(),
                    named_there: Some("theirs".to_owned()),
                    dials: None,
                },
                walls: vec![wall("theirs"), wall("not-ours")],
                ..Chunk::default()
            },
        ],
        convs: vec![conv("a", "one"), conv("b", "two")],
        ..seated()
    }
}

/// Run `keys` frame by frame over a real window and hand back what it painted.
pub(super) fn typed(model: &mut Model, keys: &[egui::Key]) -> String {
    let window = Window::new();
    let mut painted = String::new();
    for key in keys {
        painted = crate::paint_probe::text_of(&window.frame(vec![press(*key)], |ctx| {
            crate::ui::render(ctx, model);
        }));
    }
    painted
}

/// **Escape puts a notice down**, which is the × button's own act reached by
/// the key an operator already presses to dismiss things.
#[test]
fn escape_puts_the_notice_down() {
    let mut model = Model {
        notice: Some(Notice::Refused("unknown workspace".to_owned())),
        ..stocked()
    };
    let painted = typed(&mut model, &[egui::Key::Escape]);
    assert_eq!(model.notice, None);
    assert!(
        !painted.contains("the engine refused"),
        "the bar is off the glass:\n{painted}"
    );
}

/// **A box that is taking text takes every key.** While the composer holds the
/// keyboard an arrow is a cursor move inside the draft, so nothing here runs —
/// which is what lets Escape mean *leave the box* there and *dismiss* here.
#[test]
fn nothing_is_bound_while_a_box_is_taking_text() {
    // **The bound is generous rather than exact.** What is asserted is that
    // the box CAN take the keyboard, and how many stops away it is is a fact
    // about how many controls the window happens to offer ahead of it — a
    // number every pane that lands moves, and one no assertion here is about.
    const STOPS: usize = 64;
    let mut model = stocked();
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    // Tab until the composer's box has the keyboard, then press the keys that
    // would otherwise walk a list and dismiss a notice.
    window.frame(Vec::new(), &mut body);
    let mut wanted = false;
    for _ in 0..STOPS {
        window.frame(vec![press(egui::Key::Tab)], &mut body);
        wanted = window.focused() == Some(egui::Id::new(super::BOX_ID));
        if wanted {
            break;
        }
    }
    assert!(wanted, "the composer's box never took the keyboard");
    window.frame(vec![press(egui::Key::ArrowDown)], &mut body);
    window.frame(Vec::new(), &mut body);
    assert_eq!(
        model.conversation.as_deref(),
        seated().conversation.as_deref(),
        "the arrow went into the draft, not into a list"
    );
}

/// The cursor's arithmetic, at the ends and on nothing at all.
#[test]
fn a_cursor_saturates_and_an_empty_list_has_nowhere_to_go() {
    assert_eq!(moved(0, None, 1), None);
    assert_eq!(moved(3, None, -1), Some(0), "an unentered list opens at 0");
    assert_eq!(moved(3, Some(0), -1), Some(0));
    assert_eq!(moved(3, Some(2), 1), Some(2));
    assert_eq!(moved(3, Some(1), 1), Some(2));
}

/// A box that has been asked nothing has no row to walk, which is the walk's
/// own empty case rather than a state anybody has to handle.
#[test]
fn a_fresh_window_has_nothing_to_walk() {
    assert_eq!(
        roster::track(&Model::default()),
        Vec::new(),
        "a box that has been asked nothing offers no row to walk"
    );
}

/// **Every box that takes text stands the arrows down, not only the draft**
/// (bl-dbc9).
///
/// It was one id, and one was enough while the only way into the other two was
/// Tab. A conversation row's menu now LANDS the cursor in the reason box and in
/// the arming box (`crate::ui::model::fill`), and an arrow taken from inside
/// either would have walked the conversation list under a half-typed reason —
/// flagging, or arming a deletion on, the row it landed on.
#[test]
fn the_two_parameter_boxes_stand_the_arrows_down_as_the_draft_does() {
    for (fill, id) in [
        (crate::ui::Fill::Reason, super::REASON_ID),
        (crate::ui::Fill::Arming, super::ARM_ID),
    ] {
        let mut model = stocked();
        model.fill_in("a", fill);
        let window = Window::new();
        let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
        window.frame(Vec::new(), &mut body);
        assert_eq!(
            window.focused(),
            Some(egui::Id::new(id)),
            "the box took the keyboard"
        );
        window.frame(vec![press(egui::Key::ArrowDown)], &mut body);
        window.frame(Vec::new(), &mut body);
        assert_eq!(
            model.conversation.as_deref(),
            Some("a"),
            "the arrow went into the box, not into the list"
        );
    }
}
