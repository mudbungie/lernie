//! **One wall's row**, and the band of per-wall controls that hangs off the
//! aimed one ([`controls`]).
//!
//! Split from [`super`] at the design-time budget on the seam that module's own
//! doc draws twice over: it is *the channels and their sections*, `acts` is
//! *the ops whose subject is every channel*, and this is *one workspace* — the
//! line it wears, whether this seat can address it at all, and what an operator
//! can do to it once it is aimed at. The first changes when a section grows a
//! sentence; this one when a wall-level op lands a control.

use crate::reply::roster::WsRow;
use crate::ui::{Chunk, Model, theme};

/// The eight acts on one aimed wall, and the band they lie in.
mod controls;

/// **What a row this seat cannot address says instead of being hidden.**
///
/// Dropping it would hide a workspace the operator has; addressing it by the
/// entry's leaf would aim a gesture at a different wall. So it is painted, and
/// painted as what it is.
///
/// The wording states the FACT rather than a verdict (bl-77df). It used to read
/// *"this seat holds no name for it"*, which lands beside a perfectly correct
/// provisioning and reads as an error about the row above it. What is actually
/// true is structural: an entry directory names one workspace, the channel
/// enumerates every workspace that client is registered in, and the extras have
/// no entry of their own — so no envelope this seat can write reaches them.
pub const NO_NAME_HERE: &str = "no entry here names it, so nothing typed here can address it";

/// **The word on the control that floats this wall to the front of the strip**,
/// offered on an aimed row that is not pinned.
pub const PIN: &str = "pin";
/// **And the one that takes it back out**, offered where it is pinned. Two
/// words rather than one that toggles, because the two ops are assertions: the
/// control names the act it fires (`crate::verbs::workspace`).
pub const UNPIN: &str = "unpin";

/// One wall: selectable when this seat can address it, a plain line when it
/// cannot.
///
/// **The eight per-wall controls hang off the aimed row and off no other**,
/// because
/// an enrollment mints the pair `(client, workspace)` and the workspace is
/// exactly what an aim is. Offering it on every row would be offering it before
/// the operator had said which wall — and the answer to that question is
/// already on the screen, once.
pub fn render(ui: &mut egui::Ui, model: &mut Model, chunk: &Chunk, row: &WsRow, reveal: bool) {
    let Some(address) = chunk.channel.address(row) else {
        // **It is a row, and it is faint** (`docs/STYLE.md` §5): the same
        // shape as the rows above and below it, so the list reads as one list,
        // in the ink that says *on the glass and not a target*. It carries no
        // state and takes no click, because there is no gesture to fire.
        theme::paint::row(
            ui,
            &format!("{}  — {NO_NAME_HERE}", line(row)),
            theme::INK_FAINT,
            None,
            false,
            0.0,
        );
        return;
    };
    let aimed = model.aimed_at(&chunk.channel.name, Some(&address));
    // **The row's rule is its state and the aim is the brand** (§2: *the eye
    // lands on green*). Asking outranks running because asking is the one that
    // wants a person; the aimed row keeps the brand over both, which
    // `theme::paint::row` decides so that no pane holds a second opinion.
    let state = if row.attention > 0 {
        Some(theme::accent(theme::State::Attention))
    } else if row.running {
        Some(theme::accent(theme::State::Working))
    } else {
        None
    };
    let seat = theme::paint::row(ui, &line(row), theme::INK, state, aimed, 0.0);
    // **The read this gesture reaches**, not the one that painted the row
    // (yog's `docs/PARITY.md` §2: the interactable a query owes a seat is the
    // affordance that reaches the view it populates). Aiming at a wall is what
    // makes this seat read that wall's conversations.
    crate::ui::act::tag(&seat, &[crate::verbs::CONVERSATIONS.word]);
    if aimed && reveal {
        seat.scroll_to_me(None);
    }
    // **Tab and the arrows agree** (bl-2d6b): a Tab that lands on a wall's
    // row hands the arrows to the roster, so the mark on the heading moves
    // with the ring on the row and one keyboard has one model of where it is.
    if seat.gained_focus() {
        model.focus = crate::ui::keys::Pane::Roster;
    }
    if seat.clicked() {
        model.aim_at(&chunk.channel.name.clone(), &address);
    }
    // **All eight per-wall controls hang off the aimed row and off no other**,
    // and all eight stand down while a pane already covers the conversation:
    // what they open would replace what is standing there, so offering them is
    // offering to lose it without saying so.
    if !aimed || model.covered() {
        return;
    }
    // **They are ONE compact strip under the row, indented, in the order the
    // ledger reads them** (`docs/STYLE.md` §2, DESIGN §4.20): the acts on the
    // wall as an object, a short verb apiece, wrapping at the column's width,
    // with the destructive one last (bl-f251). A column of eight full-width
    // controls under a row would read as eight more rows.
    ui.horizontal_wrapped(|ui| {
        ui.add_space(theme::space::L);
        controls::render(ui, model, row);
    });
}

/// One wall's line: what it is called, how it is classified, and its rollups.
/// The two rollups are stated only when they are non-zero — a roster of `0
/// waiting` on every row teaches nothing and costs the one that says `3`.
pub fn line(row: &WsRow) -> String {
    let mut said = vec![format!(
        "{}  ({})  {} conversations",
        row.workspace,
        row.kind.label(),
        row.agents
    )];
    if row.attention > 0 {
        said.push(format!("{} waiting", row.attention));
    }
    if row.running {
        said.push("running".to_owned());
    }
    said.join("  ")
}

#[cfg(test)]
mod tests;
