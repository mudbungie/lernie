//! **The enrollment pane**: a name, a grade, and the symbol that comes back
//! (yog's `docs/REMOTE.md` §8.4).
//!
//! # It stands where the conversation would, and that is the design
//!
//! The shell's rule is *a bar rather than a modal*, because a refusal about one
//! pane must not stop the operator reading the other three. This is the
//! exception and it earns it: what the pane holds is a **private key on a
//! screen**, the whole act is "look at this now and then close it", and leaving
//! a conversation legible behind it would invite exactly the thing the material
//! must not have — a long life on a display. So it replaces the central panel
//! while it is open, and the control that closes it is the one that forgets.
//!
//! # It is a modal, so it behaves like one (bl-7574)
//!
//! The argument above is about the WINDOW and the pane only ever covered a
//! PANEL, which left it a modal in name. Two things close that, and neither
//! widens what it covers.
//!
//! **Escape closes it** ([`crate::ui::Model::escape`]), doing exactly what
//! [`CLOSE`] does — the material included, because the control that closes this
//! pane is the control that forgets. Reachable by Tab is not the same as
//! operable, and this is the one pane whose stated purpose is *close it
//! quickly*. Its name box wears [`crate::ui::keys::NAME_ID`] so the gate that
//! makes Escape mean *leave the box* while typing covers it too.
//!
//! **Nothing live paints under it**: the composer is a bottom panel and so
//! outside what a central panel covers, which put a live `start` control —
//! firing a conversation on the very wall being enrolled into — beneath the
//! symbol. `crate::ui::shell` stands it down while an enrollment is open. The
//! roster and the conversation list stay, and that is deliberate: they are
//! where the operator looks, not what they act with, and the doc's reason above
//! is about a *conversation* legible behind the material.
//!
//! # The symbol is geometry, so the assertions are geometry
//!
//! A QR symbol has no glyphs. `crate::paint_probe` is the one walk over painted
//! *text* and it has nothing to say about one, so the tests assert the **module
//! matrix** — which is the thing that is right or wrong — and the paint probe
//! asserts the words around it. Pixels are the wrong altitude to be right at:
//! a symbol drawn at the wrong scale still carries the same bytes, and a symbol
//! with a wrong module does not.

use crate::qr::Symbol;
use crate::ui::{Grade, Model, theme};

/// The word that opens an enrollment, on the wall the window is aimed at.
pub const OPEN: &str = "enroll a box…";
/// The word that spends it.
pub const SEND: &str = "mint";
/// The word that closes it and forgets the material.
pub const CLOSE: &str = "done — forget it";
/// The pane's own heading.
pub const HEADING: &str = "enroll a box";
/// What the name box says before anything is typed.
pub const NAME_HINT: &str = "the new box's name";
/// The line under the symbol. It says the one thing an operator cannot see by
/// looking.
pub const KEPT: &str = "not written down anywhere — scan it now, or enroll again";
/// What stands where the symbol will be, while the engine is minting.
pub const MINTING: &str = "minting…";

/// How many screen points one module gets. Small enough that a version-40
/// symbol still fits a pane, large enough that a camera resolves one.
const MODULE: f32 = 4.0;
/// The quiet zone, in modules — the standard's four, which a decoder uses to
/// find the symbol's edge.
const QUIET: usize = 4;

/// Paint the enrollment, and take the clicks on it. Answers whether there was
/// one to paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    let Some(enrolling) = model.enroll.clone() else {
        return false;
    };
    ui.heading(HEADING);
    ui.label(format!(
        "into {} on {}",
        enrolling.aim.address, enrolling.aim.channel
    ));
    ui.separator();
    match &enrolling.shown {
        Some(shown) => {
            ui.label(shown.caption.clone());
            ui.colored_label(theme::NOTICE, KEPT);
            if ui.button(CLOSE).clicked() {
                model.close_enrollment();
            }
            ui.separator();
            paint(ui, &shown.symbol);
        }
        None => form(ui, model),
    }
    true
}

/// The half before the answer: what to call the box, what grade to mint it at,
/// and the button that spends the act.
/// **Every row here WRAPS, because this pane is not as wide as the window**
/// (bl-dc07). The enrollment stands in the central panel, which is what the two
/// side panels leave: at a 400-point window that is about 120 points, and an
/// unwrapped row lays its controls on one line however long the line has to be.
/// The name box, the `foot` grade and the control that CLOSES the pane all went
/// off the right edge of the window — and the one that closes it is the one
/// that forgets the material, so the pane holding a private key was the pane
/// with no visible way out of it.
fn form(ui: &mut egui::Ui, model: &mut Model) {
    let Some(enrolling) = model.enroll.as_mut() else {
        return;
    };
    ui.horizontal_wrapped(|ui| {
        ui.label(NAME_HINT);
        ui.text_edit_singleline(&mut enrolling.name);
    });
    ui.horizontal_wrapped(|ui| {
        for grade in Grade::both() {
            let chosen = enrolling.grade == grade;
            if ui.selectable_label(chosen, grade.word()).clicked() {
                enrolling.grade = grade;
            }
        }
    });
    let ready = enrolling.ready();
    let minting = enrolling.minting();
    ui.horizontal_wrapped(|ui| {
        if ui.add_enabled(ready, egui::Button::new(SEND)).clicked() {
            model.post_enrollment();
        }
        if ui.button(CLOSE).clicked() {
            model.close_enrollment();
        }
    });
    // **A mint in flight is said, and the control that made it is closed.** The
    // gesture a frame composes is drained within the beat, so the outbox cannot
    // answer "has this been asked" — and an operator with no answer to that
    // clicks again, which is a second box.
    if minting {
        ui.label(MINTING);
    }
}

/// **The symbol, as rectangles.** One filled rectangle per dark module on a
/// light ground, with the quiet zone painted rather than assumed: a decoder
/// uses it to find the symbol's edge, and a pane's own background is not it.
fn paint(ui: &mut egui::Ui, symbol: &Symbol) {
    let side = points(symbol.side() + 2 * QUIET);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(side, side), egui::Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, theme::PAPER);
    for y in 0..symbol.side() {
        for x in 0..symbol.side() {
            if symbol.dark(x, y) {
                painter.rect_filled(module(rect, x, y), 0.0, theme::INK);
            }
        }
    }
}

/// Where one module sits inside the symbol's rectangle, quiet zone included.
fn module(rect: egui::Rect, x: usize, y: usize) -> egui::Rect {
    egui::Rect::from_min_size(
        rect.min + egui::vec2(points(x + QUIET), points(y + QUIET)),
        egui::vec2(MODULE, MODULE),
    )
}

/// `n` modules, in screen points. Through `u16` rather than by a cast: the
/// house lint set denies a lossy numeric cast and its only home would be a
/// manifest-wide relaxation, and the widest symbol there is measures 185
/// modules with its quiet zone — so the conversion is exact and the saturation
/// is unreachable by arithmetic rather than by hope.
fn points(n: usize) -> f32 {
    f32::from(u16::try_from(n).unwrap_or(u16::MAX)) * MODULE
}

#[cfg(test)]
mod tests;
