//! **The place a trail is cut in** (bl-b8f7; DESIGN §4.20's idiom, §4.35's
//! reading of it): the engines it would cut, the way out, and the act.
//!
//! # It is an unmaking, so it is a PLACE
//!
//! `clear-trail` truncates: what is gone is gone, and the row that replaces it
//! is not the rows it replaced. Everything else on the trail pane reads, so
//! this is the one control there that destroys — and §4.20's answer to a
//! control that destroys is a pane of its own, opened from the surface that
//! names its subject, holding nothing else, with the way out first. All three
//! of that section's reasons carry across unchanged: a covering pane is the
//! only placement identical in both layout shapes, it is already inside the
//! reach walk and the parity world set, and a pane an operator moves through
//! must not be able to take a mis-aimed click that cuts something.
//!
//! # There is no name to type back, and that is what the pane says
//!
//! §4.20's arming is **the subject's own name**, and it is an enablement
//! because the wire makes it one: `delete-workspace` carries a `typed` field
//! and refuses unless it matches. `clear-trail` carries **no field at all** —
//! the envelope is `{"op": "clear-trail"}` and nothing else — and that same
//! section rules that *"the seat reads which of the two a `typed` is off the
//! wire's own grammar and invents no policy"*. A box this seat invented would
//! be an arming the engine does not have, on a act it does not gate.
//!
//! So what the arming was FOR is bought a different way, and the pane is the
//! purchase: the subject is stated before the act is offered — every channel
//! this box holds, named, one to a line — and the act is the last control on
//! the pane rather than one an operator passes.
//!
//! # The fan is the right reading, said in words
//!
//! `clear-trail` names no workspace, so the poster fans it and one gesture
//! cuts the trail of every engine. That is not a shape to work around: the
//! pane above it *is* the union across channels, and there is no seam in
//! `crate::ui::Posted` for a channel because the envelope's own workspace
//! field is this seat's one addressing table (`crate::envelope`). What the
//! seat owes is saying so, which is why this pane lists the engines by name
//! rather than counting them.

use crate::ui::{Model, theme};

/// The word that opens it, on the trail pane.
pub const OPEN: &str = "cut every trail…";
/// The word that closes it, having cut nothing. It says what it keeps rather
/// than what it abandons, for the unmaking's reason: `cancel` would name the
/// destruction as the thing in progress, and nothing is in progress.
pub const CLOSE: &str = "keep them";
/// The word that spends it.
pub const CONFIRM: &str = "cut them now";
/// The pane's own heading.
pub const HEADING: &str = "cutting every trail";
/// What it says about what it would do, before it offers to do it.
pub const WHAT: &str =
    "every row before the cut is gone; the cut itself is logged as the new trail's first row";
/// What it says about how far the cut reaches. The engines are named under it.
pub const REACH: &str = "one gesture cuts the trail of every engine this box holds:";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if !model.clearing() {
        return false;
    }
    ui.heading(HEADING);
    ui.colored_label(theme::NOTICE, WHAT);
    ui.separator();
    ui.label(REACH);
    // **The subject, in the roster's own words** — every channel this box
    // holds, named one to a line. It is read off the roster rather than off
    // the trail's own sections because the gesture goes down every channel the
    // standing set holds, whether or not that channel has answered
    // (`crate::offframe::poster`), and naming only the ones that answered
    // would understate what the act reaches.
    for chunk in &model.roster.clone() {
        ui.colored_label(
            theme::tone_ink(&crate::reply::convs::Tone::Weak),
            crate::ui::roster::header(&chunk.channel),
        );
    }
    ui.separator();
    ui.horizontal_wrapped(|ui| {
        // **The way out is first**, and therefore first in the tab order too.
        if ui.button(CLOSE).clicked() {
            model.close_clearing();
        }
        let spend = ui.button(CONFIRM);
        crate::ui::act::tag(&spend, &[crate::verbs::CLEAR_TRAIL.word]);
        if spend.clicked() {
            model.post_clear_trail();
        }
    });
    true
}

#[cfg(test)]
mod tests;
