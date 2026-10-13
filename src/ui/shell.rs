//! **The layout**: one column (DESIGN §4.42).
//!
//! The transcript fills the window; under it the status rule
//! ([`crate::ui::status`]), and under that the prompt line
//! ([`crate::ui::composer`]). Nothing else is on the glass until it is asked
//! for — by a slash verb ([`crate::ui::slash`]) or Ctrl+X
//! ([`crate::ui::switcher`]) — and what is asked for floats in the overlay's
//! frame ([`crate::ui::overlay`]) over the transcript. There is no side
//! column, no column bar, no narrow and broad shape and no dragged edge: a
//! width buys wrapping and nothing else, and the phone-shaped viewport is the
//! same window narrower.
//!
//! **The composer and the rule stand down under an overlay and under the
//! enrollment**, the composer for the reason it always did — the conversation
//! the box deposits into is not what the operator is looking at — and the
//! rule because every pane is built to fit the narrowest shape the layout
//! promises (§4.32) and the rule's line is what the tallest of them needs
//! there; the overlay's name line says where you are meanwhile. Nothing live
//! paints under the enrollment (bl-7574): it covers the whole window, the one
//! pane that does, because what it holds must not be legible beside anything.

use crate::ui::{Model, ask, chat, composer, dock, enroll, keys, overlay, slash, status, theme};

/// Paint one frame of the whole window.
///
/// It takes the context rather than an `eframe::Frame`, which is what makes the
/// window testable at all: every assertion in this crate runs this function on
/// an offscreen context and reads back the glyphs it painted
/// (`crate::paint_probe`). The native boot is `src/main.rs`, which decides
/// nothing.
pub fn render(ctx: &egui::Context, model: &mut Model) {
    theme::install(ctx);
    // **The keys come first**, so what one changed is what this frame paints.
    // Nothing here is a control of its own: every binding calls the door the
    // click beneath it calls (`crate::ui::keys`).
    keys::handle(ctx, model);
    // **Bottom panels stack upward in the order they are laid**: the prompt
    // takes the bottom edge, and the rule stands on it — which is the
    // baseline's order, the rule between the transcript and the prompt.
    if !model.covered() {
        egui::TopBottomPanel::bottom("composer").show(ctx, |ui| composer::render(ui, model));
        egui::TopBottomPanel::bottom("rule").show(ctx, |ui| status::render(ui, model));
        // **The dock stands above the rule while anything runs** (§4.42,
        // item 9): one line, or a row apiece when unfolded.
        let running = dock::running(model);
        if !running.is_empty() {
            egui::TopBottomPanel::bottom("dock")
                .frame(
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(theme::space::L, theme::space::XXS)),
                )
                .show(ctx, |ui| dock::render(ui, model, &running));
        }
        // **The ask stands above the rule while the selected conversation
        // asks** (§4.42, item 8): the baseline's approval panel, between the
        // transcript and the line the answer is typed on.
        if let Some(row) = ask::row(model) {
            egui::TopBottomPanel::bottom("ask")
                .frame(
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(theme::space::L, theme::space::XS)),
                )
                .show(ctx, |ui| ask::render(ui, model, &row));
        }
    }
    egui::CentralPanel::default().show(ctx, |ui| {
        if enroll::render(ui, model) {
            return;
        }
        chat::render(ui, model);
        // **The menu floats over the box it completes** (§4.42, item 5): laid
        // last in the column, at its bottom, which is where the rule begins.
        slash::menu(ui, model);
        // **Every other pane floats** (§4.42, item 6): painted last, over the
        // conversation it used to replace.
        overlay::stand(ui, model);
    });
}

#[cfg(test)]
mod tests;
