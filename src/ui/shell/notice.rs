//! **The notice bar**: the last thing the seat heard that was not content, in
//! the words of whoever said it, and dismissible.
//!
//! Split from [`super`] on the seam that module's own doc draws: the layout is
//! the two shapes a window takes, and this is the one strip that stands where
//! content would have been. It is a **bar rather than a modal** because a
//! refusal about one pane must not stop the operator reading the other three:
//! the engine refusing a deposit says nothing about the roster beside it.

use crate::ui::{Model, theme};

/// **The ink a notice is said in** — the state's, on the six-colour ruling:
/// a failure of any of the five kinds is an error that will not mend itself
/// until somebody acts, and a receipt is a note wanting salience.
fn notice_ink(notice: &crate::ui::Notice) -> egui::Color32 {
    match notice {
        crate::ui::Notice::Said(_) => theme::accent(theme::State::Annotation),
        _ => theme::accent(theme::State::Error),
    }
}

/// Paint it, or nothing at all when the seat has heard nothing that was not
/// content.
pub(super) fn render(ctx: &egui::Context, model: &mut Model) {
    let Some(notice) = model.notice.clone() else {
        return;
    };
    egui::TopBottomPanel::top("notice").show(ctx, |ui| {
        ui.horizontal_top(|ui| {
            if ui.button(DISMISS).clicked() {
                model.dismiss();
            }
            // **The sentence WRAPS** (bl-3d0f). A horizontal layout lays its
            // labels on one line however long they are, and the panel cuts what
            // reaches the frame — with no ellipsis, because the galley was
            // never truncated and so never had one added. Every refusal this
            // seat paints puts the fact first and the remedy last, so the half
            // that was cut was always the half that says what to do; the first
            // run of a seat on an unprovisioned box loses the whole of the one
            // instruction on the window. A second line in a bar already sized
            // to its content costs nothing that matters.
            ui.add(
                egui::Label::new(egui::RichText::new(notice.line()).color(notice_ink(&notice)))
                    .wrap(),
            );
        });
    });
}

/// The word that puts a notice down. An operator who has read it should not
/// have to wait for the next answer to clear it.
pub const DISMISS: &str = "×";
