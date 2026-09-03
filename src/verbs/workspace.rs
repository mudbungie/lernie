//! **The workspace's own act**, and the only row in this table whose product is
//! that its subject is gone (bl-48fa).
//!
//! It is a file of its own beside [`super::conversation`] on that module's own
//! seam: a row lives with the noun it acts on, and every other row here acts on
//! a conversation, a conversation's records, the queue or a box. This one acts
//! on the wall.
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
