//! **The workflow mark on the governing half** (REMOTE §9.24, bl-ed20): which
//! `workflow.yaml` governs when it is not the followed tip's, and the two acts
//! that set and clear that.
//!
//! # Here, beside the mark, because the argument is only discoverable here
//!
//! `workflow` takes a `config`, a lineage name, and §9.24 rules that what a
//! conversation can be marked to *is* `request/lineages` — the listing the
//! config pane already reads (bl-5c53). So the control is the fork's shape one
//! half over (DESIGN §4.29): one button per lineage the wall holds, each
//! carrying its own name, rather than a box that would accept a string nothing
//! on the glass vouched for. The listing stands on this pane for that reason
//! (`crate::offframe::asker::wall`), and with none answered there is nothing
//! to offer and nothing is offered.
//!
//! # Neither act is an unmaking
//!
//! DESIGN §4.20's arming is for an act that is not undone by doing the other
//! thing. A mark is a ref litany writes and deletes; the conversation, its
//! history and the marked commit all stand either way, and setting a mark
//! again is the other thing. So both are plain buttons, on `retarget`'s terms.
//!
//! # `clear` is offered only where a mark is read
//!
//! The pin pair's rule (DESIGN §4.25): a control is withheld where this seat
//! can READ that it would change nothing, and `workflow_mark: null` is that
//! reading. A mark inherited from an ancestor is still offered the clear — the
//! holder is a descent id this seat does not map to a conversation address, so
//! the engine's own answer says whose mark it was.

use serde_json::Value;

use crate::reply::governing::Mark;
use crate::ui::Model;

/// The word on the control that deletes the mark.
pub const CLEAR: &str = "clear workflow mark";

/// Paint the mark, if one governs, and the acts on it — into the governing
/// half's own wrapped row, so the half costs no line it did not before.
pub fn row(ui: &mut egui::Ui, model: &mut Model, mark: Option<&Mark>) {
    if let Some(mark) = mark {
        ui.label(mark.line());
        let clear = ui.button(CLEAR);
        crate::ui::act::tag(&clear, &[crate::verbs::CLEAR_WORKFLOW.word]);
        if clear.clicked() {
            post(model, &crate::verbs::clear_workflow);
        }
    }
    let names: Vec<String> = model
        .lineages
        .iter()
        .flatten()
        .map(|lineage| lineage.name.clone())
        .collect();
    for name in names {
        let mark = ui.button(marking(&name));
        crate::ui::act::tag(&mark, &[crate::verbs::WORKFLOW.word]);
        if mark.clicked() {
            post(model, &|workspace, agent| {
                crate::verbs::workflow(workspace, agent, name.clone())
            });
        }
    }
}

/// The word on the control that marks to `lineage`. It names the lineage it
/// carries, so two lineages never offer one label.
pub fn marking(lineage: &str) -> String {
    format!("workflow from config/{lineage}")
}

/// **Post an act on the selected conversation**, or nothing where nothing is
/// aimed at or selected — `crate::ui::model::spine`'s `post_fork` gate, one
/// act over. The gesture is a trait object rather than a generic so the gate
/// is one function, judged once, whichever act it carries.
pub fn post(model: &mut Model, gesture: &dyn Fn(String, String) -> Value) {
    let (Some(aim), Some(agent)) = (model.aim.clone(), model.conversation.clone()) else {
        return;
    };
    model
        .outbox
        .push(crate::ui::Posted::act(gesture(aim.address, agent)));
}

#[cfg(test)]
mod tests;
