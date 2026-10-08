//! **What a reviewer has staged for this wall's config, on the config pane**
//! (bl-a1d6; yog's `docs/REMOTE.md` §9.22; DESIGN §4.30).
//!
//! Split from [`super`] at the design-time budget on a real seam: [`super`] is
//! the config that GOVERNS — the lineages and one file's bytes — and this is a
//! CANDIDATE, a patch no lineage points at until somebody says so. It hangs on
//! the same pane because it is the same subject one branch away, not a pane of
//! its own.
//!
//! # The standing is the engine's word
//!
//! A row says `fresh` or `stale` exactly as the engine read it in the pass
//! that read the lineages (REMOTE §9.4); nothing here infers one off an empty
//! lineage list.
//!
//! # The whole is painted verbatim
//!
//! A diff a seat reformatted is a diff nobody can apply, so the message and
//! the patch are one monospace run that is never wrapped — a long line
//! scrolls sideways rather than reflowing.
//!
//! # The two verdicts are §4.20's, armed by the proposal's own id
//!
//! The opened row states its subject before it offers the act, the way out
//! comes first, and both verdicts are dark until the id is typed back — an
//! accept moves a lineage beyond the row on screen, and a reject throws a
//! reviewer's work away.

use crate::reply::proposals::Proposal;
use crate::ui::{Model, Proposing, keys, theme};
use crate::verbs::proposals::VERDICTS;

/// The listing's own heading.
pub const HEADING: &str = "staged proposals";
/// What it says for a wall nobody has been answered about yet.
pub const NOT_ANSWERED: &str = "waiting to hear what a reviewer has staged";
/// What it says for a wall with nothing staged — a fact, not a wait.
pub const NOTHING_STAGED: &str = "nothing is staged for this wall";
/// What it says under a named row that has not answered whole yet.
pub const NOT_WHOLE: &str = "waiting to hear this proposal whole";
/// What it says for a named row the engine's listing no longer carries.
pub const GONE: &str = "no longer staged — it was settled, or its branch went";
/// The way out of a named row. It says what it leaves: the proposal stays
/// staged and nothing is settled.
pub const CLOSE: &str = "leave it staged";
/// What the arming box asks for.
pub const ARM: &str = "the proposal's own id";
/// What it says while the box does not hold that id.
pub const NOT_ARMED: &str = "not armed — type the id above, exactly";
/// What it says once it does.
pub const ARMED: &str = "armed";
/// What it says once a verdict has been asked for.
pub const ASKED: &str = "asked — waiting for the engine";
/// What each verdict does, said before it is offered.
pub const VERDICTS_DO: &str = "accept moves the lineage onto it and every conversation on \
                               that lineage picks it up at its next step; reject deletes \
                               the staging branch";

/// How wide the arming box is — the unmaking's, for the same reason.
const ARM_WIDTH: f32 = 200.0;

/// Paint the listing, and the named row whole beneath it.
pub(super) fn render(ui: &mut egui::Ui, model: &mut Model) {
    ui.label(HEADING);
    let named = model.named_proposal();
    match model.proposals.clone() {
        None => {
            ui.label(NOT_ANSWERED);
        }
        Some(staged) if staged.rows.is_empty() => {
            ui.label(NOTHING_STAGED);
        }
        Some(staged) => {
            for row in &staged.rows {
                listed(ui, model, row, named.as_ref());
            }
        }
    }
    if let Some(named) = named {
        ui.separator();
        whole(ui, model, &named);
    }
}

/// **One row's line**: the engine's standing, the lineage it would move, its
/// diffstat and the reviewer's subject — a pure function, so the row is read
/// back as a value.
pub fn line(row: &Proposal) -> String {
    format!(
        "{} — moves {} — {} — {}",
        row.standing(),
        row.moves(),
        row.diffstat,
        row.subject
    )
}

/// One row of the listing: the control that names it, then what it is.
fn listed(ui: &mut egui::Ui, model: &mut Model, row: &Proposal, named: Option<&Proposing>) {
    ui.horizontal_wrapped(|ui| {
        let chosen = named.is_some_and(|held| held.id == row.id);
        let control = ui.selectable_label(chosen, &row.id);
        crate::ui::act::tag(&control, &[crate::verbs::PROPOSALS.word]);
        if control.clicked() {
            model.name_proposal(&row.id);
        }
    });
    ui.colored_label(theme::tone_ink(&crate::reply::convs::Tone::Weak), line(row));
}

/// **The named row whole**: its message and diff, verbatim, then the arming
/// and the two verdicts.
fn whole(ui: &mut egui::Ui, model: &mut Model, named: &Proposing) {
    let staged = model.staged_proposal(&named.id);
    ui.label(&named.id);
    match (
        &staged,
        model.proposals.as_ref().and_then(|held| held.whole.clone()),
    ) {
        (None, _) => {
            ui.colored_label(theme::NOTICE, GONE);
        }
        (Some(_), None) => {
            ui.label(NOT_WHOLE);
        }
        (Some(_), Some(text)) => verbatim(ui, &named.id, &text),
    }
    ui.colored_label(theme::NOTICE, VERDICTS_DO);
    if let Some(typed) = model.verdict_box() {
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::TextEdit::singleline(typed)
                    .id(egui::Id::new(keys::VERDICT_ID))
                    .desired_width(ARM_WIDTH)
                    .hint_text(ARM),
            );
        });
    }
    let live = staged.is_some() && model.named_proposal().is_some_and(|held| held.armed());
    ui.horizontal_wrapped(|ui| {
        // **The way out is first**, and therefore first in the tab order too.
        if ui.button(CLOSE).clicked() {
            model.unname_proposal();
        }
        for verdict in VERDICTS {
            let spend = ui.add_enabled(live, egui::Button::new(verdict));
            crate::ui::act::tag(&spend, &[crate::verbs::PROPOSAL.word]);
            if spend.clicked() {
                model.settle_proposal(verdict);
            }
        }
    });
    if let Some(held) = model.named_proposal() {
        ui.colored_label(theme::NOTICE, said(&held));
    }
}

/// **The message and diff, as the engine wrote them** — one monospace run,
/// never wrapped, scrolling sideways where a line is longer than the pane.
fn verbatim(ui: &mut egui::Ui, id: &str, text: &str) {
    egui::ScrollArea::horizontal().id_salt(id).show(ui, |ui| {
        ui.add(
            egui::Label::new(egui::RichText::new(text).monospace())
                .wrap_mode(egui::TextWrapMode::Extend),
        );
    });
}

/// **What the row says about its own arming**, as a pure function of the
/// state — the unmaking's three sentences, one noun over.
pub fn said(held: &Proposing) -> String {
    if held.posted {
        ASKED.to_owned()
    } else if held.armed() {
        ARMED.to_owned()
    } else {
        NOT_ARMED.to_owned()
    }
}

#[cfg(test)]
mod tests;
