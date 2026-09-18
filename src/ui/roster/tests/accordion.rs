//! **The accordion** (DESIGN §4.39): which rows reached the glass, which
//! engine's walls stand under it, and the click that opens one.
//!
//! Split from [`super`] on the seam the pane has: that file reads what one
//! ROW says, `sections` what one engine says about itself, and this what the
//! arrangement of them is. The three fail for different reasons — a row about
//! a name, a section about a relationship, and this about an order.

use super::super::{HEADING, header, ordered, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, own, pane, seated, wall};
use crate::ui::{Channel, Chunk, Model};

/// One engine holding the walls it is given.
fn engine(name: &str, walls: &[&str]) -> Chunk {
    Chunk {
        channel: Channel {
            name: name.to_owned(),
            named_there: None,
            dials: None,
        },
        held: crate::ui::Held::Heard,
        walls: walls.iter().map(|name| wall(name)).collect(),
        ..Chunk::default()
    }
}

/// A seat on its own engine and one other, the other holding two walls.
fn two() -> Model {
    Model {
        roster: vec![own(), engine("lab", &["bench", "annex"])],
        ..seated()
    }
}

/// **The word on the glass is *engines*** (§4.39), and the crate's word stays
/// *channel*: the rename is of a word a person reads, and one home for it is
/// §4.11's whole rule.
#[test]
fn the_pane_is_called_engines() {
    assert_eq!(HEADING, "engines");
}

/// **At most one engine is open: its walls paint under it, and a closed one
/// paints only its row.** Every engine's row is on the glass, because the list
/// is what an operator picks from.
#[test]
fn the_open_engine_s_walls_paint_and_the_closed_one_s_do_not() {
    let mut model = two();
    let painted = pane(|ui| render(ui, &mut model));
    for expected in [header(&own().channel), "lab".to_owned()] {
        assert!(painted.contains(&expected), "{expected:?}:\n{painted}");
    }
    assert!(
        painted.contains("home  (named)"),
        "the open engine's wall:\n{painted}"
    );
    for folded in ["bench", "annex"] {
        assert!(
            !painted.contains(folded),
            "the closed engine's walls are folded away: {folded:?}\n{painted}"
        );
    }
}

/// **The open engine is painted first, and the rest follow the order each was
/// last opened on this seat** — most recent first, the name breaking a tie.
#[test]
fn the_open_engine_is_painted_first() {
    let mut model = two();
    model.open_engine("lab");
    let painted = pane(|ui| render(ui, &mut model));
    let at = |needle: &str| {
        painted
            .find(needle)
            .unwrap_or_else(|| panic!("{needle:?} is on the glass:\n{painted}"))
    };
    assert!(at("lab") < at("(this box's own engine)"));
    assert!(at("lab") < at("bench"), "and its walls stand under it");
}

/// **Clicking an engine's row opens it and closes the other**, and opening
/// aims the wall last aimed under it — its first by [`ordered`] where this
/// seat has never aimed one, so an open engine is never open over nothing.
#[test]
fn a_click_on_an_engine_row_opens_it_closes_the_other_and_aims_a_wall() {
    let mut model = two();
    let window = Window::new();
    click(&window, "lab", |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert_eq!(model.engine_open().as_deref(), Some("lab"));
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some(ordered(&model.roster[1].walls)[0].workspace.clone()),
        "its first wall by `ordered`"
    );
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("annex"), "{painted}");
    assert!(
        !painted.contains("home  (named)"),
        "the engine that was open is closed:\n{painted}"
    );
}

/// **A closed engine still says what it cannot do** (bl-e620): the shell-wide
/// bar holds one sentence and the last writer wins, so a seat with two
/// unreachable engines has to be able to discover both from the glass. What
/// the accordion folds away is the WALLS.
#[test]
fn a_closed_engine_still_says_it_cannot_be_dialled() {
    let mut model = two();
    model.roster[1].held = crate::ui::Held::Unheld("connect: refused".to_owned());
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("connect: refused"), "{painted}");
    assert!(!painted.contains("bench"), "{painted}");
}

/// **The aimed wall's conversations stand under it, and no other wall's do**
/// (DESIGN §4.39). The fold's whole point, and the decision that came with it:
/// the standing read set asks about one wall (§4.12), so a seat has rows for
/// the aimed wall and no evidence at all about the others — and painting an
/// emptiness under a wall nobody has asked about would be reporting a fact
/// this seat does not hold.
#[test]
fn the_aimed_wall_s_conversations_stand_under_it_and_no_other_wall_s() {
    let mut model = Model {
        roster: vec![Chunk {
            walls: vec![wall("home"), wall("spare")],
            ..own()
        }],
        ..seated()
    };
    let painted = pane(|ui| render(ui, &mut model));
    let at = |needle: &str| {
        painted
            .find(needle)
            .unwrap_or_else(|| panic!("{needle:?} is on the glass:\n{painted}"))
    };
    let row = crate::ui::convs::headline(&model.convs[0].clone());
    assert!(at("home  (named)") < at(&row), "under its own wall's row");
    assert!(at(&row) < at("spare"), "and above the next wall's");
    assert_eq!(
        painted.matches(&row).count(),
        1,
        "once, under the wall it belongs to:\n{painted}"
    );
    // The wall nobody aimed at says nothing at all — not the wait, and not
    // the emptiness.
    assert!(
        !painted.contains(crate::ui::convs::NOT_ANSWERED),
        "{painted}"
    );
}

/// **The `+` at the right edge of an engine's row begins a conversation on
/// it** (DESIGN §4.39): the selection cleared, the aim on the engine's own
/// wall, the engine opened — a start is a use — and the caret asked for in the
/// composer's box.
#[test]
fn the_plus_on_an_engine_row_begins_a_conversation_on_it() {
    let mut model = two();
    let window = Window::new();
    click(&window, super::super::engine::BEGIN, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert_eq!(
        model.engine_open().as_deref(),
        Some("(this box's own engine)"),
        "the first row's `+` is the first engine's"
    );
    assert_eq!(model.conversation, None, "the selection is cleared");
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("home".to_owned())
    );
    assert_eq!(model.fill, Some(crate::ui::Fill::Goal));
}

/// **And it stands down under a covering pane**, which is the per-wall
/// controls' own rule: what it does is put the caret in the composer's box,
/// and the composer stands down under every pane that covers the conversation.
#[test]
fn the_plus_stands_down_while_a_pane_covers_the_conversation() {
    let mut model = two();
    model.begin_records();
    assert!(model.covered(), "the records pane is standing");
    let painted = pane(|ui| render(ui, &mut model));
    assert!(
        !painted.contains(super::super::engine::BEGIN),
        "no + is offered under a covering pane:\n{painted}"
    );
}
