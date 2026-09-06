//! **What a list pane's width actually is on the glass** (bl-fef8).
//!
//! `super::super::policy`'s suite asserts the arithmetic; this asserts that the
//! window obeys it. Three things were measured on a live seat and none of them
//! could be seen from the policy alone: both list panes stuck at the width they
//! had collapsed to at a smaller window, a preview cut mid-sentence at the
//! panel edge with no ellipsis, and a band of window surface beside the list
//! that no panel paints.
//!
//! They are one defect twice: egui stores a side panel's state as the rect its
//! CONTENT took, so a pane narrower than its cap can never grow back, and a
//! pane WIDER than its frame reserves the overrun from the layout while
//! painting none of it.

use super::{render, widths};
use crate::paint_probe::Seen;
use crate::paint_probe::frame::Window;
use crate::test_support::window::{conv, seated, seen};
use crate::ui::Model;

/// A sentence no list pane in this window is wide enough for.
const LONG: &str = "What changed in the last 20 commits of this repository? Write a concise changelog to CHANGES.md";

/// The seated window with one conversation whose preview overruns the pane.
fn overrunning() -> Model {
    let mut model = seated();
    let mut row = conv("20260830T051200Z-a1b2", "port the paint probe");
    row.preview = LONG.to_owned();
    model.convs = vec![row];
    model
}

/// One settled frame's runs. Two passes, because egui lays a panel out on the
/// pass after the one that measured it.
fn glass(window: &Window, model: &mut Model) -> Vec<Seen> {
    window.frame(Vec::new(), |ctx| render(ctx, model));
    seen(window, |ctx| render(ctx, model))
}

/// The run reading exactly `text`, or a failure naming it.
fn run(glyphs: &[Seen], text: &str) -> Seen {
    glyphs
        .iter()
        .find(|shown| shown.text == text)
        .unwrap_or_else(|| panic!("nothing on the glass reads {text:?}"))
        .clone()
}

/// **A window widened is a window whose panes widen with it.**
///
/// The defect: a seat opened at 800 points and then resized to 1440 kept a
/// 175-point roster with every row wrapped after two words, because the panel's
/// width came from the content it had held rather than from the policy. The
/// second window here is the SAME context — the same stored panel state the
/// operator's resized window had.
#[test]
fn a_pane_that_was_narrow_takes_the_width_the_policy_gives_a_wider_window() {
    let mut model = seated();
    // **One context, resized** — the operator dragging a corner. A second
    // window of the new size is a different subject: the state that outvoted
    // the policy was the state the SMALLER window left behind.
    let mut window = Window::sized(800.0, 600.0);
    glass(&window, &mut model);
    window.resize(1440.0, 900.0);
    let glyphs = glass(&window, &mut model);
    let (roster, convs) = widths(1440.0);
    let heading = run(&glyphs, crate::ui::chat::HEADING);
    assert!(
        heading.laid.min.x >= roster + convs - 8.0,
        "the conversation starts where the two lists end: {} against {}",
        heading.laid.min.x,
        roster + convs
    );
}

/// **The preview elides at its own edge, and nothing is reserved past it.**
///
/// Both halves in one beat because they are one defect: an extending label
/// overruns the panel's painted frame, and the side panel reserves the
/// overrun's right edge from the layout — so what a reader loses is the end of
/// the sentence AND the strip of window nothing paints.
#[test]
fn an_overrunning_preview_is_elided_rather_than_cut_and_reserves_no_dead_band() {
    let mut model = overrunning();
    let window = Window::sized(1440.0, 900.0);
    let glyphs = glass(&window, &mut model);
    let preview = glyphs
        .iter()
        .find(|shown| shown.text.starts_with("What changed in the last"))
        .unwrap_or_else(|| panic!("the preview is not on the glass: {glyphs:?}"));
    assert!(
        preview.text.ends_with('…'),
        "the run says it was cut: {:?}",
        preview.text
    );
    assert!(
        preview.laid.width() - preview.shown.width() < 1.0,
        "and nothing of it was clipped away: laid {:?}, shown {:?}",
        preview.laid,
        preview.shown
    );
    let (roster, convs) = widths(1440.0);
    let heading = run(&glyphs, crate::ui::chat::HEADING);
    assert!(
        heading.laid.min.x < roster + convs + 24.0,
        "no band stands between the list and the conversation: {} against {}",
        heading.laid.min.x,
        roster + convs
    );
}
