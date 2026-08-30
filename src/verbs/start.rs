//! **The start family's two envelopes** (yog's `docs/REMOTE.md` §8.1, §9.8) —
//! and they are doors without rows, which is the one thing to understand here.
//!
//! [`super`]'s table is *a word and its parameters, all of them named strings*,
//! and that is what keeps it one builder with no per-verb arm to drift. Neither
//! of these fits it: `prepare` carries a **payload rung** and `prompt` carries
//! a **prepared body**, and a nested object is not a word an operator types.
//! [`super`]'s own rule says what happens then — such a gesture *"is not added
//! as a special case"* — so the rows stay six and these are typed doors beside
//! them, exactly as [`super::message`] is a door onto its row. What an operator
//! types instead is `lernie start`, the composite: one word for both acts,
//! because a one-shot process can hold the first reply between them
//! ([`crate::seat::start`]).
//!
//! # The bare rung, and only the bare rung
//!
//! yog's `docs/DESIGN.md` §3.4 gives the payload three rungs — bare, a work
//! directory, a ball. This seat composes the **bare** one: a conversation in a
//! workspace, with no work target and no delivery obligation. The other two
//! are unbuilt rather than unreachable — a directory needs a field that refuses
//! a path that is not there, and a ball needs a project, a picker and the §3.5
//! join states — and each arrives with the surface that composes it. A seat
//! that guessed a rung would found a claim nobody asked for.
//!
//! # The prepared body is handed back, and re-addressed on the way
//!
//! The fire carries the staged body **verbatim** ([`crate::reply::start`] on
//! why), with one field rewritten: the workspace. It came back in the *host's*
//! spelling, and this box's §8.2 mapping runs client→host at
//! [`crate::seat::route`] and nowhere else — so a body handed back unrewritten
//! names a workspace no entry claims, falls through to the flat root, and fires
//! the start into this box's own engine. [`crate::envelope`] already records
//! that hazard beside the nested slot it reads; this is the site it was written
//! about.

use serde_json::{Value, json};

use crate::envelope;
use crate::reply::start::Prepared;

/// The staging act's `op`.
pub const PREPARE: &str = "prepare";
/// The fire's `op`.
pub const PROMPT: &str = "prompt";

/// The rung this seat composes, and the field it rides under.
const PAYLOAD: &str = "payload";
const RUNG: &str = "bare";

/// **Stage a start** in the workspace `address` names, on the bare rung.
///
/// `address` is **this box's** name for the workspace — the leaf of an entry,
/// or the name this box's own engine answers to — because the mapping is spent
/// at the channel boundary and every gesture is composed on the client side of
/// it.
pub fn prepare(address: String) -> Value {
    json!({
        envelope::OP: PREPARE,
        envelope::WORKSPACE: address,
        PAYLOAD: { "rung": RUNG },
    })
}

/// **Fire a staged start** with the goal the operator typed.
///
/// `address` re-addresses the body into this box's spelling (see the module
/// doc); `seed` is spelled `null` because this seat predicts no conversation
/// name — the mint is the engine's, and a seat that predicted one would have to
/// fire the name it painted.
pub fn prompt(prepared: &Prepared, address: String, goal: String) -> Value {
    json!({
        envelope::OP: PROMPT,
        envelope::PREPARED: envelope::with_workspace(&prepared.body, &address),
        "goal": goal,
        "seed": Value::Null,
    })
}

#[cfg(test)]
mod tests;
