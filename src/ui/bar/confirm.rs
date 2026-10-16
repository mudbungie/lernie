//! **The deletion's confirmation** (DESIGN §4.44 item 1; §4.42 item 5;
//! bl-e837): the box that stands on the prompt, where the ask stands, while
//! a trashcan's click waits on its answer — the conversation named, and the
//! two doors, `keep` and `delete`, Esc and Enter.
//!
//! It is the ask's shape and not a covering pane because it is a question
//! the row asked a moment ago, answered where the hands are: a pane over the
//! transcript would take the eye off the row that was clicked. **The way
//! out comes first**, on the glass and in the Tab order, as the unmaking
//! pane's does (`crate::ui::unmake`): the control a hand reaches for by
//! reflex is the one that changes nothing — and Enter is the operator's own
//! ruling, *a confirmation dialogue unless ctrl is held*, so a key answers
//! yes and a key answers no and neither is a word to type.
//!
//! **It holds the root id and names the conversation off the listing**
//! (`Model::name_of`), so what it says is what the bar says this frame, and
//! what it deletes is what the row was.

use crate::ui::{Model, theme};

/// The box's name.
pub const HEADING: &str = "delete this conversation?";
/// The two doors.
pub const KEEP: &str = "keep";
pub const DELETE: &str = "delete";
/// The keys, said on the name line.
pub const HINT: &str = "Enter deletes · Esc keeps";
/// What the two answers do.
pub const KEEP_HELP: &str = "keeps the conversation; nothing is sent";
pub const DELETE_HELP: &str = "deletes the conversation on the engine; nothing brings it back";

/// **Paint the box** for `id`, the conversation whose deletion is being
/// asked about (the shell lays the panel only while one is), and take the
/// door clicked.
pub fn render(ui: &mut egui::Ui, model: &mut Model, id: &str) {
    let name = model.name_of(id);
    let accent = theme::accent(theme::State::Error);
    let mut answered = None;
    egui::Frame::none()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0, accent))
        .rounding(theme::RADIUS)
        .inner_margin(egui::Margin::symmetric(theme::space::S, theme::space::XS))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(HEADING).color(accent));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(HINT).small().color(theme::INK_WEAK));
                });
            });
            ui.add(egui::Label::new(egui::RichText::new(&name).color(theme::INK)).wrap());
            ui.horizontal(|ui| {
                // **`keep` posts nothing**, so it wears no `act:` tag.
                if theme::paint::tab(ui, KEEP, theme::INK_WEAK, KEEP_HELP).clicked() {
                    answered = Some(false);
                }
                let delete = theme::paint::tab(ui, DELETE, accent, DELETE_HELP);
                crate::ui::act::tag(&delete, &[crate::verbs::DELETE_AGENT.word]);
                if delete.clicked() {
                    answered = Some(true);
                }
            });
        });
    match answered {
        Some(true) => model.post_delete(id),
        Some(false) => model.spare(),
        None => {}
    }
}

/// **Its keys**: Enter deletes and Esc keeps, each taken out of the frame's
/// input so the box under it reads neither — a question standing on the
/// prompt is answered before the prompt is.
pub fn keys(ctx: &egui::Context, model: &mut Model) {
    let Some(id) = model.deleting.clone() else {
        return;
    };
    let took = |key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
    if took(egui::Key::Enter) {
        model.post_delete(&id);
    } else if took(egui::Key::Escape) {
        model.spare();
    }
}
