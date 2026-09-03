//! **The interface-parity gate** — yog's `docs/PARITY.md` §5, this seat's half.
//!
//! The operator's requirement is that the desktop seat and the android client
//! have interaction parity: *if something is interactable in one it must exist
//! in the other*, and drift between them must be caught mechanically rather
//! than noticed by hand. The contract answers that without either client ever
//! reading the other's tree — **each client is judged against the roster**, and
//! the roster is yog's help table published through the vendored corpus
//! ([`roster`]). A client-vs-client diff would have no authority when the two
//! disagreed, would drift with whichever updated last, and would go quadratic
//! on a third surface.
//!
//! Four assertions, and the last two are what keep the ledger honest:
//!
//! ```text
//! roster    − exemptions ⊆ inventory      no control-classed op is silently absent
//! tags(inventory)        ⊆ ops(roster)    no act: token names a verb the wire lacks
//! ∀ exemption            ∈ roster.control no rotted row: upstream still owes it
//! ∀ exemption            ∉ inventory      no stale row: it is not surfaced already
//! ```
//!
//! # The instrument is the sibling harness's, never a second one
//!
//! The inventory comes off the SAME AccessKit tree [`super::clipped`] judges
//! and [`super::reach`] walks (bl-dc07). A second walk would be a second
//! opinion about what is on the window, and the first defect it found would be
//! a disagreement between the two instruments rather than about the seat.
//!
//! **Presence is the claim; depth is the harness's.** This asserts a tagged
//! node exists in the walked tree. Whether it is reachable in bounded gestures,
//! not clipped off-screen and not painted over are the other three assertions'
//! questions and stay theirs. A tag on a dead button passes here on purpose
//! (PARITY §8) — driving the tagged node and asserting the emitted envelope's
//! `op` equals the tag is the rung above, and it is filed rather than built.
//!
//! # Unproven is red
//!
//! A control that exists only on a screen the walk never visits fails honestly.
//! The walk's screen set — [`super::worlds`] — is part of the instrument, and
//! the start control is why it has four worlds rather than three: extend the
//! walk, or move the control.

use std::collections::BTreeSet;

use crate::ui::{Model, act};
use egui_kittest::Harness;
use egui_kittest::kittest::{Queryable, by};

pub(crate) mod exempt;
pub(crate) mod roster;

mod tests;

/// **Every op tagged on a control in one settled frame.**
///
/// It reads `author_id`, which is where [`crate::ui::act::tag`] writes and what
/// AccessKit reserves for an author's own machine identification of a node —
/// so a control's spoken label is not consulted and cannot drift into this.
/// Hidden nodes are skipped: `is_hidden` is the tree's own statement that the
/// node is not currently offered to anybody, and a control nobody is offered is
/// not a surfaced control.
pub(crate) fn inventory(harness: &Harness<'_, Model>) -> BTreeSet<String> {
    harness
        .query_all(by())
        .filter(|node| !node.is_hidden())
        .filter_map(|node| node.author_id().map(str::to_owned))
        .flat_map(|author| {
            author
                .split_whitespace()
                .filter_map(|token| token.strip_prefix(act::PREFIX))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// **The four assertions.** An empty answer is parity holding.
///
/// Every complaint names what to do about it, because the three ways this
/// reddens want three different acts — build a control, fix a typo, or edit one
/// line of `parity.toml` — and a reader looking at a red gate is a reader who
/// does not yet know which of the three they are in.
pub(crate) fn complaints(
    roster: &roster::Roster,
    exemptions: &[exempt::Exemption],
    inventory: &BTreeSet<String>,
) -> Vec<String> {
    let recorded: BTreeSet<&str> = exemptions.iter().map(|row| row.op.as_str()).collect();
    let mut out = Vec::new();
    for op in &roster.control {
        if !inventory.contains(op) && !recorded.contains(op.as_str()) {
            out.push(format!(
                "{op:?} is classed a control by the roster and this seat has neither a \
                 control tagged {PREFIX}{op} nor a line for it in parity.toml",
                PREFIX = act::PREFIX
            ));
        }
    }
    for op in inventory {
        if !roster.ops.contains(op) {
            out.push(format!(
                "a control is tagged {}{op} and the roster has no such op — \
                 a stale or mistyped tag",
                act::PREFIX
            ));
        }
    }
    for row in exemptions {
        if !roster.control.contains(&row.op) {
            out.push(format!(
                "parity.toml records {:?} absent and the roster no longer classes it a \
                 control — the line has rotted; delete it",
                row.op
            ));
        }
        if inventory.contains(&row.op) {
            out.push(format!(
                "parity.toml records {:?} absent and a control is tagged for it — \
                 the line is stale; delete it, the control is the record",
                row.op
            ));
        }
    }
    out
}
