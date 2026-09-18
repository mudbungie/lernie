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
