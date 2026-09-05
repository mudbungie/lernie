//! **The followed run's half of the login pane** — what one sign-in has said,
//! how it ended, and the command to run by hand where the engine composed one.
//!
//! Split from [`super`] at the design-time budget on a seam that is a real one:
//! that module is *what this wall can sign in to*, which is a table the
//! standing read replaces, and this is *what one run of one sign-in printed*,
//! which arrives on a held lane at a human's pace
//! (`crate::offframe::signin`). The first changes when a provider row grows a
//! fact; the second when the flow's output does.

use crate::ui::{Model, theme};

/// What it says under a row nobody has signed in to.
pub const NO_RUN: &str = "nobody has signed in to this row here";

/// What the pane says above the engine's run-by-hand command.
pub const BY_HAND: &str = "run it by hand, on the box that holds this wall:";

/// The followed run, under the row it belongs to: its lines, the exit it
/// settled on, and the command to run by hand where the engine composed one.
pub fn render(ui: &mut egui::Ui, model: &Model, name: &str) {
    if model.following().as_deref() != Some(name) {
        return;
    }
    let Some(run) = model.signin.clone() else {
        return;
    };
    // **The empty fold is a reading, not a silence** (REMOTE §10): the lane
    // opens on one frame of nothing, and *nobody has signed in here* is what
    // that frame says.
    if run.lines.is_empty() && !run.settled() {
        ui.label(NO_RUN);
        return;
    }
    for line in &run.lines {
        // **Both streams are painted and stderr is a tone, never a filter**:
        // bz writes the authorize URL, the device code and every failure's
        // remedy to stderr, so a pane that hid it would hide the flow.
        let ink = if line.err {
            theme::NOTICE
        } else {
            theme::tone_ink(&crate::reply::convs::Tone::Plain)
        };
        ui.colored_label(ink, line.text.clone());
    }
    if let Some(exit) = run.outcome {
        ui.label(settled(exit));
    }
    if let Some(command) = &run.fallback {
        ui.label(BY_HAND);
        ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), command);
    }
}

/// **What a settled run says about how it ended.** A function rather than a
/// format string at the call site, because the pane's own suite names the
/// sentence and a second spelling would be the thing that drifted.
pub fn settled(exit: i32) -> String {
    if exit == 0 {
        "signed in".to_owned()
    } else {
        format!("the sign-in ended with status {exit}")
    }
}

#[cfg(test)]
mod tests;
