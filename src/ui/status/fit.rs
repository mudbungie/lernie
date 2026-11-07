//! **What fits the status bar's line** (DESIGN §4.42 item 11's width rule):
//! the width a run takes at the bar's size, the cut a run is held to, and
//! which runs drop from the right.

use super::{LEAD, Run, SEP};
use crate::ui::theme;

/// The width `text` takes at the rule's size.
pub fn width(ui: &egui::Ui, text: &str) -> f32 {
    let font = egui::TextStyle::Small.resolve(ui.style());
    ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font, theme::INK)
            .size()
            .x
    })
}

/// **`text` cut to `room`**, an ellipsis where it was cut.
pub fn elide(ui: &egui::Ui, text: &str, room: f32) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    let mut said = text.to_owned();
    while width(ui, &said) > room && !chars.is_empty() {
        chars.pop();
        said = format!(
            "{}{}",
            chars.iter().collect::<String>(),
            theme::glyph::ELIDED
        );
    }
    said
}

/// One run's width with the mark inside it and the separator before it.
fn cost(ui: &egui::Ui, run: &Run) -> f32 {
    let gap = ui.spacing().item_spacing.x;
    let mark = run
        .mark
        .map_or(0.0, |(glyph, _)| width(ui, &format!(" {glyph}")));
    width(ui, SEP) + 2.0 * gap + width(ui, &run.text) + mark
}

/// **How the runs fit `room` beside a name `name` wide**: how many are shown
/// (the `kept` never drop; the rest drop from the right), and the width each
/// kept run's words are cut to where even they do not fit.
pub fn fitting(ui: &egui::Ui, runs: &[Run], kept: usize, name: f32, room: f32) -> (usize, f32) {
    let gap = ui.spacing().item_spacing.x;
    let mut used = width(ui, LEAD) * 2.0 + name + 4.0 * gap;
    let core: f32 = runs.iter().take(kept).map(|run| cost(ui, run)).sum();
    if used + core > room {
        // **Shared evenly**: the kept runs' words split what the line has
        // left once their separators and marks are paid — and since the
        // model left the line for the box (DESIGN §4.48 item 4) the provider
        // is the one kept run, so its share is the whole of it.
        let words: f32 = runs.iter().take(kept).map(|r| width(ui, &r.text)).sum();
        let left = room - used - (core - words);
        let n = runs.iter().take(kept).map(|_| 1.0).sum::<f32>().max(1.0);
        return (kept, (left / n).max(0.0));
    }
    used += core;
    let mut shown = kept;
    for run in runs.iter().skip(kept) {
        used += cost(ui, run);
        if used > room {
            break;
        }
        shown += 1;
    }
    (shown, f32::INFINITY)
}

/// **Whether the runs that never drop fit `room` whole** beside `name` —
/// the test a run that may stand inline or above the line is judged by
/// (the dial sentence, `super::seat`).
pub fn fits(ui: &egui::Ui, runs: &[Run], kept: usize, name: f32, room: f32) -> bool {
    width(ui, LEAD) * 2.0
        + name
        + 4.0 * ui.spacing().item_spacing.x
        + runs.iter().take(kept).map(|run| cost(ui, run)).sum::<f32>()
        <= room
}
