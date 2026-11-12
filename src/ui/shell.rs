//! **The layout**: one column (DESIGN §4.42, §4.45).
//!
//! The transcript fills the window; **directly under it the prompt line**
//! ([`crate::ui::composer`]), and under that every control of the chat — the
//! live dock while anything runs ([`crate::ui::dock`]) and, at the window's
//! bottom edge across its whole width, the status bar ([`crate::ui::status`];
//! §4.48 item 1). That is the
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
//! transcript — or, in the advanced mode, stands docked at the right as a
//! rail ([`crate::ui::rail`]; §4.48 item 8), the one query `Model::docked`
//! deciding which. There is no column bar, no narrow and broad shape and no
//! dragged edge but the bar's and the rails': a width buys wrapping and
//! nothing else, and the phone-shaped viewport is the same window narrower,
//! its panes floating whatever the mode.
//!
//! **The Tab order is the laying order**: the status bar's doors, the bar,
//! the dock, the box — its field, its pills, its send door — the ask, then
//! the column. So a Shift+Tab off the box lands on the bar, and a Tab off
//! the send door goes on up into the transcript (`crate::ui::keys`).
//!
//! **The composer and the status bar stand down under an overlay**, the
//! composer for the reason it always did — the
//! conversation the box deposits into is not what the operator is looking at
//! — and the status bar because every pane is built to fit the narrowest
//! shape the layout promises (§4.32) and its line is what the tallest of
//! them needs there; the overlay's name line says where you are meanwhile.

use crate::ui::{
    Model, ask, bar, chat, composer, dock, keys, overlay, rail, slash, status, tabs, theme,
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
    // **The window's room is read first** (§4.48 item 8), so the keys and
    // every door below decide the panes' home off this frame's width.
    model.housed(rail::room(ctx, model));
    // **The keys come first**, so what one changed is what this frame paints.
    // Nothing here is a control of its own: every binding calls the door the
    // click beneath it calls (`crate::ui::keys`).
    keys::handle(ctx, model);
    // **The status bar is laid first** (DESIGN §4.48 items 1 and 4), so it
    // takes the window's bottom edge across its whole width — under the bar
    // as well as the column.
    let covered = model.covered();
    let bar_top = (!covered).then(|| {
        egui::TopBottomPanel::bottom("status")
            .show(ctx, |ui| status::render(ui, model))
            .response
            .rect
            .top()
    });
    // **The bar is laid next** (DESIGN §4.44), so it stands the window's
    // height above the status bar and the column fits beside it.
    bar::render(ctx, model);
    // **The rails are laid next** (§4.48 item 8), so they stand the window's
    // height above the status bar and the box stays the column's.
    rail::render(ctx, model);
    // **Bottom panels stack upward in the order they are laid** (§4.45 item
    // 3): the dock stands on the status bar while anything runs, the box on
    // those, and the ask on the box.
    let box_top = (!covered).then(|| {
        // **The dock stands between the status bar and the box while
        // anything runs** (§4.42, item 9): one line, or a row apiece when
        // unfolded.
        let running = dock::running(model);
        if !running.is_empty() {
            egui::TopBottomPanel::bottom("dock")
                .frame(
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(theme::space::L, theme::space::XXS)),
                )
                .show(ctx, |ui| dock::render(ui, model, &running));
        }
        let top = egui::TopBottomPanel::bottom("composer")
            .show(ctx, |ui| composer::render(ui, model))
            .response
            .rect
            .top();
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
        top
    });
    egui::CentralPanel::default().show(ctx, |ui| {
        // **The head stands over the transcript** (§4.44, item 2): the mark
        // at the column's top right, the tabs wrapping to its left.
        tabs::render(ui, model);
        chat::render(ui, model);
        // **The menu floats over the box it completes** (§4.42, item 5): laid
        // last in the column, at its bottom, which is where the box begins.
        slash::menu(ui, model);
        // **A selector's dropup opens upward from where it stands** (§4.48
        // item 4): a pill's from the box's top edge, over the transcript's
        // foot; the status bar's from the bar's top edge, over the box — on
        // a layer of its own (§4.45, item 3).
        let boxed = model
            .dropup
            .as_ref()
            .is_some_and(|open| status::selectors::boxed(open.selector));
        let edge = if boxed { box_top } else { bar_top };
        status::dropup::render(ui, model, edge.unwrap_or(ui.max_rect().bottom()));
        // **Every other pane floats** (§4.42, item 6): painted last, over the
        // conversation it used to replace.
        overlay::stand(ui, model);
    });
}

#[cfg(test)]
mod tests;
