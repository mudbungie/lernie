//! **The conversation's own row, as the records pane's header** (bl-3257).
//!
//! The pane's other halves are what is UNDER a conversation — the steps, the
//! worktree, the spine, the mail. This is the conversation itself, so it leads
//! the pane rather than sitting in the scroll beside them: it is the subject
//! every other half is about.
//!
//! # It is DENSE, and the density is a constraint rather than a taste
//!
//! This pane covers the window and its content must fit it: a control laid out
//! past the frame is unreachable, and `crate::snapshot::clipped` fails the
//! whole matrix over one. The header answers twenty-one facts, so it says
//! several of them per line — the identity with the rest, the descent with the
//! marks and the seats, the spend with the fullness, what is in flight with
//! what may be done — joined by one separator and never by a sentence composed
//! between them. Nothing is dropped to make room; what is dropped is the line
//! breaks.
//!
//! # Every line is a pure function of the row, and three of them are the
//! engine's own words
//!
//! The strip's characteristics, the money and the attribution sentence are
//! prose the engine assembled (`crate::reply::agent`), so they are repeated
//! rather than reassembled — and the context percent is its own unclamped
//! rounding, painted as it came. What this file composes is only the joining.
//!
//! # The four gates are painted as what the ENGINE offers, not as enablement
//!
//! `nudgeable`, `stoppable`, `stop_children` and `present` are the engine's
//! answer to *what may be done to this conversation*. They are painted as a
//! sentence here — the one place in this window that says what the far end
//! thinks is available in words — and since bl-f251 the composer's row reads
//! the same answer to decide which of its acts to offer
//! (`crate::ui::composer::offers`): never greyed, which would predict a
//! refusal, but withheld on the engine's own word and offered whole until
//! the engine has said anything.

use crate::reply::agent::{Agent, Fullness, Offer};
use crate::reply::spend::Figure;
use crate::ui::{Model, theme};

/// The header's own heading.
pub const HEAD: &str = "the conversation";
/// What it says before the first answer.
pub const NOT_ANSWERED: &str = "waiting to hear what this conversation is";
/// Said of a name no stored fact backs, so no peer can address it.
pub const DISPLAY_ONLY: &str = "a display name only — nothing can be addressed by it";
/// Said of a conversation the engine's snapshot does not carry.
pub const ABSENT: &str = "the engine's snapshot does not carry it";
/// Said when the engine offers nothing on it.
pub const NOTHING_OFFERED: &str = "the engine offers nothing on it";

/// Paint the header, or the sentence for a conversation nobody has answered
/// about yet.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    // **Taken by value before anything is painted**, exactly as the pane's own
    // two halves take their listings: the offers line's control acts on the
    // model, and the row it reads is a snapshot of the same frame.
    // **The section word is the divider** (`docs/STYLE.md` §5): `L` of air, a
    // hairline and the word in weak ink, with the header's own line under it.
    theme::paint::section(ui, HEAD);
    let Some(row) = model.records.agent.clone() else {
        ui.label(NOT_ANSWERED);
        return;
    };
    let row = &row;
    ui.label(named(row));
    // **What is wrong with it shares one wrapped row, and the figures share
    // the next** (bl-3686's rule, one turn further): the pane has to FIT the
    // narrowest shape this layout promises, and what is dropped for the room
    // is a line break, never a fact. The inks still tell them apart — a
    // refusal is the error accent, a note the annotation one.
    ui.horizontal_wrapped(|ui| {
        if row.display_only {
            ui.colored_label(theme::accent(theme::State::Annotation), DISPLAY_ONLY);
        }
        if let Some(said) = row.failure.clone() {
            ui.colored_label(theme::accent(theme::State::Error), said);
        }
        if let Some(said) = parked(row) {
            ui.colored_label(theme::accent(theme::State::Annotation), said);
        }
    });
    ui.horizontal_wrapped(|ui| {
        if let Some(said) = about(row) {
            ui.colored_label(theme::INK_WEAK, said);
        }
        ui.label(costing(row));
    });
    // **The offers line and the one control that answers it share a row**
    // (bl-3686): the sentence says what the engine is offering, and the
    // cascade is the half of that sentence no other surface in this window
    // can fire. The pane is dense and a control laid out past the frame is
    // unreachable (`crate::snapshot::clipped`), so it wraps rather than
    // taking a line of its own.
    ui.horizontal_wrapped(|ui| {
        ui.colored_label(theme::INK_WEAK, doing(row));
        super::cascade::control(ui, model, row);
    });
}

/// What joins two facts on one line. A separator rather than a sentence: the
/// facts are the engine's and the joining is this seat's.
const JOIN: &str = "  ·  ";

/// **The line that joins where it hangs, what it wears and who is sitting in
/// it** — three facts about the conversation's shape, none of which is worth a
/// line of its own. `None` where it states none of them.
pub fn about(row: &Agent) -> Option<String> {
    let said: Vec<String> = [descent(row), marked(row), seated(row)]
        .into_iter()
        .flatten()
        .collect();
    (!said.is_empty()).then(|| said.join(JOIN))
}

/// **The line that joins the spend with the fullness** — two figures that
/// answer different questions and are read in one glance.
pub fn costing(row: &Agent) -> String {
    let said = spent(&row.spend);
    match &row.context {
        Some(full) => format!("{said}{JOIN}{}", contextual(full)),
        None => said,
    }
}

/// **The line that joins what is happening with what may be done about it** —
/// the strip's own characteristics where the engine composed any, and the
/// gates beside them.
pub fn doing(row: &Agent) -> String {
    match flighted(row) {
        Some(said) => format!("{said}{JOIN}{}", offered(row)),
        None => offered(row),
    }
}

/// **The one line the header always gets**: what it is called, how it is
/// resting, and the branch tip every config derivation is taken against.
pub fn named(row: &Agent) -> String {
    format!("{} — {} — at {}", row.display, resting(row), tipped(row))
}

/// The tip, or the engine's own emptiness said as a word: a conversation the
/// snapshot does not carry has no tip, and an empty string in a sentence reads
/// as a sentence that lost its ending.
fn tipped(row: &Agent) -> String {
    if row.tip.is_empty() {
        return "no branch tip".to_owned();
    }
    row.tip.clone()
}

/// How it is resting: the state, and whether the provider refused the last
/// turn — which is the fact that tells an operator's own stop apart from one.
pub fn resting(row: &Agent) -> String {
    let said = row.state.label();
    if row.refused {
        return format!("{said} — the provider refused the latest turn");
    }
    said
}

/// What is in flight on it, with the engine's own characteristics where it
/// composed any.
pub fn flighted(row: &Agent) -> Option<String> {
    match (&row.flight, &row.strip) {
        (_, Some(strip)) => Some(format!("{} — {}", strip.class, strip.facts)),
        (Some(class), None) => Some(format!("in flight: {class}")),
        (None, None) => None,
    }
}

/// The invocation parked at its capability boundary, said in the engine's
/// own sentence about it.
pub fn parked(row: &Agent) -> Option<String> {
    let held = row.held.as_ref()?;
    Some(format!(
        "held at {} ({}) — {}",
        held.tool, held.tool_use, held.reason
    ))
}

/// Where it hangs, or none for a conversation that is its own root.
pub fn descent(row: &Agent) -> Option<String> {
    if row.ancestors.is_empty() {
        return None;
    }
    Some(format!("under {}", row.ancestors.join(" / ")))
}

/// The marks it wears, or none.
pub fn marked(row: &Agent) -> Option<String> {
    (!row.marks.is_empty()).then(|| format!("marked {}", row.marks.join(", ")))
}

/// The live mark's seats, or none — the mark at rest says nothing rather than
/// saying it is empty.
pub fn seated(row: &Agent) -> Option<String> {
    if row.seats.is_empty() {
        return None;
    }
    let said: Vec<String> = row
        .seats
        .iter()
        .map(|seat| format!("{} {}", seat.name, seat.doing))
        .collect();
    Some(said.join(", "))
}

/// What it has spent: the counters' total, the money where there is a price
/// table, and whose figure it is where that is not obvious.
pub fn spent(figure: &Figure) -> String {
    let mut said = format!("{} tokens", figure.tokens.total);
    if let Some(usd) = &figure.usd {
        said = format!("{said} — {usd}");
    }
    match &figure.attribution.label {
        Some(label) => format!("{said}, {label}"),
        None => said,
    }
}

/// How full the context is, in the engine's own percent.
pub fn contextual(full: &Fullness) -> String {
    format!(
        "context {}% — {} of {} for {}",
        full.percent, full.prompt_tokens, full.window, full.model
    )
}

/// **What the engine says may be done to it.** A sentence here, and the
/// composer's row reads the same list to decide what it offers
/// (`crate::ui::composer::offers`); a refusal is still the far end's to make.
pub fn offered(row: &Agent) -> String {
    if !row.present {
        return ABSENT.to_owned();
    }
    if row.offers.is_empty() {
        return NOTHING_OFFERED.to_owned();
    }
    let words: Vec<&str> = row.offers.iter().copied().map(word).collect();
    format!("the engine offers {}", words.join(", "))
}

/// One offer's word. The set is closed, so this is a total function and not a
/// table that can miss.
///
/// **It is the cascade control's label too** (`super::cascade`), which is why
/// it is reachable at all: the sentence and the button say one word, read from
/// one place, so neither can be reworded out of agreement with the other.
///
/// `pub(crate)` and not `pub`, because a `&'static str` off the crate's real
/// surface is rule 2's borrow-shaped return — and the honest demotion is what
/// that rule asks for before a clone-to-own.
pub(crate) fn word(offer: Offer) -> &'static str {
    match offer {
        Offer::Nudge => "nudge",
        Offer::Stop => "stop",
        Offer::Children => "stop with its children",
    }
}

#[cfg(test)]
mod tests;
