//! **The workspace's own three acts** — the unmaking whose product is that its
//! subject is gone (bl-48fa), and the pin pair that orders the strip every seat
//! paints (bl-7782).
//!
//! It is a file of its own beside [`super::conversation`] on that module's own
//! seam: a row lives with the noun it acts on, and every other row here acts on
//! a conversation, a conversation's records, the queue or a box. These three act
//! on the wall.
//!
//! # The pin pair are ASSERTIONS and not a toggle
//!
//! yog's own help row says what they are: *"Says what it means rather than
//! flipping whatever it found: unpinning one that is not pinned leaves the list
//! alone, which is what lets two seats send it at once and agree."* So there is
//! no `pin` taking a bool, and a seat must not compose one from a flag it read
//! a beat ago — it sends the act it means, and two seats meaning the same thing
//! converge. The control shapes itself to the row's own rank
//! (`crate::ui::roster`), which is a reading of what is on the glass rather than
//! a state this end holds.
//!
//! **Both answer the workspace listing** — the roster, with the ranks it now
//! carries — so nothing new is decoded for them and the next answer is what
//! says the act landed. A pin is *an assertion about the world, not an
//! arrangement of one screen*: the same list on every seat, surviving a
//! restart, which is why it crosses the boundary at all instead of being a
//! local sort.
//!
//! # `typed` is an ARMING here and a PARAMETER next door
//!
//! Both unmakings carry a field spelled `typed` and the two are not the same
//! kind of thing, which is the whole of what DESIGN §4.20 rules on:
//!
//! - [`super::conversation::DELETE_AGENT`] takes it as a **parameter**. An
//!   empty one deletes the one conversation and the name typed back is what
//!   admits its descendants, so both values are gestures somebody meant.
//! - this row takes it as an **arming**. There is one value the engine
//!   accepts — the workspace's own name — and every other value, the empty
//!   string included, is refused. So a seat that offered the act with the box
//!   empty would be offering a refusal.
//!
//! Nothing here decides that. It is the wire's grammar, read off yog's own help
//! row, and the seat's control shapes itself to which of the two it is
//! (`crate::ui::unmake`).

use serde_json::Value;

use super::Verb;

/// **The unmaking of a wall.** The one row whose subject is the workspace
/// itself, and the one whose third state — refused — is the common one.
pub const DELETE_WORKSPACE: Verb = Verb {
    word: "delete-workspace",
    params: &["workspace", "typed"],
    summary: "unmake a workspace; the typed name is the arming",
    detail: "It unmakes the workspace and releases the balls it held. Fail-closed \
             at fire time wherever it is asked: refused unless the workspace is \
             the engine's own, nothing in it is live, and `typed` matches the \
             workspace's name exactly. The typed name is therefore an ARMING and \
             not content — there is one value it accepts, and an empty one is a \
             refusal rather than a bare form.",
};

/// The unmaking, typed. `typed` is the arming, and the caller sends what it
/// armed: an engine comparing a name against a value this end trimmed after
/// checking it would be two spellings of one string.
pub fn delete_workspace(workspace: String, typed: String) -> Value {
    DELETE_WORKSPACE.built(vec![workspace, typed])
}

/// **The pin.** Float this wall to the front of the strip and keep it there.
pub const PIN: Verb = Verb {
    word: "pin",
    params: &["workspace"],
    summary: "float this workspace to the front of the strip and keep it there",
    detail: "It adds the workspace to the durable pin list every seat orders              its strip by: pinned first, in the order they were pinned, ahead              of the rest in name order. A pin is an assertion about the world              rather than an arrangement of one screen, so it is the same list              on every seat and it survives a restart. Pinning one already              pinned moves it to the end of the pinned run rather than saying              it twice. It answers the workspace listing with the ranks it now              carries.",
};

/// **The unpin.** Take it back out of the pinned run.
pub const UNPIN: Verb = Verb {
    word: "unpin",
    params: &["workspace"],
    summary: "take this workspace back out of the pinned run",
    detail: "It removes the workspace from the pin list, so it falls back into              name order with everything unpinned. It says what it means rather              than flipping whatever it found: unpinning one that is not pinned              leaves the list alone, which is what lets two seats send it at              once and agree. It answers the workspace listing with the ranks it              now carries.",
};

/// The pin, typed.
pub fn pin(workspace: String) -> Value {
    PIN.built(vec![workspace])
}

/// The unpin, typed.
pub fn unpin(workspace: String) -> Value {
    UNPIN.built(vec![workspace])
}

#[cfg(test)]
mod tests;
