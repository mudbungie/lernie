//! **The typed view**: every setting the engine's schema found in the bytes
//! below it, and the engine's own judgement of each (REMOTE §9.18).
//!
//! Split from [`super`] at the design-time budget on the seam the pane already
//! paints: [`super`] is *which file*, this is *what the engine reads in it*,
//! and [`super::edit`] is *what this operator would put in it instead*.
//!
//! **Nothing here composes a judgement.** A setting's `fault` is the same call
//! yog's own pick gate makes, so what is red is what the far end says is
//! wrong; its absence is *nothing is wrong with this value* and never *nobody
//! looked*. The bounds are decoded and stated for the same reason and enforced
//! by nothing: a seat that refused a value would be a second authority across
//! a boundary.

use crate::reply::config::{Config, Setting};
use crate::ui::theme;

/// Every setting the schema found, each under the declaration it belongs to.
pub(super) fn render(ui: &mut egui::Ui, held: &Config) {
    if held.settings.is_empty() {
        ui.colored_label(
            theme::tone_ink(&crate::reply::convs::Tone::Weak),
            super::NO_SETTINGS,
        );
        return;
    }
    for row in &held.settings {
        setting(ui, row);
    }
}

/// One setting: where it lives, what it says, what it takes, and the engine's
/// judgement of it where there is one.
fn setting(ui: &mut egui::Ui, row: &Setting) {
    let weak = theme::tone_ink(&crate::reply::convs::Tone::Weak);
    // **Every row wraps**, because this pane stands in the central panel and
    // the central panel is what the two side panels leave (bl-dc07).
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{}.{}", row.entry, row.name));
        ui.label(row.value.clone());
        ui.colored_label(weak, row.control.says());
    });
    ui.colored_label(weak, row.help.clone());
    if let Some(fault) = &row.fault {
        ui.colored_label(theme::NOTICE, fault.clone());
    }
}
