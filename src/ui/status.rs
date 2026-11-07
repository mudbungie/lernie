//! **The status bar**: the window's bottom edge across its whole width,
//! under the bar as well as the column (DESIGN §4.48 items 1 and 4,
//! re-homing §4.42 item 11's rule; `docs/STYLE.md` §2).
//!
//! Left: `— <provider> | <priority> | <state> · <age> | <tokens> | <spend> |
//! <context> — <name>`, the conversation's line. The workspace is the bar's
//! foot (DESIGN §4.44), and the model and the effort are pills in the box
//! (`crate::ui::composer::pills`). The two selectors open a dropup over
//! their segment ([`dropup`]); the rest are the conversation's
//! **diagnostics** in weak ink ([`diagnostics`]) — the state slot first,
//! where the notice stands while there is one. The name is the switcher's
//! door (bl-4e55). Right: the seat's facts ([`seat`]) — the dial sentence,
//! the waiting count as the queue's door, the version.
//!
//! **A notice too long for the line is never cut** (bl-7e3e; bl-3d0f ruled
//! why: the half a cut sentence loses is the remedy) **and never costs a
//! fact**: where it does not fit beside every run, the name and the right
//! end, it stands whole on its own lines directly above the line, wrapped,
//! in its ink, until Esc; and the slot says what it says with no notice.
//! The dial sentence keeps the same rule: inline at the right end where the
//! provider stays whole beside it, else whole on its own lines above.
//!
//! **What runs out of width drops from the right** — the diagnostics, then
//! priority — and the provider never drops: it elides inside what is left,
//! because a line that lost it would say everything but what you are
//! talking to.

use crate::ui::slash::Row;
use crate::ui::{Model, Selector, theme};

/// The conversation's facts after the selectors.
pub mod diagnostics;
/// A selector's list, floating over its segment.
pub mod dropup;
/// What fits the line: the widths, the cut and the drop.
mod fit;
/// The seat's facts at the right end.
pub mod seat;
/// The four selectors, and the workspace's run the bar's foot paints.
pub mod selectors;
/// The seat-wide waiting count, as the queue's door.
pub mod waiting;

pub use diagnostics::{READY, state};
pub use fit::{elide, fits, fitting, width};
pub use selectors::NO_MODEL;

/// The glyph the rule opens with, and the one that parts the facts from the
/// name.
pub const LEAD: &str = theme::glyph::LEAD;
/// What parts one fact from the next.
pub const SEP: &str = theme::glyph::SEP;
/// The right end where nothing is aimed.
pub const SERVERS: &str = "servers";

/// **What a run opens**: a completion-menu row fired by its word, or a
/// selector's dropup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Door {
    Row(Row),
    Drop(Selector),
}

/// One run of the rule: its words, its ink, what it opens, and the small
/// mark beside it — a selector's affordance, or its reload and its check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub text: String,
    pub ink: egui::Color32,
    pub door: Option<Door>,
    pub mark: Option<(&'static str, egui::Color32)>,
}

/// A run that opens nothing.
pub fn run(text: impl Into<String>, ink: egui::Color32) -> Run {
    Run {
        text: text.into(),
        ink,
        door: None,
        mark: None,
    }
}

/// **The menu row `word` names, where this seat offers it right now** — the
/// one answer to whether a run is a door, so a run opens nothing the menu
/// would not.
pub fn door(model: &Model, word: &str) -> Option<Row> {
    crate::ui::slash::rows(model)
        .into_iter()
        .find(|row| row.word == word)
}

/// The selected conversation's own row, where the list holds one.
pub fn selected(model: &Model) -> Option<crate::reply::convs::ConvRow> {
    let id = model.conversation.as_ref()?;
    model.rows().into_iter().find(|row| row.root_id == *id)
}

/// **The right end**: the conversation, the workspace, or the window — and the
/// door to the switcher, wherever it stands.
pub fn name(model: &Model) -> Run {
    let text = match (selected(model), &model.aim) {
        (Some(row), _) => row.display,
        (None, Some(aim)) => aim.address.clone(),
        (None, None) => SERVERS.to_owned(),
    };
    Run {
        door: door(model, crate::ui::switcher::WORD).map(Door::Row),
        ..run(text, theme::INK)
    }
}

/// **The rule's runs between the leads**, in order, and how many lead it
/// that never drop.
pub fn runs(model: &Model) -> (Vec<Run>, usize) {
    let (mut out, kept) = selectors::selectors(model);
    out.extend(diagnostics::diagnostics(model));
    (out, kept)
}

/// **What a door does**, one sentence (DESIGN §4.46 item 2): a menu row's
/// is its own summary, a selector's is the selector's.
fn help(door: Door) -> String {
    match door {
        Door::Row(row) => row.summary.to_owned(),
        // **The model's selector cites Ctrl+O** (§4.46 item 3), the one
        // selector a chord opens.
        Door::Drop(selector) => crate::ui::keys::cited(
            selectors::help(selector),
            match selector {
                crate::ui::Selector::Model => &[crate::ui::keys::Act::Model],
                _ => &[],
            },
        ),
    }
}

/// **Paint one run**, answering whether it was clicked and where it stood —
/// on the rule, and at the bar's foot (`crate::ui::bar`).
pub(crate) fn paint(ui: &mut egui::Ui, run: &Run, words: &str) -> (bool, egui::Rect) {
    let Some(door) = run.door else {
        let seat = ui.label(egui::RichText::new(words).small().color(run.ink));
        return (false, seat.rect);
    };
    // **The mark is in the door** (DESIGN §4.44 item 5): one seat, one stop
    // in the Tab order, the mark tinting and opening with the words.
    let seat = match run.mark {
        Some(mark) => theme::paint::link_marked(ui, words, run.ink, mark, &help(door)),
        None => theme::paint::link(ui, words, run.ink, &help(door)),
    };
    crate::ui::act::tag(&seat, selectors::ops(door));
    (seat.clicked(), seat.rect)
}

/// Paint the status bar, and open what a click on one of its doors opens.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let (mut runs, kept) = runs(model);
    let name = name(model);
    let room = ui.available_width();
    let named = elide(ui, &name.text, room / 4.0);
    let gap = ui.spacing().item_spacing.x;
    let reserve = width(ui, &named) + seat::width(ui, model) + gap;
    let dial = seat::dial(model);
    let beside = dial
        .as_ref()
        .map(|said| width(ui, said) + gap)
        .filter(|w| fits(ui, &runs, kept, reserve + w, room));
    let faint = |ui: &mut egui::Ui, text: &str| {
        ui.label(egui::RichText::new(text).small().color(theme::INK_FAINT));
    };
    let wrapped = |ui: &mut egui::Ui, said: &str, ink| {
        let words = egui::RichText::new(said).small().color(ink);
        ui.add(egui::Label::new(words).wrap());
    };
    if let (Some(said), None) = (&dial, beside) {
        wrapped(ui, said, theme::accent(theme::State::Error));
    }
    let reserve = reserve + beside.unwrap_or(0.0);
    let noticed = model.notice.as_ref().map(|_| state(model));
    if let Some(said) = noticed
        && fitting(ui, &runs, kept, reserve, room).0 < runs.len()
    {
        wrapped(ui, &said.text, said.ink);
        if let Some(slot) = runs.iter_mut().find(|run| **run == said) {
            *slot = diagnostics::calm(model);
        }
    }
    let (shown, cut) = fitting(ui, &runs, kept, reserve, room);
    let mut fired = None;
    let mut asked = false;
    ui.horizontal(|ui| {
        faint(ui, LEAD);
        for (at, run) in runs.iter().take(shown).enumerate() {
            if at > 0 {
                faint(ui, SEP);
            }
            let words = if at < kept {
                elide(ui, &run.text, cut)
            } else {
                run.text.clone()
            };
            let (clicked, rect) = paint(ui, run, &words);
            if let Some(Door::Drop(selector)) = run.door {
                dropup::anchor(ui, model, selector, rect);
            }
            fired = clicked.then_some(run.door).flatten().or(fired);
        }
        faint(ui, LEAD);
        fired = paint(ui, &name, &named)
            .0
            .then_some(name.door)
            .flatten()
            .or(fired);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            asked = seat::render(ui, model, beside.and(dial.as_deref()));
        });
    });
    if asked {
        crate::ui::slash::fire(model, crate::ui::queue::WORD, "");
    }
    match fired {
        Some(Door::Row(row)) => {
            crate::ui::slash::fire(model, row.word, "");
        }
        Some(Door::Drop(selector)) => dropup::toggle(ui, model, selector),
        None => {}
    }
}

#[cfg(test)]
mod tests;
