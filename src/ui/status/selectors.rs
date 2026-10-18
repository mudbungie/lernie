//! **The rule's four selectors** (DESIGN §4.42, item 11, as §4.44 amends
//! it): provider, model, effort and priority, left to right — what this
//! conversation talks to, each a dropup. The workspace left the rule for the
//! bar's foot (`crate::ui::bar`), because a workspace is not a fact of a
//! conversation; its run is still spelled here ([`workspace`]), once.
//!
//! The four are the **worker role's** row off the
//! `roles` read, which stands on the aim (`crate::offframe::asker::workspace`).
//! Effort and priority stand only where the worker's provider row takes them
//! — or where the providers read has not said, because absence is not
//! evidence (`crate::ui::model::tuning::offers`' own rule).

use super::{Door, Run, run};
use crate::ui::{Model, Reload, Selector, theme};

/// The workspace's words with nothing aimed — `⏶` stands beside them.
pub const NO_WORKSPACE: &str = "workspace";
/// The provider slot where the worker's row is not in hand.
pub const NO_PROVIDER: &str = "provider…";
/// The model slot where neither the worker's row nor the transcript names
/// one (bl-0966: it names the control and claims nothing about the workspace).
pub const NO_MODEL: &str = "model…";
/// What heads the effort and the priority segment's words.
pub const EFFORT: &str = "effort";
pub const PRIORITY: &str = "priority";

/// **The ops a door's click spends** — a menu row's, or the read a
/// selector's dropup posts.
pub(crate) fn ops(door: Door) -> &'static [&'static str] {
    use crate::verbs;
    match door {
        Door::Row(row) => row.ops,
        Door::Drop(Selector::Workspace) => &[verbs::WORKSPACES.word],
        Door::Drop(Selector::Model) => &[verbs::MODELS.word],
        Door::Drop(_) => &[verbs::PROVIDERS.word],
    }
}

/// **What each selector changes** (DESIGN §4.46 item 2).
pub(crate) fn help(selector: Selector) -> &'static str {
    match selector {
        Selector::Workspace => "picks the workspace the window is aimed at",
        Selector::Provider => "picks the provider this workspace's worker runs on",
        Selector::Model => "picks the model this workspace's worker runs on",
        Selector::Effort => "picks how hard the worker's model thinks",
        Selector::Priority => {
            "puts the worker's calls on the provider's priority lane, or takes them off"
        }
    }
}

/// **The mark beside a selector**: its affordance at rest, and while its
/// dropup is open the reload in the annotation accent until the read
/// answers, then the check in the attention accent.
fn mark(model: &Model, selector: Selector) -> (&'static str, egui::Color32) {
    match model
        .dropup
        .as_ref()
        .filter(|open| open.selector == selector)
    {
        Some(open) if open.reload == Some(Reload::Asked) => (
            theme::glyph::RELOAD,
            theme::accent(theme::State::Annotation),
        ),
        Some(open) if open.reload == Some(Reload::Landed) => {
            (theme::glyph::DONE, theme::accent(theme::State::Attention))
        }
        _ => (theme::glyph::DROP, theme::INK_FAINT),
    }
}

/// One selector's run: its words in body ink, or a placeholder in weak ink.
fn selector(model: &Model, selector: Selector, said: Option<String>, empty: &str) -> Run {
    let (text, ink) = said.map_or((empty.to_owned(), theme::INK_WEAK), |said| {
        (said, theme::INK)
    });
    Run {
        door: Some(Door::Drop(selector)),
        mark: Some(mark(model, selector)),
        ..run(text, ink)
    }
}

/// The model that last answered on the transcript, where one did.
fn answered_by(model: &Model) -> Option<String> {
    model
        .transcript
        .entries
        .iter()
        .rev()
        .find_map(|entry| match &entry.kind {
            crate::reply::transcript::EntryKind::Model { model_id, .. } => Some(model_id.clone()),
            _ => None,
        })
}

/// **The workspace's run** — the aimed workspace's name, or [`NO_WORKSPACE`] —
/// which the bar's foot paints and the rule does not (DESIGN §4.44 item 3).
pub(crate) fn workspace(model: &Model) -> Run {
    let workspace = model.aim.as_ref().map(|aim| aim.address.clone());
    selector(model, Selector::Workspace, workspace, NO_WORKSPACE)
}

/// **The selectors the rule offers, in order**, and how many of them never
/// drop: with nothing aimed none stands; with a workspace aimed the provider and
/// the model, and effort and priority where the worker's row takes them.
pub fn selectors(model: &Model) -> (Vec<Run>, usize) {
    let mut out = Vec::new();
    if model.aim.is_none() {
        return (out, 0);
    }
    let worker = model.worker().cloned();
    out.push(selector(
        model,
        Selector::Provider,
        worker.as_ref().map(|w| w.provider.clone()),
        NO_PROVIDER,
    ));
    let said = worker
        .as_ref()
        .map(|w| w.model.clone())
        .or_else(|| answered_by(model));
    out.push(selector(model, Selector::Model, said, NO_MODEL));
    let Some(worker) = worker else {
        return (out, 2);
    };
    let row = model
        .providers
        .iter()
        .flatten()
        .find(|row| row.name == worker.provider);
    if row.is_none_or(|row| row.effort) {
        let level = crate::verbs::tuning::word(worker.effort.as_ref());
        let said = Some(format!("{EFFORT} {level}"));
        out.push(selector(model, Selector::Effort, said, EFFORT));
    }
    if row.is_none_or(|row| row.priority) {
        let said = Some(format!("{PRIORITY} {}", crate::ui::lane(worker.priority)));
        out.push(selector(model, Selector::Priority, said, PRIORITY));
    }
    (out, 2)
}
