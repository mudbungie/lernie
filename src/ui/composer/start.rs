//! **The start half of the composer**: where a conversation is begun.
//!
//! It is a **mode of the composer and not a pane beside it**, and that is this
//! ball's one layout decision. A workspace with no conversation selected already had
//! a box that refused — *pick a workspace and a conversation to say anything* —
//! and that refusal was the start's own case wearing a sentence: there is
//! nothing to say to, so the thing to do is begin one. A second box would be
//! the same box twice, each with its own Enter, on a face that is four panes
//! wide; upstream states the rule as *one box, one Enter* and it is worth more
//! here than there.
//!
//! So the composer is one control with two subjects, decided by what is
//! selected and by nothing else:
//!
//! - no workspace aimed at → nothing to say and nothing to begin;
//! - a workspace, no conversation → **this**, the start;
//! - a workspace and a conversation → the deposit.
//!
//! **A start in flight paints its sentence instead of the box**, which is how a
//! second start is refused: not by a disarmed control, but by there being no
//! control. The receipt is the one state that paints the sentence *and* the box
//! — the minted name stays readable while the operator begins the next one.
//!
//! # One composer, in both of its modes (DESIGN §4.39)
//!
//! *Being* the same control and *looking* like it are two things, and until
//! bl-b3c3 this mode only had the first: it painted a one-line
//! [`crate::ui::theme::paint::field`] and a button beside it while the deposit
//! painted [`crate::ui::theme::paint::composer`] — the field
//! [`crate::ui::theme::COMPOSER_ROWS`] tall with the act inside it — so the
//! pane's focal element shrank to a bar whenever nothing was selected. It is
//! now laid through the same shape at the same rows, with [`START`] inside the
//! field where the deposit says `send`, and under it the same offers row
//! carrying what applies to a start ([`super::offers::starting`]).
//!
//! **What the row does NOT carry is the rule, not an omission.** `records…`,
//! `interrupt`, `stop`, `nudge` and the `…` strip are acts on a conversation's
//! turn or on a conversation as an object, and there is neither — so they are
//! absent rather than greyed, by the offers row's own rule that only what is
//! offered is on the row.

use crate::ui::{Aim, Fill, Model};

/// **What the box asks for, naming the workspace the Enter begins on** (§4.42
/// item 3; bl-9031). A start's goal is what the conversation is *for*, which
/// is a different question from what to say to one that exists; and with two
/// workspaces on one server the box is the one place the operator is looking when
/// the Enter decides where the conversation lands. While `/plan` has said the
/// next start is a plan, the hint's tail says so instead.
pub fn goal(workspace: &str, plan: bool) -> String {
    let tail = if plan {
        "begun as a plan"
    } else {
        "Enter begins it"
    };
    format!("what this conversation on {workspace} is for — {tail}")
}

/// **What the box says on a workspace whose worker reaches no model yet**
/// (bl-7a77): the worker role's provider row holds no credential, so a goal
/// typed now would be refused — the hint names the two doors and the pick
/// before the operator types one. It stands in the goal's place, not after
/// it: the goal's question is the wrong one to ask first.
pub const UNSIGNED: &str = "no provider yet — /login or /config, then pick one on the rule";
/// What the box does while it is the start.
pub const START_HELP: &str =
    "the first message of a new conversation on this workspace; Enter begins it";

/// **Whether the aimed workspace's worker reaches no model yet**: its provider's
/// row in the provider table is bare (`ProviderRow::bare`). Both reads stand
/// on the aim; where either has not answered, nothing is claimed.
pub fn unsigned(model: &Model) -> bool {
    let Some(worker) = model.worker() else {
        return false;
    };
    model
        .providers
        .iter()
        .flatten()
        .any(|row| row.name == worker.provider && row.bare())
}

/// **The start's own sentence, with the doors it names** (bl-7a77): a
/// refused start says the server's remedy — *sign in first … /config
/// brazen* — and each menu row it names stands after it as a link in the
/// brand that fires that row, as the notice's doors do on the rule.
fn said(ui: &mut egui::Ui, model: &mut Model, line: &str) {
    ui.label(line);
    let rows = crate::ui::status::diagnostics::named(model, line);
    if rows.is_empty() {
        return;
    }
    let mut fired = None;
    ui.horizontal(|ui| {
        for row in rows {
            let word = format!("{}{}", crate::ui::slash::LEAD, row.word);
            let seat =
                crate::ui::theme::paint::link(ui, &word, crate::ui::theme::BRAND, row.summary);
            crate::ui::act::tag(&seat, row.ops);
            if seat.clicked() {
                fired = Some(row.word);
            }
        }
    });
    if let Some(word) = fired {
        crate::ui::slash::fire(model, word, "");
    }
}

/// Paint the start composer and take what it was given.
pub fn render(ui: &mut egui::Ui, model: &mut Model, aim: &Aim) {
    // **Taken before anything is painted**, so a start in flight — which
    // paints its sentence and no box — spends the request rather than holding
    // it for whichever frame paints a box next (`crate::ui::model::fill`).
    let wanted = model.filling() == Some(Fill::Goal);
    if let Some(held) = model.start.clone() {
        said(ui, model, &held.line());
        if held.outstanding() {
            return;
        }
    }
    // **The same shape as the deposit, at the same rows** (§4.39) — including
    // the rows the operator dragged the panel's top edge to, because one box
    // in two modes is one box: the act
    // stands inside the field rather than beside it, and its word is the only
    // thing that differs. The start has nothing to glow about — a glow is the
    // selected conversation asking, and there is no conversation.
    //
    // **Both halves of the start ride this one control** (`crate::ui::act`):
    // the click composes `prepare`, and the `prompt` that finishes it is fired
    // by the frame when the server's reply lands (`crate::ui::model::start`),
    // from no widget at all. A walk over the accessibility tree can only ever
    // see the control that begins the pair, so both tokens ride it.
    let slashed = crate::ui::slash::parsed(&model.draft).is_some();
    let said = if unsigned(model) {
        UNSIGNED.to_owned()
    } else {
        super::hint(
            ui,
            &goal(&aim.address, model.plan),
            &crate::ui::chat::panel::WAYS,
        )
    };
    let entry = crate::ui::theme::paint::prompt(
        ui,
        egui::Id::new(crate::ui::keys::BOX_ID),
        &mut model.draft,
        &said,
        None,
        START_HELP,
    );
    crate::ui::act::tag(&entry, &[crate::verbs::PREPARE, crate::verbs::PROMPT]);
    // **The `+` on a server's row lands the caret here** (§4.39): the control
    // is on the other column and the box is this one, so what crosses between
    // them is a request the frame that paints the box answers.
    if wanted {
        entry.request_focus();
    }
    // Enter begins it, as it always did — through the deposit's own read, now
    // that the box is the deposit's own box (`super::entered`).
    if slashed {
        return;
    }
    if super::entered(ui, &entry) {
        model.stage(&aim.address);
    }
}

#[cfg(test)]
mod tests;
