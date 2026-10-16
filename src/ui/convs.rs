//! **The conversations that stand under a wall** (DESIGN §4.39) — the rows
//! themselves, the two sentences a wall with none says, and the words a row is
//! made of.
//!
//! A row is a glance, not a transcript: what it is called, what it is doing,
//! how long since it moved, why its latest model call failed if it did, and
//! the first line of what was said. Everything deeper is one click away in the
//! chat pane, and a list that tried to be the pane would be neither.
//!
//! # It was a pane and it is a level (bl-b9a3)
//!
//! Until DESIGN §4.39 this was the window's middle column, with a heading, a
//! scroll region and four emptinesses of its own. The fold put these rows
//! under the wall they belong to, inside the roster's one scroll region, so
//! what is left here is a row and the facts a row is made of — [`under`] is
//! called by `crate::ui::roster::wall` and nothing paints a pane.
//!
//! **Two of the four emptinesses went with the pane and neither is missed.**
//! *pick a workspace under channels to list its conversations* was the pane
//! saying it had no subject; there is no pane, and a wall's rows only ever
//! stand under that wall's own row, so the subject is on the glass by
//! construction. And the sentence for an aim naming a channel this seat does
//! not hold (bl-f780's permanent case) has no row to stand under at all: a
//! stale aim read back out of the place file now shows as an accordion with
//! nothing aimed in it, which is what it is.
//!
//! # And it stayed HERE rather than moving under `roster/`
//!
//! §4.39 left the choice open. The roster is its only caller, but it is not
//! its only reader: `crate::ui::queue` spells a row's age and its uncertainty
//! mark with [`age`] and [`UNCERTAIN`], because a conversation's age reads the
//! same wherever it is written. A vocabulary two panes share is a module
//! beside them, not inside one of them.

use crate::reply::convs::ConvRow;
use crate::ui::{Aim, Model, theme};

/// The row's own acts, on the menu a secondary click opens.
pub mod menu;
/// **One row on the glass**: its words, the lines under it, its subtree
/// control and its threading connectors.
mod row;

pub use row::{HIDE, SHOW};
pub(crate) use row::{STEP, indent};

/// What a wall says when it answered and holds no conversation. **It names the
/// next act** (§4.38), and the act is the `+ new` row under the wall's
/// conversations (`crate::ui::roster::wall::NEW`).
pub const NO_CONVERSATIONS: &str = "no conversations here yet — begin one with + new below";
/// **What it says for a wall it has not been ANSWERED about** (bl-f780).
///
/// The [`UNCERTAIN`] doctrine one level up: *no conversations here* is a
/// definite fact about a wall nobody has looked at yet, and this pane already
/// refuses to state a definite fact about a conversation nobody could take a
/// reading of. It stands from the keypress that aims until the answer lands —
/// a round trip on a wire, not the millisecond it is on loopback.
pub const NOT_ANSWERED: &str = "waiting to hear about this workspace";

/// The mark a state nothing observed wears, inside the badge it qualifies.
pub const UNCERTAIN: &str = "?";
/// What a conversation's row does wherever it stands (DESIGN §4.46 item 2).
pub const SELECT_HELP: &str = "shows this conversation";
/// What the bar row's trashcan does.
pub const TRASH_HELP: &str = "deletes this conversation; Ctrl+click skips the question";

/// **The conversations that stand under the aimed wall**, and the sentence
/// that stands there when it holds none.
///
/// **The list is the model's, not this file's**: a conversation this window
/// has started but the engine cannot resolve yet stands in it as a row of its
/// own (`crate::ui::model::claim`), and it stands there for the pointer and
/// the keyboard alike because both walk the one list
/// (`crate::ui::roster::track`).
pub(crate) fn under(ui: &mut egui::Ui, model: &mut Model, aim: &Aim, reveal: bool) {
    let rows = model.rows();
    if rows.is_empty() {
        theme::paint::empty(
            ui,
            if model.answered.as_ref() == Some(aim) {
                NO_CONVERSATIONS
            } else {
                NOT_ANSWERED
            },
        );
        return;
    }
    for (at, conv) in rows.iter().enumerate() {
        // **What a rail says is a fact about the LIST**, not about the row, so
        // it is answered here where the whole list is in hand and handed down.
        // Level `depth` itself is the row's own elbow and never a rail.
        let rails: Vec<u64> = (1..conv.depth)
            .filter(|level| continues(&rows, at, *level))
            .collect();
        row::conversation(ui, model, aim, conv, &rails, reveal);
    }
}

/// **Whether an ancestor level's thread continues past this row** — the whole
/// of what a threading rail claims (`crate::ui::convs::row`).
///
/// The engine's order is a root followed by its descendants, deepest last, so
/// the answer is the first row below `at` that is NOT deeper than `level`: if
/// it sits exactly at `level` the branch has another member coming and the
/// rail carries on past this row; if it is shallower the branch ended here and
/// a rail would be drawing a sibling that does not exist.
///
/// Pure, over the rows and an index, because that is the only shape of it a
/// test can put a truth table against.
fn continues(rows: &[ConvRow], at: usize, level: u64) -> bool {
    rows.iter()
        .skip(at + 1)
        .find(|row| row.depth <= level)
        .is_some_and(|row| row.depth == level)
}

/// A row's headline: its state's glyph, its label, how long since it moved,
/// and what is waiting under it (DESIGN §4.42 item 7).
///
/// **The glyph wears a `?` for a state nothing observed** — `✓?` is *at
/// rest, as far as anyone has looked*. The engine answers a reading and
/// whether it could take one, and painting the first without the second
/// would state a definite fact about a conversation nobody looked at —
/// including this window's own, between a start's receipt and its driver's
/// first write. The model is not on the row: the wire's conversation row
/// does not carry one, and the switcher does not invent it.
pub fn headline(row: &ConvRow) -> String {
    format!("{} {}", mark(row).0, words(row))
}

/// The headline after its glyph — what the row paints in the row's own ink.
pub(crate) fn words(row: &ConvRow) -> String {
    let mut said = vec![format!("{}  {}", row.display, age(row.age_secs))];
    if row.attention > 0 {
        said.push(format!("{} waiting", row.attention));
    }
    if row.members > 1 {
        said.push(format!("{} members", row.members));
    }
    said.join("  ")
}

/// **A row's state rule**, the accent at its left edge.
///
/// **Asking outranks the state word.** A row that wants the operator wears
/// the attention accent whatever its driver is doing, because the whole of
/// §2's *the eye lands on green* is that nothing else is green — a row asking
/// while it streams would otherwise be purple and lost. Where nothing is
/// waiting the rule is the state's, and a state this build does not know gets
/// none rather than one it is not (`theme::state_of`). The switcher and the
/// bar both read it here (DESIGN §4.44).
pub fn rule(row: &ConvRow) -> Option<egui::Color32> {
    if row.attention > 0 {
        Some(theme::accent(theme::State::Attention))
    } else {
        theme::state_of(&row.state).map(theme::accent)
    }
}

/// **A row's state glyph and its ink**: `?` in the attention accent while it
/// asks — asking outranks the state, because it is the one that wants a
/// person — else `▶` while its driver is live or in flight, `✓` at rest and
/// `■` stopped, each in its state's ink. A state this build does not know
/// wears the note's dot in body ink rather than a glyph it is not.
pub fn mark(row: &ConvRow) -> (String, egui::Color32) {
    glyph(&row.state, row.uncertain, row.attention > 0)
}

/// **[`mark`] over the three facts it reads**, so a row of another kind — the
/// decision queue's (bl-6e59) — wears the switcher's idiom from this one
/// spelling rather than a second.
pub fn glyph(
    state: &crate::reply::convs::AgentState,
    uncertain: bool,
    asking: bool,
) -> (String, egui::Color32) {
    use crate::reply::convs::AgentState;
    let (glyph, ink) = if asking {
        (theme::glyph::ASKING, theme::accent(theme::State::Attention))
    } else {
        let glyph = match state {
            AgentState::Live | AgentState::InFlight => theme::glyph::WORKING,
            AgentState::Quiescent => theme::glyph::AT_REST,
            AgentState::Stopped => theme::glyph::STOPPED,
            AgentState::Unknown(_) => theme::glyph::NOTE,
        };
        (glyph, theme::state_ink(state))
    };
    let unsure = if uncertain { UNCERTAIN } else { "" };
    (format!("{glyph}{unsure}"), ink)
}

/// A compact age: `42s`, `7m`, `3h`, `2d`. **Negative clamps to zero** — two
/// machines' clocks disagreeing is a fact about a seat that dials somewhere
/// else, and an age in the future is not a thing to paint.
pub fn age(secs: i64) -> String {
    let secs = secs.max(0);
    for (bound, unit, per) in [(60, 's', 1), (3600, 'm', 60), (86_400, 'h', 3600)] {
        if secs < bound {
            return format!("{}{unit}", secs / per);
        }
    }
    format!("{}d", secs / 86_400)
}

#[cfg(test)]
mod tests;
