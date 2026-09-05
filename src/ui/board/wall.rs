//! **The aimed wall's half of the ball pane** (bl-d2af): what this wall holds,
//! what each of them has cost, and the branch its `bl` verbs track on.
//!
//! Two ops, both naming a workspace, so both are the tuning pane's shape
//! rather than the board's: they stand while this pane is open and are retired
//! when the aim moves (`crate::ui::model::board`). It is a section of the
//! board rather than a pane of its own because *the board* and *what this wall
//! holds* are one question asked at two widths, and two panes would put them
//! on two screens.
//!
//! **Its rows are where three of the pane's five acts hang** (bl-f7ae): every
//! field an act on an existing ball carries — the project its verbs run in,
//! the id, and the wall whose name they stamp — is on this section's own row
//! or on the wall it is about, so the block a row opens ([`acts`]) derives
//! nothing.
//!
//! **Every emptiness gets its own sentence**, which is the rule the records
//! pane states and this section has four of: no wall aimed at, a wall not yet
//! answered about, a wall that answered and holds nothing, and a wall whose
//! branch is the only thing that came back. Painting one sentence over all
//! four would say *this wall holds nothing* about a wall nobody has asked.

use crate::reply::balls::BoundBall;
use crate::ui::{Model, theme};

use super::acts;

/// What it says with no wall aimed at.
pub const UNAIMED: &str = "aim at a workspace to see the balls it holds";
/// What it says about a wall nobody has been answered about yet.
pub const NOT_ANSWERED: &str = "waiting to hear what this workspace holds";
/// What it says about a wall that answered and holds none.
pub const NOTHING: &str = "this workspace holds no balls";

/// Paint the section, and take the clicks that open the authoring block on
/// one of its rows.
///
/// **Every control here opens a block and crosses no wire**, so none of them
/// carries a parity token — the division `enroll a box…` keeps with `mint`
/// (§4.20). The acts themselves are [`acts`]'s.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    ui.separator();
    let Some(aim) = model.aim.clone() else {
        ui.label(UNAIMED);
        return;
    };
    ui.label(aim.address);
    if let Some(branch) = &model.marks {
        ui.colored_label(
            theme::tone_ink(&crate::reply::convs::Tone::Weak),
            tracking(branch),
        );
    }
    // **Filing is offered whether or not the wall holds anything**, because a
    // ball that does not exist yet is not one of this wall's rows: the wall is
    // only what supplies the name it is stamped with.
    if ui.button(acts::FILE).clicked() {
        model.begin_filing();
    }
    let Some(rows) = model.holding.clone() else {
        ui.label(NOT_ANSWERED);
        return;
    };
    if rows.is_empty() {
        ui.label(NOTHING);
        return;
    }
    for row in &rows {
        ui.label(bound(row));
        ui.colored_label(
            theme::tone_ink(&crate::reply::convs::Tone::Weak),
            super::cost(&row.spend),
        );
        if ui.button(acts::ACT).clicked() {
            model.begin_amending(row);
        }
    }
}

/// **The branch this wall tracks its tasks on.** Every agent tracks in a task
/// space of its own so two agents' churn never collides, and `balls/tasks` is
/// the project's shared board — which is why the branch is worth a line: it is
/// how an operator tells the one from the other.
pub fn tracking(branch: &str) -> String {
    format!("tracking tasks on {branch}")
}

/// **One ball this wall holds**: what it is, where its verbs run, and the name
/// they stamp. The badge rides where the engine wrote one and is absent —
/// never blank — where the state needs none.
pub fn bound(row: &BoundBall) -> String {
    let mut said = vec![row.id.clone(), format!("[{}]", row.state)];
    if let Some(badge) = &row.badge {
        said.push(badge.clone());
    }
    said.push(format!("in {} as {}", row.project, row.owner));
    said.join("  ")
}

#[cfg(test)]
mod tests;
