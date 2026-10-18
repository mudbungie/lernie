//! **The status rule**: the conversation's own line, under the prompt at the
//! window's bottom edge (DESIGN §4.42, item 11; §4.45 item 3 put it under
//! the box; `docs/STYLE.md` §2).
//!
//! Left to right: `— <provider> | <model> | <effort> | <priority> |
//! <state> · <age> | <tokens> | <spend> | <context> — <name>`. The workspace
//! is not on it: it is the bar's foot (DESIGN §4.44, `crate::ui::bar`).
//! The first four are **selectors** ([`selectors`]): a click opens a dropup
//! over the segment listing what this seat already holds and posts the read
//! that refreshes it ([`dropup`]). The rest are the conversation's
//! **diagnostics** in weak ink ([`diagnostics`]) — the state slot first,
//! where the notice stands while there is one. The name at the right end is
//! the switcher's door (bl-4e55). Nothing seat-wide stands here: the waiting
//! and server counts are the seat panel's, the switcher's and `/attention`'s.
//!
//! **A notice too long for the line is never cut** (bl-7e3e; bl-3d0f ruled
//! why: the half a cut sentence loses is the remedy) **and never costs a
//! fact**: where it does not fit beside every run and the name, it stands
//! whole on its own lines directly above the rule — under the box, since the
//! rule is — wrapped, in its ink, until Esc; and the slot says what it says
//! with no notice.
//!
//! **What runs out of width drops from the right** — the diagnostics, then
//! priority, then effort — and the provider and model never drop:
//! they elide inside an equal share of what is left, because a rule that lost
//! them would say everything but what you are talking to.

use crate::ui::slash::Row;
use crate::ui::{Model, Selector, theme};

/// The conversation's facts after the selectors.
pub mod diagnostics;
/// A selector's list, floating over its segment.
pub mod dropup;
/// The four selectors, and the workspace's run the bar's foot paints.
pub mod selectors;
/// The seat-wide waiting count, as the queue's door off the rule.
pub mod waiting;

pub use diagnostics::{READY, state};
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

/// The width `text` takes at the rule's size.
pub fn width(ui: &egui::Ui, text: &str) -> f32 {
    let font = egui::TextStyle::Small.resolve(ui.style());
    ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(text.to_owned(), font, theme::INK)
            .size()
            .x
    })
}

/// **`text` cut to `room`**, an ellipsis where it was cut.
pub fn elide(ui: &egui::Ui, text: &str, room: f32) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    let mut said = text.to_owned();
    while width(ui, &said) > room && !chars.is_empty() {
        chars.pop();
        said = format!(
            "{}{}",
            chars.iter().collect::<String>(),
            theme::glyph::ELIDED
        );
    }
    said
}

/// One run's width with the mark inside it and the separator before it.
fn cost(ui: &egui::Ui, run: &Run) -> f32 {
    let gap = ui.spacing().item_spacing.x;
    let mark = run
        .mark
        .map_or(0.0, |(glyph, _)| width(ui, &format!(" {glyph}")));
    width(ui, SEP) + 2.0 * gap + width(ui, &run.text) + mark
}

/// **How the runs fit `room` beside a name `name` wide**: how many are shown
/// (the `kept` never drop; the rest drop from the right), and the width each
/// kept run's words are cut to where even they do not fit.
pub fn fitting(ui: &egui::Ui, runs: &[Run], kept: usize, name: f32, room: f32) -> (usize, f32) {
    let gap = ui.spacing().item_spacing.x;
    let mut used = width(ui, LEAD) * 2.0 + name + 4.0 * gap;
    let core: f32 = runs.iter().take(kept).map(|run| cost(ui, run)).sum();
    if used + core > room {
        // **Water-filled**: a short run keeps its words, and what is left is
        // shared by the rest — one cut every longer run is held to.
        let mut words: Vec<f32> = runs.iter().take(kept).map(|r| width(ui, &r.text)).collect();
        words.sort_by(f32::total_cmp);
        let mut left = room - used - (core - words.iter().sum::<f32>());
        let mut n: f32 = words.iter().map(|_| 1.0).sum();
        for said in words {
            if said > left / n {
                break;
            }
            left -= said;
            n -= 1.0;
        }
        return (kept, (left / n).max(0.0));
    }
    used += core;
    let mut shown = kept;
    for run in runs.iter().skip(kept) {
        used += cost(ui, run);
        if used > room {
            break;
        }
        shown += 1;
    }
    (shown, f32::INFINITY)
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

/// Paint the rule, and open what a click on one of its doors opens.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let (mut runs, kept) = runs(model);
    let name = name(model);
    let room = ui.available_width();
    let named = elide(ui, &name.text, room / 4.0);
    let noticed = model.notice.as_ref().map(|_| state(model));
    if let Some(said) = noticed
        && fitting(ui, &runs, kept, width(ui, &named), room).0 < runs.len()
    {
        let words = egui::RichText::new(&said.text).small().color(said.ink);
        ui.add(egui::Label::new(words).wrap());
        if let Some(slot) = runs.iter_mut().find(|run| **run == said) {
            *slot = diagnostics::calm(model);
        }
    }
    let (shown, cut) = fitting(ui, &runs, kept, width(ui, &named), room);
    let faint = |ui: &mut egui::Ui, text: &str| {
        ui.label(egui::RichText::new(text).small().color(theme::INK_FAINT));
    };
    let mut fired = None;
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
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            fired = paint(ui, &name, &named)
                .0
                .then_some(name.door)
                .flatten()
                .or(fired);
            faint(ui, LEAD);
        });
    });
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
