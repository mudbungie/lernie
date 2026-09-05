//! **What the agents changed, and the three acts a row of it earns** (§4.36;
//! yog's VISION §4.10) — the n-candidate path, on the listing that already
//! names every value it needs.
//!
//! Split from [`super`] at the design-time budget on a seam the pane already
//! had: [`super`] is *the wall's own loop and watch*, and this is *one
//! obligation's attempts*.
//!
//! # The row says which act it earns, and there is no fourth state
//!
//! A work-diff row carries a `handle` or it does not, and upstream's encoder
//! is the one that decides: a row with one is a **candidate** — an attempt on
//! `attempt/<handle>` waiting to be accepted or released — and a row without
//! is the ball's **own claim**, whose delivery obligation is the thing a fan
//! spreads. So the three controls are not a mode the pane holds; they are what
//! the row IS. Nothing here re-derives that, and a state upstream grows paints
//! as itself with whichever half of the pair its `handle` puts it in.
//!
//! # Two boxes and a count, because three acts carry a word nothing derives
//!
//! A goal, because n conversations nobody said anything to is n drivers
//! launched for nothing. A summary, because it becomes the delivery subject
//! verbatim and balls tags it with the handle — the only acceptance mark there
//! is. And a count, which is a stepper rather than a box for the cap's reason
//! one section up. Each control stands down until its own word is there
//! (§4.20's enablement rule), and **the count never goes below two**: upstream
//! reads 1 and 0 as *materialize nothing and hand back the ordinary claim
//! binding*, which is this pane's other control already.
//!
//! # Rejection has no control because it has no op
//!
//! A loser is one that was never delivered. `retire` releases a worktree and
//! is not a rejection — it changes no delivery target, ever — so nothing here
//! is armed: DESIGN §4.20 is for an act whose product is that its subject is
//! gone, and a retired candidate's source ref stays addressable unless this
//! project's own declared retention says otherwise. Which it did is the
//! engine's answer and the bar paints it (`crate::ui::model::Notice::retired`)
//! rather than this end predicting a policy it has not read.

use serde_json::Value;

use crate::reply::diff::Diff;
use crate::ui::model::Spread;
use crate::ui::{Model, theme};

/// The word on the control that spreads a ball's obligation.
pub const SPREAD: &str = "spread it over";
/// The word on the control that accepts one candidate.
pub const ACCEPT: &str = "accept this one";
/// The word on the control that releases a candidate's worktree.
pub const RELEASE: &str = "release its worktree";
/// What the goal box asks for.
pub const GOAL_HINT: &str = "what each candidate is for";
/// What the summary box asks for.
pub const SUMMARY_HINT: &str = "the delivery subject";
/// The word on the control that asks for fewer candidates. It carries the
/// step where the cap's does not, because the two steppers stand on one pane
/// and a bare glyph twice is one control an operator cannot tell from another.
pub const FEWER: &str = "−1";
/// The word on the control that asks for more.
pub const MORE: &str = "+1";
/// How wide the two boxes are.
const WIDTH: f32 = 260.0;
/// The fewest attempts a spread can ask for. One and zero materialize nothing
/// upstream, so they are not values this box can send.
const LEAST: u64 = 2;

/// Paint the listing and take the clicks on it.
pub(super) fn render(ui: &mut egui::Ui, model: &mut Model, rows: &[Diff]) {
    let mut act: Option<Value> = None;
    let mut spread: Option<(Spread, String)> = None;
    let mut wall = String::new();
    if let Some(fleet) = model.fleet.as_mut() {
        wall.clone_from(&fleet.at.address);
        ui.horizontal_wrapped(|ui| {
            if ui.button(FEWER).clicked() {
                fleet.spread = fleet.spread.saturating_sub(1).max(LEAST);
            }
            ui.label(fleet.spread.to_string());
            if ui.button(MORE).clicked() {
                fleet.spread = fleet.spread.saturating_add(1);
            }
            ui.add(
                egui::TextEdit::singleline(&mut fleet.goal)
                    .id(egui::Id::new(crate::ui::keys::FAN_GOAL_ID))
                    .desired_width(WIDTH)
                    .hint_text(GOAL_HINT),
            );
            ui.add(
                egui::TextEdit::singleline(&mut fleet.summary)
                    .id(egui::Id::new(crate::ui::keys::SUMMARY_ID))
                    .desired_width(WIDTH)
                    .hint_text(SUMMARY_HINT),
            );
        });
        for row in rows {
            ui.separator();
            ui.label(super::changed(row));
            for file in &row.files {
                ui.colored_label(
                    theme::tone_ink(&crate::reply::convs::Tone::Weak),
                    super::churn(file),
                );
            }
            match &row.handle {
                Some(handle) => {
                    if let Some(fired) = candidate(ui, fleet, row, handle) {
                        act = Some(fired);
                    }
                }
                None => {
                    if let Some(fired) = claim(ui, fleet, row) {
                        spread = Some(fired);
                    }
                }
            }
        }
    }
    if let Some(envelope) = act {
        model.post_candidate(envelope);
    }
    if let Some((over, goal)) = spread {
        model.stage_spread(&wall, over, &goal);
    }
}

/// **A candidate's two acts**, both addressed by the row's own obligation and
/// handle — never by a box, because a typed handle is a chance to name one
/// that is not on the glass.
fn candidate(
    ui: &mut egui::Ui,
    fleet: &crate::ui::model::Fleet,
    row: &Diff,
    handle: &str,
) -> Option<Value> {
    let mut fired = None;
    ui.horizontal_wrapped(|ui| {
        let summary = fleet.summary.trim();
        let take = ui.add_enabled(!summary.is_empty(), egui::Button::new(ACCEPT));
        crate::ui::act::tag(&take, &[crate::verbs::DELIVER]);
        if take.clicked() {
            fired = Some(crate::verbs::deliver(
                row.ball_id.clone(),
                row.project.clone(),
                handle.to_owned(),
                summary.to_owned(),
            ));
        }
        let drop = ui.button(RELEASE);
        crate::ui::act::tag(&drop, &[crate::verbs::RETIRE]);
        if drop.clicked() {
            fired = Some(crate::verbs::retire(
                row.ball_id.clone(),
                row.project.clone(),
                handle.to_owned(),
            ));
        }
    });
    fired
}

/// **The claim's one act**: spread its delivery obligation over n attempts.
fn claim(
    ui: &mut egui::Ui,
    fleet: &crate::ui::model::Fleet,
    row: &Diff,
) -> Option<(Spread, String)> {
    let mut fired = None;
    ui.horizontal_wrapped(|ui| {
        let goal = fleet.goal.trim();
        let over = ui.add_enabled(
            !goal.is_empty(),
            egui::Button::new(format!("{SPREAD} {}", fleet.spread)),
        );
        crate::ui::act::tag(&over, &[crate::verbs::PREPARE, crate::verbs::FAN]);
        if over.clicked() {
            fired = Some((
                Spread {
                    ball: row.ball_id.clone(),
                    project: row.project.clone(),
                    n: fleet.spread,
                },
                goal.to_owned(),
            ));
        }
    });
    fired
}

#[cfg(test)]
mod tests;
