//! **The overlay**: the one frame every pane that used to cover the
//! conversation now floats in (DESIGN §4.42, item 6; `docs/STYLE.md` §5, *an
//! overlay*) — every pane but the three the advanced mode docks beside the
//! column instead (`crate::ui::rail`; §4.48 item 8).
//!
//! A pane keeps its content and loses its own chrome: the box, its name line
//! and the hint line under it are painted here, once, and the pane paints
//! only what it is about. Esc closes whichever is standing
//! ([`Model::escape`]), and the name line's other end is that key as a door
//! ([`way_out`]; bl-a4bd): one control the frame paints once, where a hand
//! on the pointer reaches it — not the fifteen `done` controls this
//! replaced, which were fifteen spellings of one key.
//!
//! **It floats over the transcript and stands above the composer's place.**
//! The composer stands down while a pane is standing, exactly as it did under
//! a covering pane (`crate::ui::shell`): the conversation the box deposits
//! into is not what the operator is looking at.
//!
//! **The frame is the one new shape**, and `docs/STYLE.md` §1's *no outlines*
//! is amended by exactly it: a stroke on a thing that floats says what is
//! over what.

use crate::ui::{Model, theme};

/// **The way out's word**: the key's own name, on the door that is the key.
pub const HINT: &str = "ESC";
/// What the way out does.
pub const OUT_HELP: &str = "closes this — the same as pressing Esc";
/// **The way out was clicked**: raised by [`way_out`], taken down by the one
/// reader of the Escape key (`crate::ui::keys`), so a click and the key are
/// one act with one order rather than two.
pub const OUT: &str = "the overlay's way out was clicked";
/// **Where the box clips**, written to the toolkit's memory each frame it
/// stands, for the one reader that has to know: the snapshot matrix judges
/// a control laid past the window as a layout fault, and a control scrolled
/// past this rect is reachable by scrolling, not lost
/// (`crate::snapshot::clipped`). A view fact, held where view facts are.
pub const BOX: &str = "the overlay's box";
/// **The pane's first control takes the keyboard** (bl-99ed; DESIGN §4.46
/// item 3): the flag [`stand`] raises just before a pane's body paints,
/// while the pane stands and nothing holds the keyboard, and the first
/// painter to end in `theme::paint::told` takes down — so Tab walks the
/// pane before the bar, which is painted first and would otherwise lead.
/// The twin of the prompt line taking the caret back at rest
/// (`crate::ui::keys::boxes::rest`): where the keyboard goes when nothing
/// holds it is decided by what stands. **The switcher is excepted**: its
/// keys are the walk, and a focused row would be fired by the Enter that
/// chooses the walked one.
pub const FIRST: &str = "the overlay's first control takes the keyboard";
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

/// **The way out, as a door** (bl-a4bd): [`HINT`] in weak ink at the small
/// size, and a click on it is the Escape key — it raises [`OUT`] and the
/// key's own reader spends it.
///
/// **It never takes the keyboard**: it is a key already, so it is no stop in
/// the Tab order (`theme::paint::pointer_link`), and the [`FIRST`] flag is
/// held across it — it is laid before the pane's own controls, and a pane
/// that opened with the caret on its way out would close on the first Enter.
pub fn way_out(ui: &mut egui::Ui) {
    let first = egui::Id::new(FIRST);
    let held = ui.ctx().data_mut(|d| d.remove_temp::<bool>(first));
    if theme::paint::pointer_link(ui, HINT, theme::INK_WEAK, OUT_HELP).clicked() {
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::new(OUT), true));
        ui.ctx().request_repaint();
    }
    if let Some(held) = held {
        ui.ctx().data_mut(|d| d.insert_temp(first, held));
    }
}

/// **The name line with what the pane is open ON beside it**, one step
/// weaker — the subject on the name's line rather than under it, for the
/// same reason the hint is.
///
/// **The hint takes the right end first, and the subject is cut to what is
/// left** (§4.48 item 8): on a rail a conversation's id is wider than the
/// line, and a subject laid before the hint ran under it.
pub fn about(ui: &mut egui::Ui, word: &str, subject: Option<&str>) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        way_out(ui);
        // One row tall, laid left to right by name: a child in a
        // right-to-left parent would otherwise claim the pane's height.
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), theme::ROW),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.label(egui::RichText::new(word).color(theme::INK));
                if let Some(subject) = subject {
                    let subject = egui::RichText::new(subject).color(theme::INK_WEAK);
                    ui.add(egui::Label::new(subject).truncate());
                }
            },
        );
    });
    ui.add_space(theme::space::XS);
}

/// **Paint whichever pane is standing, in the frame**, and nothing where none
/// is. Tried in the order the shell always tried them, which is an order
/// rather than a rule with an exception: no two stand together
/// (`Model::covered` is one question), so the order is never spent.
pub fn stand(ui: &mut egui::Ui, model: &mut Model) {
    if !model.covered() {
        return;
    }
    let first = egui::Id::new(FIRST);
    let wants = !model.switching && ui.ctx().memory(egui::Memory::focused).is_none();
    ui.ctx().data_mut(|d| d.insert_temp(first, wants));
    float(ui, "the overlay", ceiling(ui), |ui| {
        use crate::ui::{
            board, clear, commands, find, fleet, prices, queue, records, settings, switcher, trail,
            unmake,
        };
        // **The settings overlay answers for seven of the panes** (§4.48
        // item 7): the tuning, the login, the config, the clients, the
        // doctor and the knob table are pages of it, and it paints whichever
        // stands under its tree.
        // **The three re-homed panes float only while they are not docked**
        // (§4.48 item 8; `crate::ui::rail` paints them otherwise) — the
        // config pane through the settings overlay, which answers for it
        // only then (`Model::page`).
        let floating = !model.docked();
        let _ = switcher::render(ui, model)
            || settings::render(ui, model)
            || (floating && records::render(ui, model))
            || queue::render(ui, model)
            || (floating && (trail::render(ui, model) || clear::render(ui, model)))
            || board::render(ui, model)
            || fleet::render(ui, model)
            || commands::render(ui, model)
            || find::render(ui, model)
            || prices::render(ui, model)
            || unmake::render(ui, model);
    });
    // A pane with no control leaves the flag standing; nothing after the
    // overlay may take it.
    ui.ctx().data_mut(|d| d.remove_temp::<bool>(first));
}

#[cfg(test)]
mod tests;
