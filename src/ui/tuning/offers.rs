//! **What a provider row offers, on the tuning pane** (bl-b7d9): the mark on
//! a row whose model its provider does not offer, and the offered ids under
//! the editor.
//!
//! Split from [`super`] on the seam the read draws: the rest of the pane is
//! the `roles` answer, and this is the `models` one — the read `lernie model`
//! spends before it writes (`crate::seat::model`), standing beside `roles`
//! while the pane is open (`crate::offframe::asker`). What is filed and what
//! counts as evidence is the model's (`crate::ui::model::tuning`).

use crate::reply::roles::RoleRow;
use crate::ui::{Edit, Model, theme};

/// The word over the offered ids in the editor.
pub const OFFERS: &str = "it offers:";
/// What picking an offered id does.
pub const PICK_HELP: &str = "puts this id in the model box";

/// **Say, on the row and in the error accent, that its provider does not
/// offer its model** — or nothing, which is also what a provider with no
/// listing earns.
pub fn mark(ui: &mut egui::Ui, model: &Model, row: &RoleRow) {
    if let Some(said) = model.unoffered(row) {
        ui.colored_label(theme::accent(theme::State::Error), said);
    }
}

/// **The draft's provider's ids, each a seat that fills the model field.**
/// The field stays free text: a list can be stale, and an id the operator
/// knows is theirs to send (`crate::seat::model`'s *a warning, and never a
/// refusal*).
pub fn pick(ui: &mut egui::Ui, model: &mut Model, edit: &Edit) {
    let offered = model.offered_by(&edit.provider);
    if offered.is_empty() {
        return;
    }
    let mut picked = None;
    ui.horizontal_wrapped(|ui| {
        ui.label(OFFERS);
        for id in offered {
            if theme::paint::seat(ui, edit.model.trim() == id, &id, PICK_HELP).clicked() {
                picked = Some(id);
            }
        }
    });
    if let Some(id) = picked
        && let Some(draft) = model.draft_assignment()
    {
        draft.model = id;
    }
}

#[cfg(test)]
mod tests;
