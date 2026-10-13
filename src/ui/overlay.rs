//! **The overlay**: the one frame every pane that used to cover the
//! conversation now floats in (DESIGN §4.42, item 6; `docs/STYLE.md` §5, *an
//! overlay*).
//!
//! A pane keeps its content and loses its own chrome: the box, its name line
//! and the hint line under it are painted here, once, and the pane paints
//! only what it is about. Esc closes whichever is standing
//! ([`Model::escape`]), and that is said on the glass by the hint rather than
//! by a control — the fifteen `done` controls this replaces were fifteen
//! spellings of one key.
//!
//! **It floats over the transcript and stands above the composer's place.**
//! The composer stands down while a pane is standing, exactly as it did under
//! a covering pane (`crate::ui::shell`): the conversation the box deposits
//! into is not what the operator is looking at.
//!
//! **The frame is the one new shape**, and `docs/STYLE.md` §1's *no outlines*
//! is amended by exactly it: a stroke on a thing that floats says what is
//! over what. The enrollment is not here — it keeps §4.15's whole-window
//! cover, because what it holds must not be legible beside anything.

use crate::ui::{Model, theme};

/// What every overlay says under its rows.
pub const HINT: &str = "Esc closes";
/// **Where the box clips**, written to the toolkit's memory each frame it
/// stands, for the one reader that has to know: the snapshot matrix judges
/// a control laid past the window as a layout fault, and a control scrolled
/// past this rect is reachable by scrolling, not lost
/// (`crate::snapshot::clipped`). A view fact, held where view facts are.
pub const BOX: &str = "the overlay's box";
/// **What a box's height may be at most**: the pane it floats over, less a
/// margin at the top that keeps the conversation's own head on the glass.
///
/// All of it rather than a share, because every pane is built to fit the
/// narrowest shape the layout promises (DESIGN §4.32's density constraint)
/// and a box that took a share of that would be a pane that fits nowhere.
pub fn ceiling(ui: &egui::Ui) -> f32 {
    (ui.max_rect().height() - theme::space::XS).max(0.0)
}

/// **A framed box over the bottom of the pane this is called in**, `height`
/// tall and as wide as the pane minus a margin each side.
///
/// It is a child region of the pane rather than an `egui::Area`, and that is
/// a decision about what can be proven: a fresh area paints nothing on its
/// first frame (its sizing pass), so every single-frame assertion in this
/// crate would read a blank where the box stands. A region at a rect that is
/// known before anything is laid has no such frame, and because it is laid
/// after the conversation its glyphs and its controls stand over it for the
/// paint walk and the pointer alike.
pub fn float(ui: &mut egui::Ui, name: &'static str, height: f32, body: impl FnOnce(&mut egui::Ui)) {
    let room = ui.max_rect();
    let span = egui::Rangef::new(
        room.left() + theme::space::L,
        room.right() - theme::space::L,
    );
    float_at(ui, name, span, height, body);
}

/// **The same box over `span` of the pane's width** — a rule's dropup
/// (`crate::ui::status::dropup`), which stands over its own segment rather
/// than across the pane. Answers where it stood.
pub fn float_at(
    ui: &mut egui::Ui,
    name: &'static str,
    span: egui::Rangef,
    height: f32,
    body: impl FnOnce(&mut egui::Ui),
) -> egui::Rect {
    let room = ui.max_rect();
    let height = height.min(ceiling(ui));
    let rect = egui::Rect::from_min_max(
        egui::pos2(span.min, room.bottom() - height),
        egui::pos2(span.max, room.bottom()),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min))
            .id_salt(name),
    );
    egui::Frame::none()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0, theme::HAIRLINE))
        .rounding(theme::RADIUS)
        .inner_margin(egui::Margin::symmetric(theme::space::S, theme::space::XS))
        .show(&mut child, |ui| {
            let inner = rect.size() - egui::vec2(2.0 * theme::space::S, 2.0 * theme::space::XS);
            ui.set_min_size(inner);
            ui.set_max_height(inner.y);
            ui.data_mut(|d| d.insert_temp(egui::Id::new(BOX), ui.max_rect()));
            // **What does not fit is scrolled** (§4.11's rule, kept): the box
            // is a fixed share of the pane, and a pane longer than it scrolls
            // inside rather than laying controls past the window's edge.
            egui::ScrollArea::vertical()
                .id_salt(name)
                .auto_shrink(false)
                .show(ui, body);
        });
    rect
}

/// **The name a pane paints at its head**, in body ink, with the way out said
/// at the line's other end in weak ink. The one line the frame leaves to the
/// pane, because the pane is what knows its name — and the hint rides it
/// rather than taking a line of its own, because every pane is built to fit
/// the narrowest shape the layout promises and a line is what that costs.
pub fn name(ui: &mut egui::Ui, word: &str) {
    about(ui, word, None);
}

/// **The name line with what the pane is open ON beside it**, one step
/// weaker — the subject on the name's line rather than under it, for the
/// same reason the hint is.
pub fn about(ui: &mut egui::Ui, word: &str, subject: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(word).color(theme::INK));
        if let Some(subject) = subject {
            ui.label(egui::RichText::new(subject).color(theme::INK_WEAK));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(HINT).small().color(theme::INK_WEAK));
        });
    });
    ui.add_space(theme::space::XS);
}

/// **Paint whichever pane is standing, in the frame**, and nothing where none
/// is. Tried in the order the shell always tried them, which is an order
/// rather than a rule with an exception: no two stand together
/// (`Model::covered` is one question), so the order is never spent.
pub fn stand(ui: &mut egui::Ui, model: &mut Model) {
    if !model.covered() || model.enroll.is_some() {
        return;
    }
    float(ui, "the overlay", ceiling(ui), |ui| {
        use crate::ui::{
            board, clear, clients, commands, config, doctor, find, fleet, login, prices, queue,
            records, switcher, trail, tuning, unmake,
        };
        let _ = switcher::render(ui, model)
            || tuning::render(ui, model)
            || records::render(ui, model)
            || queue::render(ui, model)
            || trail::render(ui, model)
            || clear::render(ui, model)
            || board::render(ui, model)
            || fleet::render(ui, model)
            || commands::render(ui, model)
            || doctor::render(ui, model)
            || find::render(ui, model)
            || prices::render(ui, model)
            || login::render(ui, model)
            || clients::render(ui, model)
            || config::render(ui, model)
            || unmake::render(ui, model);
    });
}

#[cfg(test)]
mod tests;
