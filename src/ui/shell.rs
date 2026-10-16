//! **The layout**: one column (DESIGN §4.42, §4.45).
//!
//! The transcript fills the window; **directly under it the prompt line**
//! ([`crate::ui::composer`]), and under that every control of the chat — the
//! live dock while anything runs ([`crate::ui::dock`]) and, at the window's
//! bottom edge, the status rule ([`crate::ui::status`]). That is the
//! baseline's column with one order reversed (§4.45 item 3; operator ruling
//! 2026-10-14: *flip all the chat controls to under the conversation box, so
//! the box is right next to the chat*): the words an operator types stand
//! beside the words they answer, with no line of facts between. The ask
//! ([`crate::ui::ask`]) is the one thing that stands between them, while the
//! selected conversation asks: it is the model's question and not a control
//! of the chat, and the box under it is where its answer is typed. Nothing
//! else is on the glass until it is asked for — by a slash verb
//! ([`crate::ui::slash`]) or Ctrl+X ([`crate::ui::switcher`]) — and what is
//! asked for floats in the overlay's frame ([`crate::ui::overlay`]) over the
//! transcript. There is no side column, no column bar, no narrow and broad
//! shape and no dragged edge but the bar's: a width buys wrapping and nothing
//! else, and the phone-shaped viewport is the same window narrower.
//!
//! **The Tab order is the laying order**, which for bottom panels is the
//! stacking order read upward: the bar, the rule's selectors and name, the
//! dock, the box, the ask, then the column. So a Shift+Tab off the box lands
//! on the rule — its name first, then the selectors right to left — and a
//! Tab off it goes on up into the transcript (`crate::ui::keys`).
//!
//! **The composer and the rule stand down under an overlay and under the
//! enrollment**, the composer for the reason it always did — the conversation
//! the box deposits into is not what the operator is looking at — and the
//! rule because every pane is built to fit the narrowest shape the layout
//! promises (§4.32) and the rule's line is what the tallest of them needs
//! there; the overlay's name line says where you are meanwhile. Nothing live
//! paints under the enrollment (bl-7574): it covers the whole window, the one
//! pane that does, because what it holds must not be legible beside anything.

use crate::ui::{
    Model, ask, bar, chat, composer, dock, enroll, keys, overlay, slash, status, tabs, theme,
};

/// Paint one frame of the whole window.
///
/// It takes the context rather than an `eframe::Frame`, which is what makes the
/// window testable at all: every assertion in this crate runs this function on
/// an offscreen context and reads back the glyphs it painted
/// (`crate::paint_probe`). The native boot is `src/main.rs`, which decides
/// nothing.
pub fn render(ctx: &egui::Context, model: &mut Model) {
    theme::install(ctx, &model.settings);
    // **The keys come first**, so what one changed is what this frame paints.
    // Nothing here is a control of its own: every binding calls the door the
    // click beneath it calls (`crate::ui::keys`).
    keys::handle(ctx, model);
    // **The bar is laid first** (DESIGN §4.44), so it stands the window's
    // full height and the column fits beside it at every size.
    bar::render(ctx, model);
    // **Bottom panels stack upward in the order they are laid** (§4.45 item
    // 3): the rule takes the window's bottom edge, the dock stands on it
    // while anything runs, the box on those, and the ask on the box.
    let rule = (!model.covered()).then(|| {
        let rule = egui::TopBottomPanel::bottom("rule")
            .show(ctx, |ui| status::render(ui, model))
            .response
            .rect;
        // **The dock stands between the rule and the box while anything
        // runs** (§4.42, item 9): one line, or a row apiece when unfolded.
        let running = dock::running(model);
        if !running.is_empty() {
            egui::TopBottomPanel::bottom("dock")
                .frame(
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(theme::space::L, theme::space::XXS)),
                )
                .show(ctx, |ui| dock::render(ui, model, &running));
        }
        egui::TopBottomPanel::bottom("composer").show(ctx, |ui| composer::render(ui, model));
        // **A deletion's confirmation stands on the box** (§4.44 item 1;
        // bl-e837), where the ask stands: the question a trashcan asked.
        if let Some(id) = model.deleting.clone() {
            egui::TopBottomPanel::bottom("confirm")
                .frame(
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(theme::space::L, theme::space::XS)),
                )
                .show(ctx, |ui| bar::confirm::render(ui, model, &id));
        }
        // **The ask stands above the box while the selected conversation
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
        rule
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        if enroll::render(ui, model) {
            return;
        }
        // **The tabs stand over the transcript** (§4.44, item 2).
        tabs::render(ui, model);
        // **The seat panel's waiting count is the queue's door** (§4.42,
        // item 11), as the rule's was.
        if chat::render(ui, model) {
            slash::fire(model, crate::ui::queue::WORD, "");
        }
        // **The menu floats over the box it completes** (§4.42, item 5): laid
        // last in the column, at its bottom, which is where the box begins.
        slash::menu(ui, model);
        // **A selector's dropup floats over its segment** (§4.42, item 11),
        // which is under the box now: its foot stands on the rule's top edge,
        // on a layer of its own (§4.45, item 3).
        let foot = rule.map_or(ui.max_rect().bottom(), |rule| rule.top());
        status::dropup::render(ui, model, foot);
        // **Every other pane floats** (§4.42, item 6): painted last, over the
        // conversation it used to replace.
        overlay::stand(ui, model);
    });
}

#[cfg(test)]
mod tests;
