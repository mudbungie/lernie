//! **The login pane**: what this wall can sign in to, and the act that signs it
//! in (yog's `docs/REMOTE.md` §8.3; PROTOCOL 13).
//!
//! # The act is the ENGINE'S and this pane only asks for it
//!
//! `bz --login` runs on the engine, inside the named workspace's wall, so the
//! credential lands where the agents that read it run and **nothing
//! credential-shaped crosses this wire** (REMOTE §8.3). This crate spawns no
//! sign-in, holds no run and has no `exec` at all. What it does is post one
//! gesture and paint the lines that come back — which is the whole of why the
//! surface is portable.
//!
//! **Closing the pane terminates nothing.** The run is engine RAM, one per
//! workspace × provider, replaced by the operator's own next sign-in and swept
//! at an hour. This side gets that for free and must not re-implement it.
//!
//! # Which sphere the sign-in is for, said in the surface
//!
//! The wall's name and the channel it came down, and — for a wall held
//! elsewhere — that it is held elsewhere, off the roster's own client-side
//! channel stamp (`crate::ui::Channel`). **Never a host address**: two entries
//! naming one listener are still two trust relationships, and an address in
//! this sentence would be this end telling the operator where a credential is
//! going to land.
//!
//! # The loopback remedy is stated, never built
//!
//! A row whose flow needs a browser at the ENGINE's own loopback can only be
//! completed from a browser on that box, or through a port-forward the operator
//! sets up — *an operator act on boxes the operator administers*, and REMOTE
//! §8.3 rules it is stated as a remedy and never made a channel feature.
//!
//! **It is said for every row on a wall held elsewhere, not for the browser-only
//! ones**, and that is a limit rather than a choice: the `device` column is on
//! yog's own provider row and not on the view that crosses the wire
//! (`crate::reply::providers`), so this seat cannot tell the two apart. A
//! stated remedy that is sometimes unnecessary beats a silence that is
//! sometimes wrong.
//!
//! # One control fires two ops, because following the run is part of starting it
//!
//! `sign in to this row` carries `act:login act:login-tail`. The act starts the
//! run and the pane's `following` is what stands the held lane up, so a seat
//! that tagged only the act would be claiming `login-tail` is reachable
//! nowhere. It is the start control's shape exactly (§4.16).
//!
//! # The run-by-hand fallback is the engine's sentence, with one of ours
//!
//! `fallback` is composed by the end that knows the wall and arrives only on a
//! non-zero exit. The original surface spelled a local wall's fallback as an
//! `exec` on this box and an entry's as something else; both are the same case
//! now — **an act the operator runs on the box that HOLDS the wall** — so what
//! this pane adds is one sentence saying so, naming the channel and no address.

use crate::reply::providers::ProviderRow;
use crate::ui::{Model, theme};

/// The followed run's own half of the pane — its lines, its exit and the
/// command to run by hand.
pub mod run;

/// The word that opens the pane, on the wall the window is aimed at.
pub const OPEN: &str = "sign in…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "sign in";
/// What it says for a wall nobody has been answered about yet.
pub const NOT_ANSWERED: &str = "waiting to hear what this wall can sign in to";
/// What it says for a wall that answered and routes no provider at all. A fact
/// about the workspace, and the one empty state here that is not a wait.
pub const NO_PROVIDERS: &str = "this wall routes no provider";
/// The word on the control that starts a sign-in on a row.
pub const SIGN_IN: &str = "sign in to this row";
/// The word on the control that asks a row what it offers.
pub const OFFERS: &str = "what it offers…";
/// What it says under a row that was asked and has not answered.
pub const NOT_OFFERED: &str = "waiting to hear what this row offers";
/// What it says for a row that answered and offers nothing.
pub const OFFERS_NOTHING: &str = "this row offers no model id";
/// The sentence that says where the sign-in is actually happening.
pub const ELSEWHERE: &str = "this wall is held on another box, and the sign-in \
                             runs there — the credential lands in its wall, not \
                             on this one";
/// The §8.3 remedy for a row whose flow wants a browser at the engine's own
/// loopback, said wherever the engine is not this box.
pub const LOOPBACK: &str = "if the flow opens an authorize URL, it redirects to \
                            the ENGINE's loopback: finish it in a browser on \
                            that box, or forward the port by hand";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if model.login.is_none() {
        return false;
    }
    ui.heading(HEADING);
    if let Some(aim) = model.aim.clone() {
        ui.label(format!("on {} — {}", aim.address, aim.channel));
    }
    // **Both sentences hang off one fact and are said once**, above the rows
    // rather than on each: where the engine is, is a property of the wall.
    if model.elsewhere() {
        ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), ELSEWHERE);
        ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), LOOPBACK);
    }
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_login();
        }
    });
    ui.separator();
    match model.providers.clone() {
        None => {
            ui.label(NOT_ANSWERED);
        }
        Some(rows) if rows.is_empty() => {
            ui.label(NO_PROVIDERS);
        }
        // The list scrolls and the heading above it does not — the shape every
        // list in this window keeps (bl-e5d2).
        Some(rows) => {
            egui::ScrollArea::vertical()
                .id_salt(HEADING)
                .auto_shrink(false)
                .show(ui, |ui| {
                    for row in &rows {
                        provider(ui, model, row);
                    }
                });
        }
    }
    true
}

/// One provider row: what it is, what the engine knows about its credential,
/// and the two controls on it.
fn provider(ui: &mut egui::Ui, model: &mut Model, row: &ProviderRow) {
    ui.separator();
    ui.label(row.name.clone());
    let weak = theme::tone_ink(&crate::reply::convs::Tone::Weak);
    ui.colored_label(weak, row.fact.clone());
    if let Some(takes) = row.takes() {
        ui.colored_label(weak, takes);
    }
    // **A blocked row states the engine's own reason and offers no sign-in.**
    // A control that fired an act the far end has already said it will refuse
    // is a control that only looks actionable, which is the one thing every
    // pane in this window is written not to be.
    if let Some(why) = &row.blocked {
        ui.colored_label(theme::NOTICE, why.clone());
    }
    // **Every row wraps**, because this pane stands in the central panel and
    // the central panel is what the two side panels leave (bl-dc07).
    ui.horizontal_wrapped(|ui| {
        let start = ui.add_enabled(row.signable(), egui::Button::new(SIGN_IN));
        // **Two ops on one control** (DESIGN §4.16): the click starts the run,
        // and following it is what opens this seat's held lane on
        // `login-tail` (`crate::offframe::signin`). It is the start control's
        // own shape — `prepare` then `prompt` — and the lane, like the tail's,
        // has no control of its own because nothing but this act can make one
        // exist.
        crate::ui::act::tag(
            &start,
            &[crate::verbs::LOGIN.word, crate::verbs::LOGIN_TAIL.word],
        );
        if start.clicked() {
            model.post_signin(&row.name);
        }
        let offers = ui.button(OFFERS);
        crate::ui::act::tag(&offers, &[crate::verbs::MODELS.word]);
        if offers.clicked() {
            model.post_offering(&row.name);
        }
    });
    offering(ui, model, &row.name);
    run::render(ui, model, &row.name);
}

/// What this row offers, under the row that asked — and only under it, which is
/// what the pane's `asking` is for: the reply carries no provider name, so the
/// question is the only thing that can say which row it answers.
fn offering(ui: &mut egui::Ui, model: &Model, name: &str) {
    if model.login.as_ref().and_then(|pane| pane.asking.as_deref()) != Some(name) {
        return;
    }
    match model.offered.as_deref() {
        None => {
            ui.label(NOT_OFFERED);
        }
        Some([]) => {
            ui.label(OFFERS_NOTHING);
        }
        Some(ids) => {
            ui.label(ids.join("  "));
        }
    }
}

#[cfg(test)]
mod tests;
