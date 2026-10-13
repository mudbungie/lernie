//! **The two panels at the head of the pane** (DESIGN §4.42, item 4): the
//! conversation's facts over its transcript, and the seat's where nothing is
//! selected — the baseline's session panel, with this seat's facts in it.
//!
//! A framed card, one line a fact, in the ink scale: the conversation's name
//! and the wall and engine it is on; the model that answered and what it has
//! spent (§4.41's words); where it hangs where it hangs under anything
//! (§4.29). The seat's card names each engine this seat reaches with its dial
//! state and what it holds, the seat's own version, and the ways in — the
//! verbs, the conversation, the model and the provider, because those are
//! what a window with nothing selected is for and none of them is on the
//! glass until it is named (bl-4e55).

use crate::ui::{Model, theme};

/// The seat's line: its own name and version.
pub fn version() -> String {
    crate::cli::version()
}
/// **The ways in**, in the order they are dropped last to first where a
/// line has no room for all of them (`crate::ui::composer::hint`): the verbs,
/// what you talk in, what you talk to, and who pays for it.
pub const WAYS: [&str; 4] = [
    "/help for commands",
    "Ctrl+X for a conversation",
    "/roles for the model",
    "/login for a provider",
];
/// What parts one way from the next.
pub const PART: &str = " · ";

/// **The way to a wall where there is none** (bl-673e): the switcher's row
/// that founds one, named on the card because a fresh engine holds nothing
/// this seat is in and the card is the first thing it reads.
pub const FOUNDING: &str = "Ctrl+X then + new workspace to found one";

/// The ways in as one line, said once under the seat's card, where it wraps.
pub fn ways_in() -> String {
    format!("{}{PART}{FOUNDING}", WAYS.join(PART))
}

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

/// **The seat's card**: each engine with its dial state, the version, the
/// queue's door while anything waits, and the ways in. Answers whether the
/// door was clicked.
pub fn seat(ui: &mut egui::Ui, model: &Model) -> bool {
    let mut asked = false;
    card(ui, |ui| {
        line(ui, &version(), theme::INK);
        // **One line an engine, not a channel** (bl-13b9): two names for one
        // address are one server, counted once (`crate::ui::model::fold`).
        for engine in model.folded() {
            let header = engine.header();
            // **The dial state first** (bl-4cac): a channel that will not dial
            // says the wire's own sentence, whole and in the error accent — as
            // does an engine that refused the roster (bl-693d) — and one not
            // heard from yet says so. None has counts worth painting, because a
            // count off walls nobody can reach reads as a healthy engine.
            let state = engine.held();
            if let Some(why) = state.said() {
                line(
                    ui,
                    &format!("{header} — {why}"),
                    theme::accent(theme::State::Error),
                );
                continue;
            }
            if state == crate::ui::Held::Unheard {
                line(
                    ui,
                    &format!("{header} — {}", crate::ui::roster::NOT_ANSWERED),
                    theme::INK_WEAK,
                );
                continue;
            }
            let held = engine.walls();
            let walls = held.len();
            let conversations: u64 = held.iter().map(|wall| wall.agents).sum();
            let asking: u64 = held.iter().map(|wall| wall.attention).sum();
            let waiting = if asking > 0 {
                format!(", {asking} waiting")
            } else {
                String::new()
            };
            let said = format!(
                "{header} — {}, {}{waiting}",
                crate::render::counted(walls as u64, "wall"),
                crate::render::counted(conversations, "conversation"),
            );
            let ink = if asking > 0 {
                theme::accent(theme::State::Attention)
            } else {
                theme::INK_WEAK
            };
            line(ui, &said, ink);
        }
        asked = crate::ui::status::waiting::door(ui, model);
        line(ui, &ways_in(), theme::INK_WEAK);
    });
    asked
}

#[cfg(test)]
mod tests;
