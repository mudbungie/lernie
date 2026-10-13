//! **The status rule**: one line between the transcript and the prompt that
//! says where you are and what it costs (DESIGN §4.42, item 2;
//! `docs/STYLE.md` §2, *the rule says where you are*).
//!
//! Left to right: `─ <state> · <age> │ <model> │ <spend> │ <N waiting> │
//! <N engines> ─ <name>`. The state is the wire's own word for the selected
//! conversation in its state's ink, `ready` in weak ink where nothing is
//! selected — and **the notice stands in that slot** in its own ink while
//! there is one: what the seat last heard that was not content is a state of
//! the window, read where the window's state is read, and Esc puts it down
//! (`Model::escape`). The segments are the conversation header's facts moved
//! onto one line: the model that answered, what the conversation has spent
//! (§4.41's words, never `$0`), how many conversations across every engine
//! are asking — in the attention accent, the one green on a resting window —
//! and how many engines this seat reaches. The right end is the selected
//! conversation's name, the aimed wall's where none is selected, and the word
//! `engines` where nothing is aimed.
//!
//! **Segments drop from the right as the width runs out, the name last**:
//! a rule that wrapped would be two lines, and a rule that elided the name
//! would say everything but where you are.

use crate::ui::{Model, theme};

/// The glyph the rule opens with, and the one that parts the facts from the
/// name.
pub const LEAD: &str = theme::glyph::LEAD;
/// What parts one fact from the next.
pub const SEP: &str = theme::glyph::SEP;
/// The state slot's word where nothing is selected and nothing is heard.
pub const READY: &str = "ready";
/// The right end where nothing is aimed.
pub const ENGINES: &str = "engines";

/// One run of the rule: its words and its ink.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub ink: egui::Color32,
}

fn run(text: impl Into<String>, ink: egui::Color32) -> Run {
    Run {
        text: text.into(),
        ink,
    }
}

/// **The state slot**: the notice, else the selected conversation's state
/// and age, else `ready`.
pub fn state(model: &Model) -> Run {
    if let Some(notice) = &model.notice {
        let ink = match notice {
            crate::ui::Notice::Said(_) => theme::accent(theme::State::Annotation),
            _ => theme::accent(theme::State::Error),
        };
        return run(notice.line(), ink);
    }
    match selected(model) {
        Some(row) => run(
            format!(
                "{} · {}",
                row.state.label(),
                crate::ui::convs::age(row.age_secs)
            ),
            theme::state_ink(&row.state),
        ),
        None => run(READY, theme::INK_WEAK),
    }
}

/// The selected conversation's own row, where the list holds one.
fn selected(model: &Model) -> Option<crate::reply::convs::ConvRow> {
    let id = model.conversation.as_ref()?;
    model.rows().into_iter().find(|row| row.root_id == *id)
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

/// How many conversations are asking, across every engine this seat reaches.
pub fn waiting(model: &Model) -> u64 {
    model
        .roster
        .iter()
        .flat_map(|chunk| chunk.walls.iter())
        .map(|wall| wall.attention)
        .sum()
}

/// **The facts between the state and the name**, in order, each a run.
pub fn facts(model: &Model) -> Vec<Run> {
    let mut out = Vec::new();
    if let Some(model_id) = answered_by(model) {
        out.push(run(model_id, theme::INK_WEAK));
    }
    if let Some(agent) = &model.records.agent {
        out.push(run(
            crate::ui::records::header::costing(agent),
            theme::INK_WEAK,
        ));
    }
    let asking = waiting(model);
    if asking > 0 {
        out.push(run(
            format!("{asking} waiting"),
            theme::accent(theme::State::Attention),
        ));
    }
    let engines = model.roster.len();
    out.push(run(
        if engines == 1 {
            "1 engine".to_owned()
        } else {
            format!("{engines} engines")
        },
        theme::INK_WEAK,
    ));
    out
}

/// **The right end**: the conversation, the wall, or the window.
pub fn name(model: &Model) -> String {
    if let Some(row) = selected(model) {
        return row.display;
    }
    model
        .aim
        .as_ref()
        .map_or_else(|| ENGINES.to_owned(), |aim| aim.address.clone())
}

/// The width one run takes at the rule's size.
fn width(ui: &egui::Ui, text: &str) -> f32 {
    let font = egui::TextStyle::Small.resolve(ui.style());
    ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font, theme::INK)
            .size()
            .x
    })
}

/// **Which facts fit** beside the state and the name at this width: the
/// leading ones, dropped from the right until the line holds.
pub fn fitting(ui: &egui::Ui, state: &Run, facts: &[Run], name: &str, room: f32) -> usize {
    let gap = ui.spacing().item_spacing.x;
    let sep = width(ui, SEP) + 2.0 * gap;
    let fixed = width(ui, LEAD) * 2.0 + width(ui, &state.text) + width(ui, name) + 6.0 * gap;
    let mut used = fixed;
    let mut shown = 0;
    for fact in facts {
        let more = sep + width(ui, &fact.text);
        if used + more > room {
            break;
        }
        used += more;
        shown += 1;
    }
    shown
}

/// Paint the rule.
pub fn render(ui: &mut egui::Ui, model: &Model) {
    let state = state(model);
    let facts = facts(model);
    let name = name(model);
    let shown = fitting(ui, &state, &facts, &name, ui.available_width());
    let small = |text: &str, ink| egui::RichText::new(text).small().color(ink);
    ui.horizontal(|ui| {
        ui.label(small(LEAD, theme::INK_FAINT));
        ui.label(small(&state.text, state.ink));
        for fact in facts.iter().take(shown) {
            ui.label(small(SEP, theme::INK_FAINT));
            ui.label(small(&fact.text, fact.ink));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(small(&name, theme::INK));
            ui.label(small(LEAD, theme::INK_FAINT));
        });
    });
}

#[cfg(test)]
mod tests;
