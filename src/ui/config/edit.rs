//! **The editor**: the box a config file's new bytes are typed into, and the
//! two controls the box enables (bl-4855; DESIGN §4.30).
//!
//! Split from [`super`] at the design-time budget on a real seam: [`super`] is
//! *which file*, [`super::settings`] is *what the engine reads in it*, and this
//! is *what this operator would put in it instead*.
//!
//! # The box IS the arming
//!
//! DESIGN §4.20's idiom is for an act whose subject ceases to exist, and it
//! answers with a PLACE — a covering pane holding nothing else. A config write
//! is not that act: its subject is a file that still exists afterwards, and
//! reaching the control means having authored the file's whole text, which no
//! mis-aimed click does. So it takes §4.20's other half, the ENABLEMENT: the
//! control is dark until the box differs from what the engine last answered,
//! and **the refusal is spelled beside it**, because a greyed control says a
//! thing is not live and nothing about what would make it live.
//!
//! # Nothing records that a write is in flight, and that is the enablement
//! # working rather than a gap
//!
//! The control asks *is there anything to write*, which is a fact about the
//! world — the box against the engine's own latest answer — and stays true and
//! correct while a write crosses. A flag saying *asked* would be this seat
//! holding a second opinion about a question the standing read answers every
//! beat, and the worst it prevents is writing the same bytes twice, which is
//! the same file.
//!
//! # What `revert` is for, and why it is one control and not two
//!
//! The box becomes the file. That is the way out of an edit and it is also the
//! way to take another writer's bytes over your own draft — one act, because
//! it is one act. It is what makes the write ordinary in §4.20's own test:
//! undone by doing the other thing, with the other thing on the glass beside
//! it.

use crate::ui::{Model, keys, theme};
use crate::verbs::Where;

/// The control that writes the box to the file.
pub const WRITE: &str = "write";
/// The control that puts the engine's answer back in the box.
pub const REVERT: &str = "revert";
/// The refusal spelled beside the dark controls.
pub const NOTHING_TO_WRITE: &str = "the box holds what the file holds";
/// What it says when the file is neither what the box holds nor what the box
/// and the engine last agreed on — which leaves one reading, another writer.
pub const MOVED: &str = "this file has changed on the engine since the box last \
                         agreed with it — reverting takes the engine's bytes";

/// Paint the box and take the clicks on it.
pub(super) fn render(ui: &mut egui::Ui, model: &mut Model, at: &Where, answered: &str) {
    model.draft_config(answered);
    if let Some(text) = model.draft_box() {
        ui.add(
            egui::TextEdit::multiline(text)
                .id(egui::Id::new(keys::CONFIG_ID))
                .desired_width(f32::INFINITY)
                .code_editor(),
        );
    }
    let Some(draft) = model.drafted() else {
        return;
    };
    if draft.moved(answered) {
        ui.colored_label(theme::NOTICE, MOVED);
    }
    let unwritten = draft.unwritten(answered);
    ui.horizontal_wrapped(|ui| {
        let write = ui.add_enabled(unwritten, egui::Button::new(WRITE));
        crate::ui::act::tag(&write, &[crate::verbs::CONFIG]);
        if write.clicked() {
            model.write_config(at, draft.text.clone());
        }
        if ui
            .add_enabled(unwritten, egui::Button::new(REVERT))
            .clicked()
        {
            model.revert_config(answered);
        }
        if !unwritten {
            ui.colored_label(
                theme::tone_ink(&crate::reply::convs::Tone::Weak),
                NOTHING_TO_WRITE,
            );
        }
    });
}

#[cfg(test)]
mod tests;
