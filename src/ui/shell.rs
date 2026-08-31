//! **The layout**, and the notice that stands where content would have been.
//!
//! One function, and it is the whole window: a notice bar, the roster, the
//! conversation list, and the conversation with its composer under it. There is
//! no per-pane enablement to drift — a pane with nothing to show says so in its
//! own words, which is a sentence an operator can act on rather than a control
//! that only looks actionable.

use crate::ui::{Model, chat, composer, convs, enroll, keys, roster, theme};

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
    let (roster_width, convs_width) = widths(ctx.screen_rect().width());
    egui::SidePanel::left("roster")
        .default_width(ROSTER)
        .max_width(roster_width)
        .show(ctx, |ui| roster::render(ui, model));
    egui::SidePanel::left("conversations")
        .default_width(CONVS)
        .max_width(convs_width)
        .show(ctx, |ui| convs::render(ui, model));
    // **Nothing live paints under the enrollment** (bl-7574). The composer is a
    // bottom panel, so it is outside what the central panel below covers — and
    // what stood there was a live `start` control firing a conversation on the
    // very wall being enrolled into, from under the symbol.
    if model.enroll.is_none() {
        egui::TopBottomPanel::bottom("composer").show(ctx, |ui| composer::render(ui, model));
    }
    // **The enrollment stands where the conversation would**, and it is the one
    // pane in this window that covers another. It earns that: what it holds is
    // a private key on a screen, the act is "look at this now and close it",
    // and a conversation legible behind it would invite the one thing the
    // material must not have, which is a long life on a display
    // (`crate::ui::enroll`).
    egui::CentralPanel::default().show(ctx, |ui| {
        if !enroll::render(ui, model) {
            chat::render(ui, model);
        }
    });
}

/// **What the two list panes are worth when the window is wide enough**, in
/// points: the roster holds a handful of short words, the conversation list
/// holds a headline and a preview under it.
const ROSTER: f32 = 280.0;
const CONVS: f32 = 320.0;

/// **The floor the conversation and its composer keep.** Below this a chat pane
/// is a strip: a message elides inside its own width, the composer's box shows
/// the first few words of a draft, and `send` sits against the frame.
pub const CHAT_FLOOR: f32 = 420.0;

/// **The width a list pane never goes under**, however narrow the window. A
/// pane below it shows nothing at all, which is worse than a chat pane under
/// its floor — so this is the one thing the floor yields to.
pub const SIDE_FLOOR: f32 = 140.0;

/// **The two list panes' widths at a given window width** — the policy the
/// window had none of (bl-e5d2).
///
/// The side panels used to keep their widths as the window narrowed and the
/// central panel absorbed the whole loss, so at 900 points the pane the window
/// exists for was a ~140-point strip while the roster kept 280. The rule is the
/// other way round: **the conversation has a floor and the list panes yield to
/// it**, together and in proportion to what each is worth, until they reach
/// their own floor. Past that nothing yields, because two panes showing nothing
/// buys the chat pane a width it still cannot use.
///
/// It is a pure function of one number, so the policy is a value a test reads
/// back rather than a layout somebody has to look at.
pub fn widths(window: f32) -> (f32, f32) {
    let share = ((window - CHAT_FLOOR) / (ROSTER + CONVS)).clamp(0.0, 1.0);
    (
        (ROSTER * share).max(SIDE_FLOOR),
        (CONVS * share).max(SIDE_FLOOR),
    )
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
