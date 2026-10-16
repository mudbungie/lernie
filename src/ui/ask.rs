//! **The ask**: the selected conversation's decision, standing above the
//! prompt while it waits (DESIGN §4.42, item 8; `docs/STYLE.md` §5, *the
//! ask*).
//!
//! It is §4.19's queue entry for the one conversation on the glass, painted
//! where the answer is typed: a framed box in the attention accent between
//! the transcript and the prompt line — the one thing that stands between
//! them (§4.45 item 3) — carrying what the queue's row says about it
//! — the parked call, the flag, the signals, the failure, the last words —
//! and its choices as numbered rows. The prompt keeps the caret while it
//! asks (bl-1252), so the digits pick, the arrows move and Enter answers only
//! while the draft is empty ([`keys`]); with words in it every key is the
//! box's and Enter sends them. A click on a row is the same answer.
//!
//! **The choices are the queue's own acts and no others**: `seen`, and where
//! a call is parked the three verdicts — `pass` for this call, for this
//! conversation and for this workspace, which are the baseline's *once*,
//! *this session* and *always* on this wire's own words (REMOTE's scopes),
//! then `refuse` and `hold`. `go to it` is not among them: the operator is
//! already there. The queue pane (`/attention`) is the same rows for every
//! conversation — it reads [`choices`] too, so there is one list — and the
//! count on the rule is its door.
//!
//! **Which reaches `pass` is offered at is the held row's to say** (yog
//! bl-0da6; bl-da16). The engine refuses every reach but the call for a call
//! it classified secret or destructive, and the seat used to offer all three
//! regardless, so an operator who picked a wider one was refused and the call
//! stayed parked. The classification and the reaches it permits ride the wire
//! on the row (`Held::scopes`), because REMOTE's rule is that a seat never
//! re-derives either from the reason's prose; a row that lists none is an
//! older engine, and offers all three as before — the refusal still lands on
//! the rule, and the call-scoped rows always go through.
//!
//! **The read stands while the conversation asks.** The queue's read used to
//! stand only on its pane; this box is the second place it is spent, so it
//! stands while the selected conversation's row says it is asking
//! (`crate::state::Standing::asking`) — which is the roster's own word, read
//! before the queue has answered, so the box can be answered on the frame
//! the queue's row lands.

use crate::reply::queue::QueueRow;
use crate::ui::{Channel, Model, queue, theme};

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

    /// **What the choice does to the parked call**, one sentence (DESIGN
    /// §4.46 item 2) — the scope said where it is wider than the one call.
    pub fn help(self) -> String {
        match self {
            Self::Seen => "marks the ask seen; the call stays parked".to_owned(),
            Self::Verdict(verdict, scope) => {
                let does = match verdict {
                    "pass" => "lets the call run",
                    "refuse" => "refuses the call",
                    _ => "keeps the call parked for a later answer",
                };
                if scope == crate::verbs::capability::CALL {
                    does.to_owned()
                } else {
                    format!("{does}, and every call like it in this {scope}")
                }
            }
        }
    }

    /// The op the row fires, which is its tag.
    pub(crate) fn op(self) -> &'static str {
        match self {
            Self::Seen => crate::verbs::SEEN.word,
            Self::Verdict(..) => crate::verbs::ANSWER.word,
        }
    }

    /// Whether it settles more than the one held call — the queue pane
    /// stands these under their own line.
    pub fn wider(self) -> bool {
        matches!(self, Self::Verdict(_, scope) if scope != crate::verbs::capability::CALL)
    }
}

/// **The choices a row offers**, in the order they are numbered: `seen`;
/// where a call is parked, `pass` at each reach the row lists — in
/// `SCOPES`' order, which is the wire's, and only the reaches this build can
/// spell, so a word a newer engine adds is not offered rather than offered
/// unsaid — and all three where it lists none; then `refuse` and `hold` for
/// the one call (bl-da16, the module's own doc).
pub fn choices(row: &QueueRow) -> Vec<Choice> {
    let mut out = vec![Choice::Seen];
    if let Some(held) = &row.held {
        let listed =
            |scope: &&str| held.scopes.is_empty() || held.scopes.iter().any(|s| s == scope);
        out.extend(
            crate::verbs::SCOPES
                .iter()
                .copied()
                .filter(listed)
                .map(|scope| Choice::Verdict("pass", scope)),
        );
        out.push(Choice::Verdict("refuse", crate::verbs::capability::CALL));
        out.push(Choice::Verdict("hold", crate::verbs::capability::CALL));
    }
    out
}

/// **The selected conversation's own row in what is waiting**, where the
/// queue has answered one — with the channel its section came down, which
/// every answer is resolved within.
///
/// Matched on the conversation's id **and on its wall resolving to the aim**
/// (bl-666f): ids and wall names are each only unique within one engine, so
/// a row is the selected conversation's only where it is that id on the very
/// wall, on the very channel, the operator is standing on.
pub fn row(model: &Model) -> Option<(Channel, QueueRow)> {
    let id = model.conversation.as_ref()?;
    let aim = model.aim.as_ref()?;
    model.waiting.iter().find_map(|section| {
        section
            .rows
            .iter()
            .find(|row| {
                row.agent == *id
                    && model.wall(&section.channel, &row.workspace).as_ref() == Some(aim)
            })
            .map(|row| (section.channel.clone(), row.clone()))
    })
}

/// **Answer with one choice**: the two doors, which the queue pane spends
/// through this same function.
pub fn answer(model: &mut Model, channel: &Channel, row: &QueueRow, choice: Choice) {
    match choice {
        Choice::Seen => model.post_seen(channel, row),
        Choice::Verdict(verdict, scope) => model.post_answer(channel, row, verdict, scope),
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
    let len = choices(&row.1).len();
    let at = chosen(ctx, len).saturating_add_signed(step).min(len - 1);
    ctx.data_mut(|d| d.insert_temp(egui::Id::new(CHOSEN_ID), at));
}

/// **Answer with the chosen row** — Enter.
pub fn confirm(ctx: &egui::Context, model: &mut Model) {
    let Some(row) = row(model) else {
        return;
    };
    let offered = choices(&row.1);
    if let Some(choice) = offered.get(chosen(ctx, offered.len())) {
        answer(model, &row.0, &row.1, *choice);
    }
}

/// **Answer with the `n`th row, counted from one** — a digit. Answers
/// whether there was such a row.
pub fn pick(model: &mut Model, n: usize) -> bool {
    let Some(row) = row(model) else {
        return false;
    };
    let Some(choice) = n
        .checked_sub(1)
        .and_then(|i| choices(&row.1).get(i).copied())
    else {
        return false;
    };
    answer(model, &row.0, &row.1, choice);
    true
}

/// **Why the engine this conversation is on will not dial**, where it will
/// not (bl-8d6d; `crate::ui::status::diagnostics::down`): nothing can be
/// answered meanwhile, so the box's choices go faint and take no click or
/// key, and the field's hint is this sentence instead of the keys.
pub(crate) fn down(model: &Model) -> Option<String> {
    crate::ui::status::diagnostics::down(model)
}

/// Paint the box for `row`, the selected conversation's own
/// ([`row`]), which the shell has already found.
pub fn render(ui: &mut egui::Ui, model: &mut Model, asked: &(Channel, QueueRow)) {
    let row = &asked.1;
    let accent = theme::accent(theme::State::Attention);
    let offered = choices(row);
    let at = chosen(ui.ctx(), offered.len());
    let down = down(model).is_some();
    let ink = if down { theme::INK_FAINT } else { theme::INK };
    let mut taken = None;
    egui::Frame::none()
        .fill(theme::SURFACE)
        .stroke(egui::Stroke::new(1.0, accent))
        .rounding(theme::RADIUS)
        .inner_margin(egui::Margin::symmetric(theme::space::S, theme::space::XS))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(HEADING).color(accent));
                if !down {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(HINT).small().color(theme::INK_WEAK));
                    });
                }
            });
            for (said, ink) in words(row) {
                ui.add(egui::Label::new(egui::RichText::new(said).color(ink)).wrap());
            }
            for (i, choice) in offered.iter().enumerate() {
                let seat = theme::paint::row(
                    ui,
                    &format!("{}. {}", i + 1, choice.word()),
                    ink,
                    None,
                    i == at,
                    0.0,
                    &choice.help(),
                );
                crate::ui::act::tag(&seat, &[choice.op()]);
                if seat.clicked() && !down {
                    taken = Some(*choice);
                }
            }
        });
    if let Some(choice) = taken {
        answer(model, &asked.0, &asked.1, choice);
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

/// The ask's keys, and what the field says while it asks.
mod keys;

pub use keys::{hint, keys};

#[cfg(test)]
mod tests;
