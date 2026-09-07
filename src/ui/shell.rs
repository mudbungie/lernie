//! **The layout** — the two shapes a window takes, and the notice that stands
//! where content would have been.
//!
//! One function, and it is the whole window: a notice bar, the roster, the
//! conversation list, and the conversation with its composer under it. There is
//! no per-pane enablement to drift — a pane with nothing to show says so in its
//! own words, which is a sentence an operator can act on rather than a control
//! that only looks actionable.
//!
//! **What a width buys is [`policy`]'s** and nothing here decides it. Wide
//! enough, and the three columns stand side by side with the two lists yielding
//! to the conversation's floor; narrower than that, the window shows one
//! [`Column`] at a time and a bar naming the three (bl-dfda). The panes
//! themselves know nothing about either shape: what changed is where they are
//! put, and — because a column's name has one home — where their heading is
//! painted.

use crate::ui::{
    Model, board, chat, clear, clients, commands, composer, config, convs, enroll, find, fleet,
    keys, login, queue, records, roster, theme, trail, tuning, unmake,
};

/// The width policy: the yield, the two shapes, and the three columns.
pub mod policy;

pub use policy::{CHAT_FLOOR, Column, SIDE_FLOOR, Shape, shape, widths};

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
    notice(ctx, model);
    // **The shown column is the shape's answer, not the model's**: in the broad
    // shape every column is on the glass, so the central panel is the
    // conversation's and the model's own column is not consulted at all.
    let (shown, broad) = match shape(ctx.screen_rect().width()) {
        Shape::Broad { roster, convs } => {
            lists(ctx, model, roster, convs);
            (Column::Conversation, true)
        }
        Shape::Narrow => {
            bar(ctx, model);
            (model.column, false)
        }
    };
    // **Nothing live paints under the enrollment** (bl-7574). The composer is a
    // bottom panel, so it is outside what the central panel below covers — and
    // what stood there was a live `start` control firing a conversation on the
    // very wall being enrolled into, from under the symbol.
    //
    // **The tuning pane stands the composer down for the same reason it stands
    // down under the enrollment** (bl-4a2c), one noun over: the conversation
    // the composer deposits into is not on the glass while a pane covers it,
    // and a send box whose subject an operator cannot see is a control aimed at
    // something they are not looking at.
    // **The records pane stands it down too** (bl-2cf7), the decision queue
    // with it (bl-f0ef), and the window's own two panes with those (bl-40ec),
    // for the same reason one noun over: the conversation the composer deposits
    // into is not on the glass while any pane covers it.
    //
    // **And the narrow shape stands it down off every other column**, which is
    // that rule with the word *covers* read literally: the composer acts on the
    // selected conversation, and in the narrow shape the conversation is on the
    // glass only when its own column is.
    if !model.covered() && shown == Column::Conversation {
        egui::TopBottomPanel::bottom("composer").show(ctx, |ui| composer::render(ui, model));
    }
    egui::CentralPanel::default().show(ctx, |ui| central(ui, model, shown, broad));
}

/// **The two list panes, side by side with the conversation** — the broad
/// shape, and the only one that has side panels at all.
///
/// **The heading is painted here rather than by the pane** (bl-dfda). A
/// column's name has one home, and in the narrow shape that home is the bar:
/// two nodes carrying the word `channels` would be two things an operator — and
/// the accessibility tree the snapshot harness walks — has to tell apart. So
/// the pane paints its content and the layout paints its name, which also keeps
/// bl-e5d2's rule structural: the heading is outside the pane, and therefore
/// outside the region the pane scrolls.
///
/// **The width is [`policy`]'s, EXACTLY, and a panel asked for a default plus a
/// cap does not obey one** (bl-fef8). egui stores a side panel's state as the
/// rect its CONTENT took and reads that back as the width on the next pass, so
/// a pane narrower than its cap shrank to its content and could never grow
/// again: a window opened at 800 points and then widened to 1440 kept a
/// 175-point roster forever, every row in it wrapped after two words, because
/// nothing in the loop consulted the policy a second time. `exact_width` makes
/// the policy the whole of the answer — the stored rect is clamped to a point
/// range, so what a pane took last frame cannot outvote what this window is
/// worth. It costs the drag handle, and that is subtraction rather than loss: a
/// dragged width is a second home for a fact the policy already owns, and the
/// drag did not work anyway — a pane pulled wider than its content snapped back
/// to the content on the next pass.
fn lists(ctx: &egui::Context, model: &mut Model, roster_width: f32, convs_width: f32) {
    egui::SidePanel::left("roster")
        .resizable(false)
        .exact_width(roster_width)
        .show(ctx, |ui| {
            heading(ui, roster::HEADING, model.focus == keys::Pane::Roster);
            roster::render(ui, model);
        });
    egui::SidePanel::left("conversations")
        .resizable(false)
        .exact_width(convs_width)
        .show(ctx, |ui| {
            heading(ui, convs::HEADING, model.focus == keys::Pane::Conversations);
            convs::render(ui, model);
        });
}

/// **A column's heading**: `HEADING` size, weak ink at rest, full ink with
/// the brand mark when the arrows are its (`docs/STYLE.md` §5). No fill and
/// no rule under it — the gap is the boundary.
fn heading(ui: &mut egui::Ui, word: &str, focused: bool) {
    let ink = if focused { theme::INK } else { theme::INK_WEAK };
    ui.label(
        egui::RichText::new(keys::heading(word, focused))
            .heading()
            .color(ink),
    );
}

/// **The ink a notice is said in** — the state's, on the six-colour ruling:
/// a failure of any of the five kinds is an error that will not mend itself
/// until somebody acts, and a receipt is a note wanting salience.
fn notice_ink(notice: &crate::ui::Notice) -> egui::Color32 {
    match notice {
        crate::ui::Notice::Said(_) => theme::accent(theme::State::Annotation),
        _ => theme::accent(theme::State::Error),
    }
}

/// **The narrow shape's navigation**: the three columns' own names, with the
/// one on the glass showing as chosen.
///
/// It is a bar of all three rather than a *back* control, because a stack would
/// make the roster two gestures from the conversation and put the number of
/// gestures a pane costs at the mercy of where the operator happened to be. One
/// seat each is one gesture from anywhere to anywhere, which is the bound
/// `crate::snapshot::reach` asserts.
///
/// **It stands down under a covering pane**, exactly as the composer does: a
/// pane that covers the window is a modal, and a navigation control that
/// changed a column nobody can see would be a control that answers a click with
/// nothing. The way out of a covering pane is its own control, which is the
/// one that forgets where the material is a secret.
fn bar(ctx: &egui::Context, model: &mut Model) {
    if model.covered() {
        return;
    }
    egui::TopBottomPanel::top("columns").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            for column in Column::all() {
                if ui
                    .selectable_label(model.column == column, column.word())
                    .clicked()
                {
                    model.column = column;
                }
            }
        });
    });
}

/// **What stands in the central panel**: a covering pane if one is open, and
/// otherwise the column the shape chose.
///
/// **The enrollment stands where the conversation would**, and it is the one
/// pane in this window that covers another. It earns that: what it holds is a
/// private key on a screen, the act is "look at this now and close it", and a
/// conversation legible behind it would invite the one thing the material must
/// not have, which is a long life on a display (`crate::ui::enroll`).
///
/// **Two panes may stand there and the enrollment wins**, which is an order
/// rather than a rule with an exception: it is the one that holds a secret, and
/// the material's whole product is a short life on a display.
fn central(ui: &mut egui::Ui, model: &mut Model, shown: Column, broad: bool) {
    if enroll::render(ui, model)
        || tuning::render(ui, model)
        || records::render(ui, model)
        || queue::render(ui, model)
        || trail::render(ui, model)
        || clear::render(ui, model)
        || board::render(ui, model)
        || fleet::render(ui, model)
        || commands::render(ui, model)
        || find::render(ui, model)
        || login::render(ui, model)
        || clients::render(ui, model)
        || config::render(ui, model)
        || unmake::render(ui, model)
    {
        return;
    }
    match shown {
        Column::Channels => roster::render(ui, model),
        Column::Conversations => convs::render(ui, model),
        Column::Conversation => {
            // The broad shape has no bar to carry the name, so the conversation
            // pane's heading stands above it here — the two list panes get
            // theirs from `lists` for the same reason.
            if broad {
                heading(ui, chat::HEADING, false);
            }
            chat::render(ui, model);
        }
    }
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

#[cfg(test)]
mod tests;
