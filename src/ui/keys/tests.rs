//! **Every act, from the keyboard, driven into a real window.**
//!
//! Each beat presses real key events into a persistent context and reads back
//! the glyphs on the glass — the same discipline the pointer beats hold, and
//! for the same reason: a galley reports the string that went in, so an
//! assertion made against the input is blind to what the toolkit did with it.

use super::{HERE, Pane, moved};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{conv, own, seated, wall};
use crate::ui::{Channel, Chunk, Model, Notice, convs, roster};

/// A window on two channels holding three addressable walls and one that no
/// envelope can reach, with two conversations under the aimed one.
fn stocked() -> Model {
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
fn typed(model: &mut Model, keys: &[egui::Key]) -> String {
    let window = Window::new();
    let mut painted = String::new();
    for key in keys {
        painted = crate::paint_probe::text_of(&window.frame(vec![press(*key)], |ctx| {
            crate::ui::render(ctx, model);
        }));
    }
    painted
}

/// **The window opens on the roster and says so.** A focus that cannot be seen
/// is a focus nobody can use, so the pane that owns the arrows wears the mark.
#[test]
fn the_pane_the_arrows_belong_to_is_marked_on_the_glass() {
    let mut model = stocked();
    let painted = typed(&mut model, &[egui::Key::ArrowRight]);
    assert!(
        painted
            .lines()
            .any(|line| line == format!("{HERE} {}", convs::HEADING)),
        "{painted}"
    );
    assert!(
        painted.lines().any(|line| line == roster::HEADING),
        "the pane that does not hold the arrows wears no mark:\n{painted}"
    );
    let painted = typed(&mut model, &[egui::Key::ArrowLeft]);
    assert!(
        painted
            .lines()
            .any(|line| line == format!("{HERE} {}", roster::HEADING)),
        "{painted}"
    );
}

/// **Moving in a list selects**, so the cursor and the selection are one thing
/// and there is nothing to keep in step — the highlight the pointer paints is
/// where the keyboard is.
#[test]
fn the_arrows_walk_the_roster_and_the_walk_is_the_aim() {
    let mut model = Model {
        aim: None,
        ..stocked()
    };
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("home".to_owned()),
        "a list nobody has entered opens at its first row"
    );
    typed(&mut model, &[egui::Key::ArrowDown]);
    let aim = model.aim.as_ref().expect("an aim");
    assert_eq!(
        (aim.channel.as_str(), aim.address.as_str()),
        ("elsewhere", "elsewhere"),
        "the walk crosses channels in the order the pane paints them"
    );
    typed(&mut model, &[egui::Key::ArrowUp]);
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("home".to_owned())
    );
}

/// **A key walks only rows a click can reach.** A wall this seat holds no name
/// for is addressed by no envelope it can write, so the pointer offers it as a
/// plain line and the keyboard skips it — one question, asked once, by both.
#[test]
fn the_walk_skips_every_row_no_gesture_could_address() {
    let model = stocked();
    let track = roster::aimable(&model);
    assert_eq!(
        track
            .iter()
            .map(|aim| aim.address.clone())
            .collect::<Vec<String>>(),
        vec!["home".to_owned(), "elsewhere".to_owned()],
        "the second channel's unreachable wall is on neither surface"
    );
    // Down twice lands on the last addressable row and stays there rather than
    // stepping onto the row nothing can address.
    let mut walked = Model { aim: None, ..model };
    for _ in 0..3 {
        typed(&mut walked, &[egui::Key::ArrowDown]);
    }
    assert_eq!(
        walked.aim.as_ref().map(|aim| aim.address.clone()),
        Some("elsewhere".to_owned())
    );
}

/// The conversation list walks the same way, and the selection it moves is the
/// one every read follows.
#[test]
fn the_arrows_walk_the_conversation_list_and_the_walk_is_the_selection() {
    let mut model = Model {
        conversation: None,
        ..stocked()
    };
    typed(&mut model, &[egui::Key::ArrowRight, egui::Key::ArrowDown]);
    assert_eq!(model.conversation.as_deref(), Some("a"));
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(model.conversation.as_deref(), Some("b"));
    // The end saturates: the same press means *next* every time, and never
    // "back to the top" without anything on the glass to say so.
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(model.conversation.as_deref(), Some("b"));
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

/// The default is the roster, because a seat with nothing aimed at has exactly
/// one thing to do next.
#[test]
fn a_fresh_window_opens_with_the_roster_holding_the_arrows() {
    assert_eq!(Model::default().focus, Pane::Roster);
    assert_eq!(
        roster::aimable(&Model::default()),
        Vec::<crate::ui::Aim>::new(),
        "a box that has been asked nothing offers no row to walk"
    );
}

/// **A row the pointer can aim at is a row a key can aim at.** The pending row
/// a start puts in the list is in the one list both walk, so an arrow leaves it
/// for the first conversation the engine actually answered — which is also how
/// an operator escapes a claim whose driver never wrote its branch.
#[test]
fn the_arrows_walk_the_started_conversation_s_row_like_any_other() {
    let mut model = Model {
        conversation: Some("brisk-otter".to_owned()),
        start: Some(crate::ui::model::Start {
            address: "home".to_owned(),
            goal: "port it".to_owned(),
            phase: crate::ui::model::Phase::Started("brisk-otter".to_owned()),
        }),
        ..stocked()
    };
    let painted = typed(&mut model, &[egui::Key::ArrowRight]);
    assert!(painted.contains("brisk-otter"), "{painted}");
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.conversation.as_deref(),
        Some("a"),
        "one step down out of the pending row and into the list"
    );
}
