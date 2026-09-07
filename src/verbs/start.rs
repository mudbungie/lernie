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
//! # Two rungs of the three, and the third is a picker
//!
//! yog's `docs/DESIGN.md` §3.4 gives the payload three rungs — bare, a work
//! directory, a ball. This seat composes **bare** and **path** (bl-4371): a
//! conversation in a workspace, with or without a work target, and in neither
//! case a delivery obligation. The BALL rung is still unbuilt rather than
//! unreachable — it needs a project, a picker and the §3.5 join states — and it
//! arrives with the surface that composes it. A seat that guessed a rung would
//! found a claim nobody asked for.
//!
//! **The path rung is not decoration, and the measurement is the ball's.** Same
//! model, same box, same minute, one goal each, the only difference being the
//! rung: the bare rung named the directory in prose, the agent's `bash` obeyed
//! the prose while its patch tool wrote into the agent worktree, and it
//! thrashed through three compactors and four million tokens to leave an empty
//! report in the wrong tree. The path rung took eight steps and wrote the file
//! where it was asked to. The rung sets the driver's working directory; prose
//! in a goal cannot.
//!
//! # The role is the one field the seat WRITES into the body it carries back
//!
//! [`ROLE`] (REMOTE §9.21, PROTOCOL 18; yog bl-9ced) is the role a conversation
//! is born on — the soul, the provider assignment and the tool grant litany
//! resolves out of the same config commit `lineage` already chose. yog derives
//! nothing into it: `prepare` answers `null`, which is litany's `worker` and is
//! byte-identical in meaning to every body answered before the field existed,
//! and *which* role an operator wants is a choice made **between** the stage
//! and the fire. Plan mode is exactly that choice, so the seat states it on the
//! body it was already handing back — one field of one gesture, no second op
//! and no second round trip.
//!
//! It is the one exception to the verbatim carry below and it is the carry's
//! own rule rather than a hole in it: the body is handed back untouched EXCEPT
//! where this seat has something to say, which was the workspace and is now
//! the workspace and the role. Unstated leaves whatever the engine answered,
//! which is how a fire that names no role stays the byte-identical fire.
//!
//! **A role this seat does not check**, for the reason it checks no path:
//! litany resolves the role before the fork, so a role the governing commit
//! does not declare leaves no branch, no ref and no worktree, and the engine
//! refuses in its own words. A second validity check here could only disagree
//! with it.
//!
//! **A path this seat does not check.** The directory is the ENGINE's box, not
//! this one — a seat dials a server and may hold no such path at all — so a
//! path that is not there is refused by the engine, in its own words, which is
//! the same division §4.10 draws for `enroll`'s grade in the other direction.
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

/// The payload, its discriminant, and the two rungs this seat composes.
const PAYLOAD: &str = "payload";
const RUNG: &str = "rung";
const BARE: &str = "bare";
const PATH: &str = "path";
/// The work target the path rung carries.
const DIR: &str = "dir";

/// **The field the born-on role rides under** (REMOTE §9.21), spelled once so
/// the door that writes it and the grammar that reads it cannot disagree.
pub const ROLE: &str = "role";

/// **Stage a start** in the workspace `address` names — on the path rung when
/// a work target is named, and on the bare rung when none is.
///
/// `address` is **this box's** name for the workspace — the leaf of an entry,
/// or the name this box's own engine answers to — because the mapping is spent
/// at the channel boundary and every gesture is composed on the client side of
/// it.
///
/// **The rung is said outright and never inferred**, which is upstream's own
/// rule for this payload. A `dir` is the whole of what distinguishes the two
/// here, so there is one `Option` and no second word for the operator to keep
/// in agreement with it.
pub fn prepare(address: String, dir: Option<String>) -> Value {
    let payload = match dir {
        None => json!({ RUNG: BARE }),
        Some(dir) => json!({ RUNG: PATH, DIR: dir }),
    };
    json!({
        envelope::OP: PREPARE,
        envelope::WORKSPACE: address,
        PAYLOAD: payload,
    })
}

/// **The goal a fire carries: the rung's own prefill, and then the
/// operator's.**
///
/// The engine composes a prefill for every rung that has one — the path rung's
/// is the §3.3 target preamble, verbatim — and its own help states the seat's
/// half: *"To fire a prefill with words of your own, send the two joined as one
/// goal — that text is the reply's `prepared.goal`, and editing it is exactly
/// what a seat with a composer does."* A one-shot process has no composer, so
/// the join is where the editing would have been.
///
/// **The bare rung prefills nothing**, so this is the operator's goal
/// unchanged and there is no arm for the ordinary case to take.
pub fn goal(prefill: &str, typed: &str) -> String {
    if prefill.trim().is_empty() {
        return typed.to_owned();
    }
    format!("{prefill}\n\n{typed}")
}

/// **Fire a staged start** with the goal the operator typed.
///
/// `address` re-addresses the body into this box's spelling (see the module
/// doc); `seed` is spelled `null` because this seat predicts no conversation
/// name — the mint is the engine's, and a seat that predicted one would have to
/// fire the name it painted.
/// `role`, where the operator asked for one, is written onto that same body —
/// see the module doc on why this is the carry's rule and not an exception to
/// it. `None` leaves the field exactly as the engine answered it, so a fire
/// that names no role is the fire this seat sent before the field existed.
pub fn prompt(prepared: &Prepared, address: String, goal: String, role: Option<String>) -> Value {
    let mut body = envelope::with_workspace(&prepared.body, &address);
    if let (Some(named), Some(map)) = (role, body.as_object_mut()) {
        map.insert(ROLE.to_owned(), Value::String(named));
    }
    json!({
        envelope::OP: PROMPT,
        envelope::PREPARED: body,
        "goal": goal,
        "seed": Value::Null,
    })
}

#[cfg(test)]
mod tests;
