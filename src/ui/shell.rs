//! **The layout**, and the notice that stands where content would have been.
//!
//! One function, and it is the whole window: a notice bar, the roster, the
//! conversation list, and the conversation with its composer under it. There is
//! no per-pane enablement to drift — a pane with nothing to show says so in its
//! own words, which is a sentence an operator can act on rather than a control
//! that only looks actionable.

use crate::ui::{Model, chat, composer, convs, keys, roster, theme};

/// Paint one frame of the whole window.
///
/// It takes the context rather than an `eframe::Frame`, which is what makes the
/// window testable at all: every assertion in this crate runs this function on
/// an offscreen context and reads back the glyphs it painted
/// (`crate::paint_probe`). The native boot is `src/main.rs`, which decides
/// nothing.
pub fn render(ctx: &egui::Context, model: &mut Model) {
    ctx.set_visuals(theme::visuals());
    // **The keys come first**, so what one changed is what this frame paints.
    // Nothing here is a control of its own: every binding calls the door the
    // click beneath it calls (`crate::ui::keys`).
    keys::handle(ctx, model);
    notice(ctx, model);
    egui::SidePanel::left("roster")
        .default_width(280.0)
        .show(ctx, |ui| roster::render(ui, model));
    egui::SidePanel::left("conversations")
        .default_width(320.0)
        .show(ctx, |ui| convs::render(ui, model));
    egui::TopBottomPanel::bottom("composer").show(ctx, |ui| composer::render(ui, model));
    egui::CentralPanel::default().show(ctx, |ui| chat::render(ui, model));
}

/// The notice bar: the last thing the seat heard that was not content, in the
/// words of whoever said it, and dismissible.
///
/// It is a **bar rather than a modal** because a refusal about one pane must not
/// stop the operator reading the other three: the engine refusing a deposit says
/// nothing about the roster beside it.
fn notice(ctx: &egui::Context, model: &mut Model) {
    let Some(notice) = model.notice.clone() else {
        return;
    };
    egui::TopBottomPanel::top("notice").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button(DISMISS).clicked() {
                model.dismiss();
            }
            ui.colored_label(theme::NOTICE, notice.line());
        });
    });
}

/// The word that puts a notice down. An operator who has read it should not
/// have to wait for the next answer to clear it.
pub const DISMISS: &str = "×";

#[cfg(test)]
mod tests;
