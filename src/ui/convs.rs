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

/// What a wall says when it answered and holds no conversation. **It names the
/// next act** (§4.38), and the act is the `+` on the engine's own row above it
/// (`crate::ui::roster::engine::BEGIN`).
pub const NO_CONVERSATIONS: &str = "no conversations here yet — begin one with + on the engine";
/// **What it says for a wall it has not been ANSWERED about** (bl-f780).
///
/// The [`UNCERTAIN`] doctrine one level up: *no conversations here* is a
/// definite fact about a wall nobody has looked at yet, and this pane already
/// refuses to state a definite fact about a conversation nobody could take a
/// reading of. It stands from the keypress that aims until the answer lands —
/// a round trip on a wire, not the millisecond it is on loopback.
pub const NOT_ANSWERED: &str = "waiting to hear about this wall";

/// The mark a state nothing observed wears, inside the badge it qualifies.
pub const UNCERTAIN: &str = "?";

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

/// A row's headline: its label, what it is doing, how long since it moved, and
/// what is waiting under it.
///
/// **The badge wears a `?` for a state nothing observed.** The engine answers a
/// reading and whether it could take one, and painting the first without the
/// second would state a definite fact about a conversation nobody looked at —
/// including this window's own, between a start's receipt and its driver's
/// first write.
pub fn headline(row: &ConvRow) -> String {
    let mut said = vec![format!(
        "{}  [{}{}]  {}",
        row.display,
        row.state.label(),
        if row.uncertain { UNCERTAIN } else { "" },
        age(row.age_secs)
    )];
    if row.attention > 0 {
        said.push(format!("{} waiting", row.attention));
    }
    if row.members > 1 {
        said.push(format!("{} members", row.members));
    }
    said.join("  ")
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
