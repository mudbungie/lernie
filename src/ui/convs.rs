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

/// The word this pane wears, and the subject the arrows act on when it is
/// focused.
pub const HEADING: &str = "conversations";
/// The mark a state nothing observed wears, inside the badge it qualifies.
pub const UNCERTAIN: &str = "?";

/// Paint the list and take a click on it.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    ui.heading(crate::ui::keys::heading(
        HEADING,
        model.focus == crate::ui::Pane::Conversations,
    ));
    let Some(aim) = model.aim.clone() else {
        ui.label(NO_WALL);
        return;
    };
    ui.label(aim.address);
    // **The list is the model's, not this pane's**: a conversation this window
    // has started but the engine cannot resolve yet stands in it as a row of
    // its own (`crate::ui::model::claim`), and it stands there for the pointer
    // and the keyboard alike because both walk the one list.
    let rows = model.rows();
    if rows.is_empty() {
        ui.label(NO_CONVERSATIONS);
        return;
    }
    // The list scrolls; the heading and the address above it do not (bl-e5d2,
    // and `crate::ui::roster` for why the heading stays out).
    let reveal = model.revealing(crate::ui::keys::Pane::Conversations);
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            for row in rows {
                conversation(ui, model, &row, reveal);
            }
        });
}

/// One row, indented to its depth under the conversation root.
fn conversation(ui: &mut egui::Ui, model: &mut Model, row: &ConvRow, reveal: bool) {
    let selected = model.conversation.as_ref() == Some(&row.root_id);
    ui.horizontal(|ui| {
        ui.add_space(indent(row.depth));
        let seat = ui.selectable_label(selected, headline(row));
        if selected && reveal {
            seat.scroll_to_me(None);
        }
        if seat.clicked() {
            model.select(&row.root_id.clone());
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
///
/// **The badge wears a `?` for a state nothing observed.** The engine answers a
/// reading and whether it could take one, and painting the first without the
/// second would state a definite fact about a conversation nobody looked at —
/// including this window's own, between a start's receipt and its driver's
/// first write.
pub fn headline(row: &ConvRow) -> String {
    let mut said = vec![format!(
        "{}  [{}{}]  {}",
        row.display,
        row.state.label(),
        if row.uncertain { UNCERTAIN } else { "" },
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
