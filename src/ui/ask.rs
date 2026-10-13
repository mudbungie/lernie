//! **The ask**: the selected conversation's decision, standing above the
//! prompt while it waits (DESIGN §4.42, item 8; `docs/STYLE.md` §5, *the
//! ask*).
//!
//! It is §4.19's queue entry for the one conversation on the glass, painted
//! where the answer is typed: a framed box in the attention accent between
//! the transcript and the rule, carrying what the queue's row says about it
//! — the parked call, the flag, the signals, the failure, the last words —
//! and its choices as numbered rows. The digits are quick picks, the arrows
//! move and Enter answers, all while nothing holds the keyboard
//! (`crate::ui::keys`); a click on a row is the same answer.
//!
//! **The choices are the queue's own acts and no others**: `seen`, and where
//! a call is parked the three verdicts — `pass` for this call, for this
//! conversation and for this workspace, which are the baseline's *once*,
//! *this session* and *always* on this wire's own words (REMOTE's scopes),
//! then `refuse` and `hold`. `go to it` is not among them: the operator is
//! already there. The queue pane (`/attention`) is the same rows for every
//! conversation, and the count on the rule is its door.
//!
//! **The read stands while the conversation asks.** The queue's read used to
//! stand only on its pane; this box is the second place it is spent, so it
//! stands while the selected conversation's row says it is asking
//! (`crate::state::Standing::asking`) — which is the roster's own word, read
//! before the queue has answered, so the box can be answered on the frame
//! the queue's row lands.

use crate::reply::queue::QueueRow;
use crate::ui::{Model, queue, theme};

/// The box's name.
pub const HEADING: &str = "it is asking";
/// The keys, said on the name line.
pub const HINT: &str = "a digit picks · arrows move · Enter answers";
/// Where the chosen row is kept between frames — the toolkit's memory, as
/// the menu's is.
const CHOSEN_ID: &str = "the ask's chosen row";

/// **One choice**: `seen`, or a verdict at a scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Seen,
    Verdict(&'static str, &'static str),
}

impl Choice {
    /// The words on the row, as the queue pane spells them.
    pub fn word(self) -> String {
        match self {
            Self::Seen => queue::SEEN.to_owned(),
            Self::Verdict(verdict, scope) if scope == crate::verbs::capability::CALL => {
                verdict.to_owned()
            }
            Self::Verdict(verdict, scope) => queue::wider(verdict, scope),
        }
    }

    /// The op the row fires, which is its tag.
    pub(crate) fn op(self) -> &'static str {
        match self {
            Self::Seen => crate::verbs::SEEN.word,
            Self::Verdict(..) => crate::verbs::ANSWER.word,
        }
    }
}

/// **The choices a row offers**, in the order they are numbered.
pub fn choices(row: &QueueRow) -> Vec<Choice> {
    let mut out = vec![Choice::Seen];
    if row.held.is_some() {
        for scope in crate::verbs::SCOPES {
            out.push(Choice::Verdict("pass", scope));
        }
        out.push(Choice::Verdict("refuse", crate::verbs::capability::CALL));
        out.push(Choice::Verdict("hold", crate::verbs::capability::CALL));
    }
    out
}

/// **The selected conversation's own row in what is waiting**, where the
/// queue has answered one.
pub fn row(model: &Model) -> Option<QueueRow> {
    let id = model.conversation.as_ref()?;
    model
        .waiting
        .iter()
        .flat_map(|section| section.rows.iter())
        .find(|row| row.agent == *id)
        .cloned()
}

/// **Answer with one choice**: the same doors the queue pane spends.
pub fn answer(model: &mut Model, row: &QueueRow, choice: Choice) {
    match choice {
        Choice::Seen => model.post_seen(row),
        Choice::Verdict(verdict, scope) => model.post_answer(row, verdict, scope),
    }
}

/// Which row is chosen, clamped to what is offered.
fn chosen(ctx: &egui::Context, len: usize) -> usize {
    ctx.data(|d| d.get_temp::<usize>(egui::Id::new(CHOSEN_ID)))
        .unwrap_or(0)
        .min(len.saturating_sub(1))
}

/// **Move the choice** by one, saturating at the ends.
pub fn moved(ctx: &egui::Context, model: &Model, step: isize) {
    let Some(row) = row(model) else {
        return;
    };
    let len = choices(&row).len();
    let at = chosen(ctx, len).saturating_add_signed(step).min(len - 1);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(CHOSEN_ID), at));
}

/// **Answer with the chosen row** — Enter.
pub fn confirm(ctx: &egui::Context, model: &mut Model) {
    let Some(row) = row(model) else {
        return;
    };
    let offered = choices(&row);
    if let Some(choice) = offered.get(chosen(ctx, offered.len())) {
        answer(model, &row, *choice);
    }
}

/// **Answer with the `n`th row, counted from one** — a digit. Answers
/// whether there was such a row.
pub fn pick(model: &mut Model, n: usize) -> bool {
    let Some(row) = row(model) else {
        return false;
    };
    let Some(choice) = n.checked_sub(1).and_then(|i| choices(&row).get(i).copied()) else {
        return false;
    };
    answer(model, &row, choice);
    true
}

/// Paint the box for `row`, the selected conversation's own
/// ([`row`]), which the shell has already found.
pub fn render(ui: &mut egui::Ui, model: &mut Model, row: &QueueRow) {
    let accent = theme::accent(theme::State::Attention);
    let offered = choices(row);
    let at = chosen(ui.ctx(), offered.len());
    let mut taken = None;
    egui::Frame::none()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0, accent))
        .rounding(theme::RADIUS)
        .inner_margin(egui::Margin::symmetric(theme::space::S, theme::space::XS))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(HEADING).color(accent));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(HINT).small().color(theme::INK_WEAK));
                });
            });
            for (said, ink) in words(row) {
                ui.add(egui::Label::new(egui::RichText::new(said).color(ink)).wrap());
            }
            for (i, choice) in offered.iter().enumerate() {
                let seat = theme::paint::row(
                    ui,
                    &format!("{}. {}", i + 1, choice.word()),
                    theme::INK,
                    None,
                    i == at,
                    0.0,
                );
                crate::ui::act::tag(&seat, &[choice.op()]);
                if seat.clicked() {
                    taken = Some(*choice);
                }
            }
        });
    if let Some(choice) = taken {
        answer(model, row, choice);
    }
}

/// **What the row says**, in the queue pane's own words and inks: the
/// parked call, the flag, the signals, the failure, the last words.
pub fn words(row: &QueueRow) -> Vec<(String, egui::Color32)> {
    let note = theme::accent(theme::State::Annotation);
    let mut out = Vec::new();
    if let Some(said) = queue::parked(row) {
        out.push((said, note));
    }
    if let Some(said) = queue::flagged(row) {
        out.push((said, note));
    }
    if let Some(said) = queue::signalled(row) {
        out.push((said, theme::INK_WEAK));
    }
    if let Some(said) = &row.failure {
        out.push((said.clone(), theme::accent(theme::State::Error)));
    }
    if !row.preview.is_empty() {
        out.push((row.preview.clone(), theme::INK));
    }
    out
}

#[cfg(test)]
mod tests;
