//! **The panel at the head of a transcript** (DESIGN §4.42, item 4): the
//! conversation's facts over its transcript.
//!
//! A framed card, one line a fact, in the ink scale: the conversation's name
//! and the workspace and server it is on; the model that answered and what it has
//! spent (§4.41's words); where it hangs where it hangs under anything
//! (§4.29). The seat's card that stood where nothing is selected is gone
//! (§4.48 item 1): its facts are the status bar's right end and the
//! switcher's head, and the empty column is the mark and one line.

use crate::ui::{Model, theme};

/// **The ways in**, in the order they are dropped last to first where a
/// line has no room for all of them (`crate::ui::composer::hint`): the verbs,
/// what you talk in, what you talk to, and who pays for it.
pub const WAYS: [&str; 4] = [
    "/help for commands",
    "Ctrl+X for a conversation",
    "Ctrl+O for the model",
    "/login for a provider",
];
/// What parts one way from the next.
pub const PART: &str = " · ";

/// The card every panel is laid in.
fn card(ui: &mut egui::Ui, body: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0, theme::HAIRLINE))
        .rounding(theme::RADIUS)
        .inner_margin(egui::Margin::symmetric(theme::space::M, theme::space::S))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            body(ui);
        });
    ui.add_space(theme::space::M);
}

/// One fact, wrapped, in the ink handed in.
fn line(ui: &mut egui::Ui, said: &str, ink: egui::Color32) {
    ui.add(egui::Label::new(egui::RichText::new(said).color(ink)).wrap());
}

/// The model that last answered on the transcript, where one did.
fn answered_by(model: &Model) -> Option<String> {
    model
        .transcript
        .entries
        .iter()
        .rev()
        .find_map(|entry| match &entry.kind {
            crate::reply::transcript::EntryKind::Model { model_id, .. } => Some(model_id.clone()),
            _ => None,
        })
}

/// **The conversation's card**: its name, where it is, what answered and what
/// it cost, and where it hangs.
pub fn conversation(ui: &mut egui::Ui, model: &Model, id: &str) {
    let row = model.rows().into_iter().find(|row| row.root_id == id);
    card(ui, |ui| {
        let name = row
            .as_ref()
            .map_or_else(|| id.to_owned(), |row| row.display.clone());
        line(ui, &name, theme::INK);
        if let Some(aim) = &model.aim {
            line(
                ui,
                &format!("on {} — {}", aim.address, aim.channel),
                theme::INK_WEAK,
            );
        }
        let mut cost = Vec::new();
        if let Some(model_id) = answered_by(model) {
            cost.push(model_id);
        }
        if let Some(agent) = model.agent() {
            cost.push(crate::ui::records::header::costing(agent));
        }
        if !cost.is_empty() {
            line(ui, &cost.join(" · "), theme::INK_WEAK);
        }
        if let Some(path) = model.agent().and_then(crate::ui::records::header::descent) {
            line(ui, &path, theme::INK_WEAK);
        }
    });
}

#[cfg(test)]
mod tests;
