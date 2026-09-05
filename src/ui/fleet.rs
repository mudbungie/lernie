//! **The fleet**: running a wall's ready balls, watching what its agents do,
//! and reading what they changed (bl-a43a; yog's `docs/REMOTE.md` §9.7).
//!
//! # The thirteenth covering pane, and the fifth about the aimed wall
//!
//! Seven ops, and every one of them carries a workspace — so this is the
//! tuning pane's shape and not the queue's: it opens on the aimed row, its two
//! reads stand while it is open, and it is retired when the aim moves. What it
//! adds to the window is the one thing the seat could see happening and could
//! not cause: until it landed, a workspace could be watched and never run.
//!
//! # The naming trap is the first thing to know about this pane
//!
//! **`fleet` and `disband` are the LOOP; `arm` and `disarm` are the
//! MONITOR.** Two families, two settings, two carriers — and one shared reply
//! kind between them, so no reader can tell which family an answer belongs to
//! by looking at it. Everything here reads the **op** back instead: the poster
//! stamps a routed receipt with the op it answers
//! (`crate::state::Said::Receipt`), the model files it under that name
//! (`crate::ui::model::fleet`), and this pane prints the op beside the flag.
//!
//! # Neither stopping act is armed, and that is an argument rather than a gap
//!
//! DESIGN §4.20's arming is for the **unmaking** — an act whose product is
//! that its subject is gone. Neither of these is that. `disband` *"stops
//! nothing that is running; everything already running is untouched and keeps
//! its ball"*, and `disarm` leaves *"every verdict already recorded on the
//! trail"*. Each is undone by doing the other thing — one `fleet` re-arms the
//! loop, one `arm` re-raises the watch — and an arming on an act that costs a
//! click to reverse teaches an operator to type through armings. What they get
//! instead is the order the unmaking gets: the acts that start things first,
//! the acts that stop them under.
//!
//! # Three boxes, because three acts carry a word nothing here can derive
//!
//! A cap, a project and a model. Each control stands down until its own box
//! has something in it — §4.20's enablement rule, where the parameter is
//! missing rather than the subject.

use crate::ui::{Model, theme};

pub use rows::{attempt, changed, churn, receipt};

/// The words a row wears, each a pure function of it.
mod rows;

/// The word that opens the pane, on the wall the window is aimed at.
pub const OPEN: &str = "run the fleet…";
/// The word that closes it.
pub const CLOSE: &str = "done";
/// The pane's own heading.
pub const HEADING: &str = "the fleet";
/// The word that starts the loop.
pub const RUN: &str = "run this many at once";
/// The word that stops it.
pub const DISBAND: &str = "stop the fleet";
/// The word that raises the watch.
pub const WATCH: &str = "watch with this model";
/// The word that drops it.
pub const DISARM: &str = "stop watching";
/// The word that flushes the inboxes.
pub const SCAN: &str = "flush the inboxes";
/// What the project box asks for.
pub const PROJECT_HINT: &str = "project";
/// What the model box asks for.
pub const MODEL_HINT: &str = "cheap model id";
/// The word on the control that lowers the cap.
pub const FEWER: &str = "−";
/// The word on the control that raises it.
pub const MORE: &str = "+";
/// The heading over the attempts.
pub const ATTEMPTS: &str = "delivery attempts";
/// The heading over what changed.
pub const CHANGES: &str = "what the agents changed";
/// What it says about a wall nobody has been answered about yet.
pub const NOT_ANSWERED: &str = "waiting to hear what this wall's agents have done";
/// What it says about a wall that answered and has none.
pub const NOTHING: &str = "this wall's agents have delivered nothing yet";

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    let Some((at, said)) = model
        .fleet
        .as_ref()
        .map(|held| (held.at.clone(), held.said.clone()))
    else {
        return false;
    };
    ui.heading(HEADING);
    ui.label(format!("on {} — {}", at.address, at.channel));
    ui.horizontal_wrapped(|ui| {
        if ui.button(CLOSE).clicked() {
            model.close_fleet();
        }
    });
    if let Some(said) = &said {
        ui.colored_label(theme::NOTICE, receipt(said));
    }
    ui.separator();
    // **The boxes are bound to the pane's own words**, not to a copy compared
    // afterwards: a draft is what the pane holds, so an edit is a write to it
    // and there is no second value for the two to disagree about. It is the
    // tuning editor's arrangement, one pane over.
    if let Some(envelope) = acts(ui, model, &at.address) {
        model.post_fleet(envelope);
    }
    ui.separator();
    listings(ui, model);
    true
}

/// The five acts, in the order a destructive one belongs in: what starts a
/// thing, then what stops it. Answers the gesture a click composed, if one
/// did, because the pane's own words are borrowed for the length of the paint.
fn acts(ui: &mut egui::Ui, model: &mut Model, wall: &str) -> Option<serde_json::Value> {
    let mut fired: Option<serde_json::Value> = None;
    let fleet = model.fleet.as_mut()?;
    ui.horizontal_wrapped(|ui| {
        if ui.button(FEWER).clicked() {
            // **Never below one.** A cap of zero is a loop that spawns nothing
            // and still reaps, which upstream refuses to spell as a cap at all
            // — `disband` is that — so it is not a value this box can send.
            fleet.cap = fleet.cap.saturating_sub(1).max(1);
        }
        ui.label(fleet.cap.to_string());
        if ui.button(MORE).clicked() {
            fleet.cap = fleet.cap.saturating_add(1);
        }
        ui.add(egui::TextEdit::singleline(&mut fleet.project).hint_text(PROJECT_HINT));
        let run = ui.add_enabled(!fleet.project.trim().is_empty(), egui::Button::new(RUN));
        crate::ui::act::tag(&run, &[crate::verbs::FLEET]);
        if run.clicked() {
            fired = Some(crate::verbs::fleet(
                wall.to_owned(),
                fleet.project.trim().to_owned(),
                fleet.cap,
            ));
        }
        let stop = ui.button(DISBAND);
        crate::ui::act::tag(&stop, &[crate::verbs::DISBAND.word]);
        if stop.clicked() {
            fired = Some(crate::verbs::disband(wall.to_owned()));
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.add(egui::TextEdit::singleline(&mut fleet.model).hint_text(MODEL_HINT));
        let watch = ui.add_enabled(!fleet.model.trim().is_empty(), egui::Button::new(WATCH));
        crate::ui::act::tag(&watch, &[crate::verbs::ARM.word]);
        if watch.clicked() {
            fired = Some(crate::verbs::arm(
                wall.to_owned(),
                fleet.model.trim().to_owned(),
            ));
        }
        let drop = ui.button(DISARM);
        crate::ui::act::tag(&drop, &[crate::verbs::DISARM.word]);
        if drop.clicked() {
            fired = Some(crate::verbs::disarm(wall.to_owned()));
        }
        let flush = ui.button(SCAN);
        crate::ui::act::tag(&flush, &[crate::verbs::SCAN.word]);
        if flush.clicked() {
            fired = Some(crate::verbs::scan(wall.to_owned()));
        }
    });
    fired
}

/// The two standing reads, each with its own emptiness.
fn listings(ui: &mut egui::Ui, model: &Model) {
    let attempts = model.attempts.clone();
    let work = model.work.clone();
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            ui.label(ATTEMPTS);
            match attempts.as_deref() {
                None => drop(ui.label(NOT_ANSWERED)),
                Some([]) => drop(ui.label(NOTHING)),
                Some(rows) => {
                    for row in rows {
                        ui.separator();
                        for said in attempt(row) {
                            ui.label(said);
                        }
                    }
                }
            }
            ui.separator();
            ui.label(CHANGES);
            match work.as_deref() {
                None => drop(ui.label(NOT_ANSWERED)),
                Some([]) => drop(ui.label(NOTHING)),
                Some(rows) => {
                    for row in rows {
                        ui.separator();
                        ui.label(changed(row));
                        for file in &row.files {
                            ui.colored_label(
                                theme::tone_ink(&crate::reply::convs::Tone::Weak),
                                churn(file),
                            );
                        }
                    }
                }
            }
        });
}

#[cfg(test)]
mod tests;
