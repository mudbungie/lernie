//! **The followed run's half of the login pane** — what one sign-in has said,
//! how it ended, and the command to run by hand where the server composed one.
//!
//! Split from [`super`] at the design-time budget on a seam that is a real one:
//! that module is *what this workspace can sign in to*, which is a table the
//! standing read replaces, and this is *what one run of one sign-in printed*,
//! which arrives on a held lane at a human's pace
//! (`crate::offframe::signin`). The first changes when a provider row grows a
//! fact; the second when the flow's output does.
//!
//! # A run waiting for a code is finished here
//!
//! A row with no device endpoint stops on its authorize URL and waits (REMOTE
//! §8.3, edition 31), and the lane says so with `authorize`. This seat is
//! where the human is, so this is where the URL opens: one control hands it
//! to the desktop's browser, one puts it on the clipboard for a browser
//! somewhere else, and a box takes what the vendor showed, sent back as
//! `login`'s `code`. **Whether to copy a code or the whole address is the
//! lane's own line to say** — bz knows the row and this pane does not, so
//! nothing here restates a vendor's page.
//!
//! **The key is the whole gate.** No server older than edition 31 writes
//! `authorize`, so a pane that offers the box only under one never sends a
//! `code` to a server that would read it as a restart.

use crate::ui::{Model, theme};

/// What it says under a row nobody has signed in to.
pub const NO_RUN: &str = "nobody has signed in to this row here";

/// The word on the control that opens a waiting run's URL in this desktop's
/// browser.
pub const OPEN: &str = "open the sign-in page";
/// What it does.
pub const OPEN_HELP: &str = "opens this sign-in's page in this desktop's browser";
/// The word on the control that copies the same URL.
pub const COPY: &str = "copy its address";
/// What it does.
pub const COPY_HELP: &str = "copies the sign-in page's address, for a browser somewhere else";
/// What the paste box says while it is empty.
pub const PASTE_HINT: &str = "paste what the sign-in page gave you";
/// The word on the control that hands the paste back.
pub const FINISH: &str = "finish the sign-in";
/// What it does.
pub const FINISH_HELP: &str =
    "sends what you pasted to the box that holds this provider, which checks it there";

/// What the pane says above the server's run-by-hand command.
pub const BY_HAND: &str = "run it by hand, on the box this workspace lives on:";

/// The followed run, under the row it belongs to: its lines, the exit it
/// settled on, and the command to run by hand where the server composed one.
pub fn render(ui: &mut egui::Ui, model: &mut Model, name: &str) {
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
    if let Some(url) = run.awaited() {
        finish(ui, model, name, url);
    }
    if let Some(exit) = run.outcome {
        ui.label(settled(exit));
    }
    if let Some(command) = &run.fallback {
        ui.label(BY_HAND);
        ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), command);
    }
}

/// **The waiting run's three controls**: the URL to this desktop's browser or
/// to the clipboard, and the box what the vendor showed is pasted into.
fn finish(ui: &mut egui::Ui, model: &mut Model, name: &str, url: &str) {
    ui.horizontal_wrapped(|ui| {
        if theme::paint::button(ui, OPEN, true, OPEN_HELP).clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url));
        }
        if theme::paint::button(ui, COPY, true, COPY_HELP).clicked() {
            ui.ctx().copy_text(url.to_owned());
        }
    });
    let sent = model.code_draft().is_some_and(|draft| {
        ui.add(
            egui::TextEdit::singleline(draft)
                .desired_width(ui.available_width())
                .hint_text(PASTE_HINT),
        );
        let send = theme::paint::button(ui, FINISH, !draft.trim().is_empty(), FINISH_HELP);
        crate::ui::act::tag(&send, &[crate::verbs::LOGIN.word]);
        send.clicked()
    });
    if sent {
        model.post_code(name);
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
