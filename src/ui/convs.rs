//! **The conversation list** for the wall the window is aimed at.
//!
//! A row is a glance, not a transcript: what it is called, what it is doing,
//! how long since it moved, why its latest model call failed if it did, and
//! the first line of what was said. Everything deeper is one click away in the
//! chat pane, and a list that tried to be the pane would be neither.

use crate::reply::convs::ConvRow;
use crate::ui::Model;

/// The row's own acts, on the menu a secondary click opens.
pub mod menu;
/// **One row on the glass**: its words, the lines under it, its subtree
/// control and its threading connectors.
mod row;

pub use row::{HIDE, SHOW};

/// What the list says with no wall aimed at.
pub const NO_WALL: &str = "pick a workspace under channels to list its conversations";
/// What it says for a wall that answered, and answered nothing.
pub const NO_CONVERSATIONS: &str = "no conversations here yet — begin one in the box below";
/// **What it says for a wall it has not been ANSWERED about** (bl-f780).
///
/// The third sentence, and it is the [`UNCERTAIN`] doctrine one level up: *no
/// conversations here* is a definite fact about a wall nobody has looked at
/// yet, and the pane already refuses to state a definite fact about a
/// conversation nobody could take a reading of. It stands from the keypress
/// that aims until the answer lands — a round trip on a wire, not the
/// millisecond it is on loopback.
pub const NOT_ANSWERED: &str = "waiting to hear about this wall";

/// **What it says for an aim this seat cannot ask about at all**, which is
/// permanent rather than transient.
///
/// `crate::place` restores a saved aim without checking it, on the ground that
/// a stale one is inert — `crate::state::Standing::aimed` finds no channel by
/// that name and asks nothing. Inert is right about the dialling and wrong
/// about the paint: nothing is ever asked, so [`NOT_ANSWERED`] would stand
/// forever over a wall that has no channel to answer it. The refusal that was
/// silent is said here instead.
pub fn no_channel(channel: &str) -> String {
    format!(
        "this seat holds no channel named {channel:?}, so nothing is asked about this wall — pick one from the channels beside it"
    )
}

/// The word this pane wears, and the subject the arrows act on when it is
/// focused. **It is painted by `crate::ui::shell`** — above the pane in the
/// broad shape, on the navigation bar in the narrow one (bl-dfda) — because a
/// column's name has one home and which one it is depends on the shape.
pub const HEADING: &str = "conversations";
/// The mark a state nothing observed wears, inside the badge it qualifies.
pub const UNCERTAIN: &str = "?";

/// Paint the list and take a click on it. **The heading is the shell's** —
/// see [`HEADING`].
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let Some(aim) = model.aim.clone() else {
        crate::ui::theme::paint::empty(ui, NO_WALL);
        return;
    };
    ui.label(aim.address.clone());
    // **An aim on a channel this seat does not hold is asked about by nobody**,
    // so it gets its own sentence rather than one that implies an answer came
    // back. The roster carries every channel this box holds from boot, off the
    // disk and before anything is dialled, so this is a question about the
    // model and not about a socket.
    if !model.holds(&aim.channel) {
        crate::ui::theme::paint::empty(ui, &no_channel(&aim.channel));
        return;
    }
    // **The list is the model's, not this pane's**: a conversation this window
    // has started but the engine cannot resolve yet stands in it as a row of
    // its own (`crate::ui::model::claim`), and it stands there for the pointer
    // and the keyboard alike because both walk the one list.
    let rows = model.rows();
    if rows.is_empty() {
        crate::ui::theme::paint::empty(
            ui,
            if model.answered.as_ref() == Some(&aim) {
                NO_CONVERSATIONS
            } else {
                NOT_ANSWERED
            },
        );
        return;
    }
    // The list scrolls; the heading and the address above it do not (bl-e5d2,
    // and `crate::ui::roster` for why the heading stays out).
    let reveal = model.revealing(crate::ui::keys::Pane::Conversations);
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            for (at, conv) in rows.iter().enumerate() {
                // **What a rail says is a fact about the LIST**, not about the
                // row, so it is answered here where the whole list is in hand
                // and handed down. Level `depth` itself is the row's own
                // elbow and never a rail.
                let rails: Vec<u64> = (1..conv.depth)
                    .filter(|level| continues(&rows, at, *level))
                    .collect();
                row::conversation(ui, model, &aim, conv, &rails, reveal);
            }
        });
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
