//! **The founding half of the composer** (DESIGN §4.42 item 7; bl-673e): the
//! one line that names a workspace that does not exist yet and says what its
//! first conversation is for.
//!
//! It is the start's own box with one more word in front, and nothing else:
//! a fresh engine holds no wall this client is registered in, so there is no
//! wall to aim and no `+ new` to pick, and the engine's own remedy is *a
//! workspace this client founds itself registers it too* (REMOTE §4). One
//! line rather than a two-step because the start already reads one line and
//! a held name between two Enters would be a second state to put down.

use crate::ui::{Fill, Model};

/// **What the box asks for while founding**: the shape of the line, then
/// the key.
pub const HINT: &str = "new workspace: <name> <what it is for> — Enter founds it";
/// What the box does while founding.
pub const BOX_HELP: &str =
    "the new workspace's name and purpose on one line; Enter founds it and begins";

/// Paint the founding box and take what it was given.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let wanted = model.filling() == Some(Fill::Goal);
    let slashed = crate::ui::slash::parsed(&model.draft).is_some();
    let entry = crate::ui::theme::paint::prompt(
        ui,
        egui::Id::new(crate::ui::keys::BOX_ID),
        &mut model.draft,
        HINT,
        None,
        BOX_HELP,
    );
    // **The start's two tokens**: the founding IS a start, whose `prepare`
    // happens to name a workspace nobody has made yet.
    crate::ui::act::tag(&entry, &[crate::verbs::PREPARE, crate::verbs::PROMPT]);
    if wanted {
        entry.request_focus();
    }
    if slashed {
        return;
    }
    if super::entered(ui, &entry) {
        model.found();
    }
}

#[cfg(test)]
mod tests;
