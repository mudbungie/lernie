//! **The conversation list** for the wall the window is aimed at.
//!
//! A row is a glance, not a transcript: what it is called, what it is doing,
//! how long since it moved, and the first line of what was said. Everything
//! deeper is one click away in the chat pane, and a list that tried to be the
//! pane would be neither.

use crate::reply::convs::ConvRow;
use crate::ui::{Model, theme};

/// What the list says with no wall aimed at.
pub const NO_WALL: &str = "pick a workspace";
/// What it says for a wall that holds nothing.
pub const NO_CONVERSATIONS: &str = "no conversations here";

/// Paint the list and take a click on it.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    ui.heading("conversations");
    let Some(aim) = model.aim.clone() else {
        ui.label(NO_WALL);
        return;
    };
    ui.label(aim.address);
    if model.convs.is_empty() {
        ui.label(NO_CONVERSATIONS);
        return;
    }
    for row in model.convs.clone() {
        conversation(ui, model, &row);
    }
}

/// One row, indented to its depth under the conversation root.
fn conversation(ui: &mut egui::Ui, model: &mut Model, row: &ConvRow) {
    let selected = model.conversation.as_ref() == Some(&row.root_id);
    ui.horizontal(|ui| {
        ui.add_space(indent(row.depth));
        if ui.selectable_label(selected, headline(row)).clicked() {
            model.conversation = Some(row.root_id.clone());
            model.transcript = crate::reply::transcript::Transcript::default();
            model.live = None;
        }
    });
    if !row.preview.is_empty() {
        ui.horizontal(|ui| {
            ui.add_space(indent(row.depth) + 12.0);
            ui.colored_label(theme::tone_ink(&row.tone), &row.preview);
        });
    }
}

/// How far a row hangs under its root, in points.
///
/// Added rather than multiplied, over a **bounded** count: there is no cast
/// from the wire's own width to a screen coordinate, so there is no truncation
/// to suppress a lint about. The cap is not a special case either — past it a
/// list is unreadable whatever the indent says, and the label is what the extra
/// width would have cost.
fn indent(depth: u64) -> f32 {
    const STEP: f32 = 16.0;
    const DEEPEST: u64 = 8;
    (0..depth.min(DEEPEST)).fold(0.0, |at, _| at + STEP)
}

/// A row's headline: its label, what it is doing, how long since it moved, and
/// what is waiting under it.
pub fn headline(row: &ConvRow) -> String {
    let mut said = vec![format!(
        "{}  [{}]  {}",
        row.display,
        row.state.label(),
        age(row.age_secs)
    )];
    if row.attention > 0 {
        said.push(format!("{} waiting", row.attention));
    }
    if row.members > 1 {
        said.push(format!("{} members", row.members));
    }
    said.join("  ")
}

/// A compact age: `42s`, `7m`, `3h`, `2d`. **Negative clamps to zero** — two
/// machines' clocks disagreeing is a fact about a seat that dials somewhere
/// else, and an age in the future is not a thing to paint.
pub fn age(secs: i64) -> String {
    let secs = secs.max(0);
    for (bound, unit, per) in [(60, 's', 1), (3600, 'm', 60), (86_400, 'h', 3600)] {
        if secs < bound {
            return format!("{}{unit}", secs / per);
        }
    }
    format!("{}d", secs / 86_400)
}

#[cfg(test)]
mod tests;
