//! **The trail's read** — the one op whose subject is what an engine has DONE
//! (yog's `docs/REMOTE.md` §9.17; bl-4c48).
//!
//! # It is a door and not a row, and the reason is one field
//!
//! `ops` carries `max`, and `max` is a **number**. [`super`]'s table is rows of
//! named strings and refuses to grow an arm for anything else — *"a gesture
//! whose parameters are not all strings … is not added as a special case"* —
//! so this is a typed door beside [`super::tuning`]'s two, on exactly their
//! terms: one builder, no second spelling of the gesture, and no row in the
//! table `lernie help` prints. What argv spends instead is `lernie ask`.
//!
//! And `max` is required rather than defaulted: the wire refuses an envelope
//! without it (`non-integer field "max"`). The default lives on yog's own line
//! grammar, which is a different surface with a different reader; a seat that
//! left the field out would be spelling a convenience it does not have.
//!
//! # It names no workspace, so its subject is every channel
//!
//! Read off the envelope and nowhere else (`crate::envelope::workspace`): a
//! gesture with no workspace field has no way to name a channel, so the asker
//! puts it down every one this box holds and the pane is the union
//! (`crate::ui::trail`). That is the decision queue's own shape, one noun over.

use serde_json::{Value, json};

use crate::envelope;

/// The word this door spells, and the envelope's `op`. One fact.
pub const OPS: &str = "ops";

/// The field that says how deep to read.
const MAX: &str = "max";

/// **How much of the trail the pane asks for.**
///
/// A bound is required by the wire and this is the seat's own answer to it,
/// stated once. It is deliberately larger than a screen: the pane scrolls, and
/// an operator reading a trail is looking for the row before the one that
/// broke — a depth that stopped at the visible rows would make *scroll down*
/// answer nothing.
pub const DEPTH: u64 = 200;

/// **The trail, as deep as [`DEPTH`]** — newest last, every action that
/// crossed the boundary.
pub fn ops(max: u64) -> Value {
    json!({ envelope::OP: OPS, MAX: max })
}

#[cfg(test)]
mod tests {
    use super::{DEPTH, OPS, ops};

    #[test]
    fn the_door_builds_the_envelope_the_wire_requires() {
        assert_eq!(
            ops(DEPTH),
            serde_json::json!({ "op": OPS, "max": DEPTH }),
            "the bound is a number and it is required"
        );
    }
}
