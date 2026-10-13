//! **The two panels at the head of the pane** (DESIGN §4.42, item 4): the
//! conversation's facts over its transcript, and the seat's where nothing is
//! selected — the baseline's session panel, with this seat's facts in it.
//!
//! A framed card, one line a fact, in the ink scale: the conversation's name
//! and the wall and engine it is on; the model that answered and what it has
//! spent (§4.41's words); where it hangs where it hangs under anything
//! (§4.29). The seat's card names each engine this seat reaches with its dial
//! state and what it holds, the seat's own version, and the two ways in —
//! because a window with nothing selected has exactly two things to do next.

use crate::ui::{Model, theme};

/// The seat's line: its own name and version.
pub fn version() -> String {
    crate::cli::version()
}
/// The two ways in, said once under the seat's card.
pub const WAYS_IN: &str = "/help for commands · Ctrl+X for a conversation";

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

/// **The seat's card**: each engine, the version, and the two ways in.
pub fn seat(ui: &mut egui::Ui, model: &Model) {
    card(ui, |ui| {
        line(ui, &version(), theme::INK);
        for chunk in &model.roster {
            let walls = chunk.walls.len();
            let conversations: u64 = chunk.walls.iter().map(|wall| wall.agents).sum();
            let asking: u64 = chunk.walls.iter().map(|wall| wall.attention).sum();
            let waiting = if asking > 0 {
                format!(", {asking} waiting")
            } else {
                String::new()
            };
            let said = format!(
                "{} — {walls} {}, {conversations} {}{waiting}",
                crate::ui::roster::header(&chunk.channel),
                if walls == 1 { "wall" } else { "walls" },
                if conversations == 1 {
                    "conversation"
                } else {
                    "conversations"
                }
            );
            let ink = if asking > 0 {
                theme::accent(theme::State::Attention)
            } else {
                theme::INK_WEAK
            };
            line(ui, &said, ink);
        }
        line(ui, WAYS_IN, theme::INK_WEAK);
    });
}

#[cfg(test)]
mod tests;
