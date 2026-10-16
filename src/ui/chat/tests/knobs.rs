//! **The two knobs the chat reads** (DESIGN §4.46 item 4): `bubbles` off
//! puts every block on the ground, and `fold` off leaves a settled working
//! and its finished steps whole.

use super::idiom::{head, prose, result, with, working};
use crate::paint_probe::frame::Window;
use crate::paint_probe::{fills_of, seen_of};
use crate::settings::Toggle;
use crate::test_support::window::{numbered, said};
use crate::ui::Model;
use crate::ui::theme::glyph;

/// A settled turn with one call and one note under its working.
fn settled() -> Model {
    with(vec![
        said("op", "port it"),
        prose("first I will read"),
        working(),
        result(&numbered(12), false),
        prose("the seam is real"),
    ])
}

/// What one settled frame put on the glass, as whole runs.
fn runs(window: &Window, model: &mut Model) -> Vec<String> {
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, model));
    seen_of(&output).into_iter().map(|run| run.text).collect()
}

/// **Bubbles off, no column-wide fill stands under any block** — the air is
/// kept, so the runs stand where they stood.
#[test]
fn bubbles_off_puts_every_block_on_the_ground_and_keeps_its_place() {
    let window = Window::sized(1400.0, 900.0);
    let mut model = settled();
    let on = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let wide = |output: &egui::FullOutput| {
        fills_of(output)
            .into_iter()
            .filter(|(rect, ink)| rect.width() > 300.0 && rect.height() < 400.0 && ink.a() > 0)
            .count()
    };
    // The transcript's runs, which stand above the composer: the box's own
    // height eases between frames and is no fact about the bubbles.
    // Rounded to the point: the two frames lay the same runs a hair apart.
    let laid = |output: &egui::FullOutput| -> Vec<(String, egui::Pos2, egui::Pos2)> {
        seen_of(output)
            .into_iter()
            .filter(|s| s.laid.min.y > 100.0 && s.laid.max.y < 300.0)
            .map(|s| (s.text, s.laid.min.round(), s.laid.max.round()))
            .collect()
    };
    let where_on = laid(&on);
    assert!(where_on.len() >= 6, "{where_on:?}");
    assert!(wide(&on) >= 3, "{}", wide(&on));
    model.settings.set(Toggle::Bubbles, false);
    let off = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    assert!(wide(&off) < wide(&on), "{} vs {}", wide(&off), wide(&on));
    assert_eq!(where_on, laid(&off), "the air is the same");
}

/// **Fold off, the settled working stands open and its finished steps
/// whole**: the counted line still heads it, and the call's output is on
/// the glass with no click.
#[test]
fn fold_off_leaves_the_working_and_its_steps_whole() {
    let window = Window::sized(1400.0, 900.0);
    let mut model = settled();
    model.settings.set(Toggle::Fold, false);
    let shown = runs(&window, &mut model);
    assert!(shown.contains(&head(true, 1, 2)), "{shown:?}");
    assert!(
        shown.iter().any(|t| t.contains("line 001")),
        "the output whole: {shown:?}"
    );
    assert!(
        !shown.contains(&format!("{} read(src/ui.rs)", glyph::DONE)),
        "no compact step: {shown:?}"
    );
}
