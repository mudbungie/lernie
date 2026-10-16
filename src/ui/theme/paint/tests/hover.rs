//! **What lifts under the pointer** (DESIGN §4.44 item 5): a row, a link and
//! a tab wear [`hover`] while the pointer is over them and nothing at rest;
//! a chosen row keeps its `RAISED`; and the link's mark is inside its seat.

use super::super::{link, link_marked, row, tab};
use crate::paint_probe::{fills_of, frame::Window, seen_of};
use crate::ui::theme::{INK, INK_FAINT, RAISED, hover};

/// One frame of `body` with the pointer at `at`, after one that laid it —
/// egui reads a hover off the previous frame's rects.
fn under(at: egui::Pos2, body: impl FnMut(&egui::Context)) -> egui::FullOutput {
    let window = Window::sized(300.0, 100.0);
    let mut body = body;
    window.frame(Vec::new(), &mut body);
    window.frame(vec![egui::Event::PointerMoved(at)], &mut body)
}

/// Whether one frame filled anything in the hover wash.
fn lifted(output: &egui::FullOutput) -> bool {
    fills_of(output).iter().any(|(_, fill)| *fill == hover())
}

/// Paint each pickable shape on its own line.
fn pickables(ctx: &egui::Context, chosen: bool) {
    egui::CentralPanel::default().show(ctx, |ui| {
        row(ui, "a row", INK, None, chosen, 0.0, "a row");
        link(ui, "a link", INK, "a link");
        tab(ui, "a tab", INK, "a tab");
    });
}

/// **Under the pointer each lifts, and nothing does away from it** — the
/// wash is the hover's alone, the brand at three quarters of the selection's.
#[test]
fn a_row_a_link_and_a_tab_lift_under_the_pointer() {
    let away = under(egui::pos2(290.0, 95.0), |ctx| pickables(ctx, false));
    assert!(!lifted(&away), "bare at rest");
    for words in ["a row", "a link", "a tab"] {
        let laid = under(egui::pos2(290.0, 95.0), |ctx| pickables(ctx, false));
        let at = seen_of(&laid)
            .into_iter()
            .find(|run| run.text == words)
            .unwrap_or_else(|| panic!("{words:?} on the glass"))
            .shown
            .center();
        assert!(lifted(&under(at, |ctx| pickables(ctx, false))), "{words}");
    }
}

/// **A chosen row under the pointer keeps its own, stronger tint.**
#[test]
fn a_chosen_row_keeps_its_raise_under_the_pointer() {
    let at = egui::pos2(20.0, 15.0);
    let output = under(at, |ctx| pickables(ctx, true));
    let fills = fills_of(&output);
    assert!(fills.iter().any(|(_, fill)| *fill == RAISED), "raised");
    let row = fills
        .iter()
        .find(|(rect, _)| rect.contains(at))
        .map(|(_, fill)| *fill);
    assert_ne!(row, Some(hover()), "not washed over");
}

/// **The mark is in the door**: one run on the glass, `<words> <mark>`, and
/// a pointer on the mark lifts the whole seat and is the hand.
#[test]
fn the_mark_is_inside_the_link() {
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            link_marked(ui, "model", INK, ("⏶", INK_FAINT), "opens");
        });
    };
    let laid = under(egui::pos2(290.0, 95.0), body);
    let run = seen_of(&laid)
        .into_iter()
        .find(|run| run.text == "model ⏶")
        .expect("one run");
    let on_mark = egui::pos2(run.shown.max.x - 2.0, run.shown.center().y);
    let output = under(on_mark, body);
    assert!(lifted(&output), "the mark lifts the seat");
    assert_eq!(
        output.platform_output.cursor_icon,
        egui::CursorIcon::PointingHand
    );
}

/// **A block row's seat is the block** (bl-232e): its aside and the lines
/// beneath stand inside the one rect the wash and the chosen tint fill.
#[test]
fn a_block_row_is_one_seat_over_every_line() {
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::super::Row {
                mark: ("?".to_owned(), INK),
                words: "head".to_owned(),
                ink: INK,
                state: None,
                chosen: false,
                indent: 0.0,
                aside: Some("2m".to_owned()),
                beneath: vec![("beneath".to_owned(), INK_FAINT, egui::TextStyle::Small)],
                door: None,
                help: "a block".to_owned(),
            }
            .paint(ui);
        });
    };
    let laid = under(egui::pos2(290.0, 95.0), body);
    let at = |text: &str| {
        seen_of(&laid)
            .into_iter()
            .find(|run| run.text == text)
            .unwrap_or_else(|| panic!("{text:?}"))
            .shown
    };
    let (aside, beneath) = (at("2m"), at("beneath"));
    assert!(
        aside.left() > at("? head").right(),
        "the aside at the right end"
    );
    let output = under(beneath.center(), body);
    let block = fills_of(&output)
        .into_iter()
        .find(|(_, fill)| *fill == hover())
        .expect("the pointer on a line beneath lifts the block")
        .0;
    assert!(block.contains(aside.center()) && block.contains(beneath.center()));
}

/// **A row's door tells its own sentence** (bl-a6b3): with the pointer
/// resting on the door, the door's help is on the glass and the row's is
/// not — egui would otherwise keep the row's tooltip open for as long as
/// the pointer stays inside the row, which the door is.
#[test]
fn a_doored_row_tells_the_door_under_the_pointer_and_itself_elsewhere() {
    let body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::super::Row {
                mark: (String::new(), INK),
                words: "head".to_owned(),
                ink: INK,
                state: None,
                chosen: false,
                indent: 0.0,
                aside: None,
                beneath: Vec::new(),
                door: Some(("×", "the door's sentence")),
                help: "the row's sentence".to_owned(),
            }
            .paint_doored(ui);
        });
    };
    let window = Window::sized(300.0, 100.0);
    // **No delay and no grace**: the clock here is egui's own stand-in, a
    // frame at a time, and what is under test is which sentence shows.
    window.ctx().style_mut(|style| {
        style.interaction.tooltip_delay = 0.0;
        style.interaction.tooltip_grace_time = 0.0;
    });
    let mut body = body;
    let said = |output: &egui::FullOutput, words: &str| {
        seen_of(output).into_iter().any(|run| run.text == words)
    };
    window.frame(Vec::new(), &mut body);
    let on_row = egui::pos2(20.0, 15.0);
    let mut output = window.frame(vec![egui::Event::PointerMoved(on_row)], &mut body);
    for _ in 0..12 {
        output = window.frame(Vec::new(), &mut body);
    }
    assert!(said(&output, "the row's sentence"), "the row tells");
    let door = seen_of(&output)
        .into_iter()
        .find(|run| run.text == "×")
        .expect("the door stands while the pointer is on the row")
        .shown
        .center();
    output = window.frame(vec![egui::Event::PointerMoved(door)], &mut body);
    for _ in 0..12 {
        output = window.frame(Vec::new(), &mut body);
    }
    assert!(said(&output, "the door's sentence"), "the door tells");
    assert!(!said(&output, "the row's sentence"), "the row stands down");
}
