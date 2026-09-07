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
//! quickly*. Its name box wears an id of its own ([`NAME_BOX`]) so the field
//! is one thing on the glass and one thing to the keyboard.
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
//!
//! The one place pixels ARE the altitude is [`symbol`], which decides the grid
//! the matrix is drawn on. That is where the scale rule and its suite live.

use crate::ui::{Grade, Model, theme};

/// The word that opens an enrollment, on the wall the window is aimed at.
pub const OPEN: &str = "enroll…";
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
/// **The id the name box wears.** A box that takes text is named so the
/// keyboard's gate can compare against it (`crate::ui::keys`); this one is not
/// in `keys::BOXES` and does not need to be, because a covering pane stands
/// the arrows down for as long as it covers (`crate::ui::Model::covered`).
const NAME_BOX: &str = "the enrollment's name box";

/// Paint the enrollment, and take the clicks on it. Answers whether there was
/// one to paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    let Some(enrolling) = model.enroll.clone() else {
        return false;
    };
    // **The heading, then one line carrying the subject and the way out**
    // (`docs/STYLE.md` §5, *a covering pane*): what is being enrolled into,
    // one step weaker, with the close at the end of that same line.
    //
    // **One close, and it is on that line in both halves.** The control that
    // closes this pane is the control that FORGETS the material, so it must be
    // in the same place before the symbol arrives and after — a way out that
    // moves when the answer lands is a way out an operator has to look for
    // while a private key is on the screen.
    ui.label(egui::RichText::new(HEADING).heading().color(theme::INK));
    ui.horizontal(|ui| {
        ui.colored_label(
            theme::INK_WEAK,
            format!(
                "into {} on {}",
                enrolling.aim.address, enrolling.aim.channel
            ),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(CLOSE).clicked() {
                model.close_enrollment();
            }
        });
    });
    ui.add_space(theme::space::S);
    match &enrolling.shown {
        Some(shown) => {
            ui.label(shown.caption.clone());
            ui.colored_label(theme::accent(theme::State::Annotation), KEPT);
            ui.add_space(theme::space::S);
            symbol::paint(ui, &shown.symbol);
        }
        None => form(ui, model),
    }
    true
}

/// The half before the answer: what to call the box, what grade to mint it at,
/// and the button that spends the act.
///
/// **Nothing here is laid side by side, because this pane is not as wide as
/// the window** (bl-dc07). The enrollment stands in the central panel, which
/// is what the two side panels leave: at a 400-point window that is about 120
/// points, and a row of controls laid on one line runs off the right edge —
/// which is how the name box, the `foot` grade and the way out all left the
/// window once. The field is full width, the two grades are one row each, and
/// the close is on the pane's own subject line above (see [`render`]).
fn form(ui: &mut egui::Ui, model: &mut Model) {
    let Some(enrolling) = model.enroll.as_mut() else {
        return;
    };
    // **The hint IS the label** (`docs/STYLE.md` §5, *the composer*): a field
    // with its own word inside it is one thing on the glass where a label
    // beside a box is two, and this pane is the narrowest one in the window.
    theme::paint::field(
        ui,
        egui::Id::new(NAME_BOX),
        &mut enrolling.name,
        NAME_HINT,
        None,
    );
    // **The two grades are rows, one per line** — never a `selectable_label`,
    // which draws a box (§5, *a row*). The chosen one wears the brand rule
    // that says where the operator is standing.
    for grade in Grade::both() {
        let chosen = enrolling.grade == grade;
        if theme::paint::row(ui, &grade.word(), theme::INK, None, chosen, 0.0).clicked() {
            enrolling.grade = grade;
        }
    }
    let ready = enrolling.ready();
    let minting = enrolling.minting();
    ui.add_space(theme::space::S);
    ui.horizontal_wrapped(|ui| {
        let mint = ui.add_enabled(ready, egui::Button::new(SEND));
        crate::ui::act::tag(&mint, &[crate::verbs::ENROLL.word]);
        if mint.clicked() {
            model.post_enrollment();
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

mod symbol;

#[cfg(test)]
mod tests;
