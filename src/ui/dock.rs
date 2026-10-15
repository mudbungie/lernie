//! **The live dock**: what is running under the aimed wall, folded to one
//! line above the rule and unfolded with Ctrl+T (DESIGN §4.42, item 9).
//!
//! The baseline keeps a dock of its subagents and background processes over
//! its status line, one line while folded. This seat's running things are
//! the aimed wall's conversations whose drivers are at work — live or in
//! flight, by the engine's own word (`AgentState`) — which the roster's rows
//! already carry; so the dock is a reading of those rows and asks nothing of
//! its own. **The fleet's runs join it when their read stands outside the
//! fleet pane** (§4.33); until then they are the pane's.
//!
//! Folded, it is one line: `+ N running · <the first's name>`. Unfolded, one
//! row per running conversation — its name, its state and its age — and a
//! click on a row selects it, which is the one act a dock affords. Whether it
//! is unfolded is the toolkit's memory, as a fold's is (`crate::ui::chat`):
//! a view state no wire gesture corresponds to and no other fact can be
//! asked for; Ctrl+T is the chord (`crate::ui::keys`) and a click on the
//! line — a door, lit under the pointer (DESIGN §4.45 item 6) — is the same
//! toggle.

use crate::reply::convs::{AgentState, ConvRow};
use crate::ui::{Model, theme};

/// **The conversations running under the aimed wall**, in the list's order.
pub fn running(model: &Model) -> Vec<ConvRow> {
    model
        .rows()
        .into_iter()
        .filter(|row| matches!(row.state, AgentState::Live | AgentState::InFlight))
        .collect()
}

/// The folded line's words.
pub fn line(rows: &[ConvRow], unfolded: bool) -> String {
    let count = rows.len();
    let first = rows
        .first()
        .map(|row| row.display.as_str())
        .unwrap_or_default();
    format!(
        "{} {count} running · {first} · Ctrl+T",
        if unfolded {
            theme::glyph::OPEN
        } else {
            theme::glyph::SHUT
        }
    )
}

/// One running conversation's row: its name, its state, its age.
pub fn row_words(row: &ConvRow) -> String {
    format!(
        "{}  {}  {}",
        row.display,
        row.state.label(),
        crate::ui::convs::age(row.age_secs)
    )
}

/// Where the fold is kept.
const UNFOLDED_ID: &str = "the dock's fold";

/// **Whether the dock is unfolded.**
pub fn unfolded(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::new(UNFOLDED_ID)))
        .unwrap_or(false)
}

/// **Unfold the dock, or fold it**: the one toggle Ctrl+T is.
pub fn toggle(ctx: &egui::Context) {
    let now = unfolded(ctx);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(UNFOLDED_ID), !now));
}

/// Paint the dock for `rows`, which the shell has found non-empty.
pub fn render(ui: &mut egui::Ui, model: &mut Model, rows: &[ConvRow]) {
    let open = unfolded(ui.ctx());
    // **The line is a door** (DESIGN §4.45 item 6): a label that sensed a
    // click lit nothing under the pointer, so the one toggle a click on it is
    // was a toggle the glass never offered. The same small run in the working
    // accent, through the one painter that wears the hover wash.
    let head = theme::paint::link(ui, &line(rows, open), theme::accent(theme::State::Working));
    if head.clicked() {
        toggle(ui.ctx());
    }
    if !open {
        return;
    }
    let mut chosen = None;
    for row in rows {
        let seat = theme::paint::row(
            ui,
            &row_words(row),
            theme::INK,
            Some(theme::state_ink(&row.state)),
            model.conversation.as_deref() == Some(row.root_id.as_str()),
            theme::space::S,
        );
        // **The two reads this gesture reaches**, as a row in the switcher
        // (`crate::ui::convs::row`): selecting is what makes this seat read
        // the transcript and open the follow lane.
        crate::ui::act::tag(
            &seat,
            &[crate::verbs::TRANSCRIPT.word, crate::verbs::FOLLOW.word],
        );
        if seat.clicked() {
            chosen = Some(row.root_id.clone());
        }
    }
    if let Some(id) = chosen {
        model.select(&id);
    }
}

#[cfg(test)]
mod tests;
