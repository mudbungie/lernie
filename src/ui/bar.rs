//! **The bar** (DESIGN §4.44 item 1, §4.45 items 1–2; `docs/STYLE.md` §2):
//! the left panel listing the aimed workspace's conversations — a glyph
//! column until the pointer is over it, the rows in full while it is and for
//! [`LINGER`] after it leaves, or always while pinned. Each of its two widths
//! is the operator's where they dragged its right edge ([`drag`]) and the
//! policy's default where they did not.
//!
//! It is a second painter of one listing and not a second listing: its rows
//! are [`Model::rows`], the switcher's under the aimed wall, spelled by
//! `crate::ui::convs` ([`rows`]). Its head is the pin and `+ new`; its foot is
//! the workspace, the one place on the glass a workspace is chosen ([`foot`]).
//!
//! **Where the pointer is, is read; when it was last over the bar, is held**
//! ([`Bar::left_at`]) — the one fact the linger needs that no frame can read
//! back, timed on egui's input clock so a test steers it.

use crate::ui::{Model, theme};

/// The right edge, dragged: the two widths as set, clamped and written.
pub mod drag;
/// The workspace at the bar's foot, and its list opening upward.
mod foot;
/// The conversations, one row each.
mod rows;

pub use drag::Dragged;
pub use rows::{EMPTY, NOTHING_AIMED};

/// **How long the bar stays open after the pointer leaves**, in seconds —
/// the number read off the ruling (§4.45 item 1), held here and nowhere else.
pub const LINGER: f64 = 0.75;
/// **How long the bar's width takes to travel between its two ends**, in
/// seconds (DESIGN §4.44 item 1): fast, so it is never waited on, and not
/// zero, so a bar that opens under the pointer slides rather than snaps.
pub const EASE: f32 = 0.12;
/// **The collapsed width's default**, in points — what a seat that dragged
/// nothing gets (§4.45 item 2): the state glyph and a few characters of a
/// name at the body size, and the panel's margin either side. **Dragged
/// wider, it shows more of each name** (bl-773e): a collapsed row and the
/// foot paint their whole words elided at the width they have, so the
/// number is a default and not a cut.
pub const SHUT: f32 = 80.0;
/// **The desk size's width** (`crate::snapshot::SIZES`), whose quarter is
/// the expanded width.
pub const DESK: f32 = 1400.0;
/// The pin's two words: it stops the auto-hiding, and it takes the stop back.
pub const PIN: &str = "pin";
pub const UNPIN: &str = "unpin";
/// The collapsed `+ new`.
pub const PLUS: &str = "+";

/// **The bar between frames**: the pin the operator chose (`Place::pinned`),
/// the widths they dragged (`Place::dragged`), when the pointer was last over
/// it, and the drag in hand.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Bar {
    /// Pinned open, whatever the pointer does.
    pub pinned: bool,
    /// **The two widths, where the operator dragged them** (§4.45 item 2).
    pub dragged: Dragged,
    /// **The input-clock instant the pointer was last over the bar**, or
    /// `None` once the linger after it has lapsed.
    pub left_at: Option<f64>,
    /// **The drag of the edge in hand**, from its first frame to its last.
    pub drag: Option<drag::Drag>,
}

impl Bar {
    /// **Whether the bar stands open at `now`.**
    pub fn open(&self, now: f64) -> bool {
        self.pinned || self.left_at.is_some_and(|at| now - at < LINGER)
    }

    /// **The pointer's news at `now`**: over the bar or not. Answers how long
    /// until the linger lapses, where one is running, so the frame can ask
    /// for a repaint at that moment.
    pub fn heard(&mut self, now: f64, over: bool) -> Option<f64> {
        if over {
            self.left_at = Some(now);
            return None;
        }
        let at = self.left_at?;
        let left = LINGER - (now - at);
        if left <= 0.0 {
            self.left_at = None;
            return None;
        }
        (!self.pinned).then_some(left)
    }
}

/// **The expanded width's default in a window `window` wide**: a quarter of
/// the desk size's width, and never more than the ceiling.
pub fn open_width(window: f32) -> f32 {
    (DESK / 4.0).min(drag::ceiling(window))
}

/// **Paint the bar** at the window's left edge, before every other panel, so
/// it stands the window's full height. Nothing paints under the enrollment.
pub fn render(ctx: &egui::Context, model: &mut Model) {
    if model.enroll.is_some() {
        return;
    }
    let now = ctx.input(|i| i.time);
    let screen = ctx.screen_rect();
    let (shut, wide) = model.bar.dragged.shown(screen.width(), drag::floor(ctx));
    // **The pointer is over the bar it last saw**: open while the clock runs,
    // the pin holds or its foot's list stands, so a pointer resting inside an
    // open bar stays inside — and shut, the edge is not the bar, so a pointer
    // reaching for it does not open what it came to drag. **While the edge is
    // in hand the bar stays in the state it was grabbed in** (§4.45 item 2):
    // the pointer is then on the edge and not over the rows.
    let held = crate::ui::status::dropup::footed(model);
    let reach = if held || model.bar.pinned || model.bar.left_at.is_some() {
        wide
    } else {
        shut - drag::HANDLE
    };
    let over = model.bar.drag.map_or_else(
        || {
            ctx.input(|i| i.pointer.hover_pos())
                .is_some_and(|at| at.x < screen.left() + reach)
        },
        |drag| drag.open,
    );
    if let Some(left) = model.bar.heard(now, over) {
        ctx.request_repaint_after(std::time::Duration::from_secs_f64(left));
    }
    let open = held || model.bar.open(now);
    // **The width eases; what it holds does not**: the rows say what `open`
    // says from the first frame, and only the edge travels, on egui's own
    // clock, so a test steps it like the linger. **A dragged edge does not
    // ease**: it is under the pointer, and the ease's memory is left standing
    // at the dragged width so the release has nowhere to travel from.
    let width = ctx.animate_value_with_time(
        egui::Id::new("bar width"),
        if open { wide } else { shut },
        if model.bar.drag.is_some() { 0.0 } else { EASE },
    );
    egui::SidePanel::left("bar")
        .resizable(false)
        .exact_width(width)
        .frame(
            egui::Frame::none()
                .fill(theme::GROUND)
                .inner_margin(drag::margin()),
        )
        .show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                let seat = foot::render(ui, model);
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    head(ui, model, open);
                    rows::render(ui, model, open);
                });
                foot::list(ui, model, seat);
            });
            drag::edge(ui, &mut model.bar, open, width);
        });
}

/// **The head**: the pin at the left, `+ new` at the right while a wall is
/// aimed and the box can take the caret.
fn head(ui: &mut egui::Ui, model: &mut Model, open: bool) {
    ui.horizontal(|ui| {
        let word = if model.bar.pinned { UNPIN } else { PIN };
        let said = if open {
            theme::worded(theme::glyph::PIN, word)
        } else {
            theme::glyph::PIN.to_owned()
        };
        if theme::paint::link(ui, &said, theme::INK_WEAK).clicked() {
            model.bar.pinned = !model.bar.pinned;
        }
        let Some(aim) = model.aim.clone() else {
            return;
        };
        if model.covered() && !model.switching {
            return;
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let said = if open {
                crate::ui::roster::wall::NEW
            } else {
                PLUS
            };
            let seat = theme::paint::link(ui, said, theme::INK_WEAK);
            crate::ui::act::tag(&seat, &[crate::verbs::PREPARE, crate::verbs::PROMPT]);
            if seat.clicked() {
                model.begin_at(&aim);
                model.chose();
            }
        });
    });
}

#[cfg(test)]
mod tests;
