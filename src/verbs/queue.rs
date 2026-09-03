//! **The decision queue's three ops** — the read, the answer, and the raise
//! (yog's `docs/REMOTE.md` §6, §9.11; bl-f0ef).
//!
//! A fourth rows file, on the seam [`super::rows`]'s doc already draws three
//! times: these are the ops whose subject is *what is waiting on the operator*
//! rather than one wall, one conversation or one config. The queue pane
//! (`crate::ui::queue`) is the surface all three exist for, and each answers a
//! kind `crate::reply` now decodes — the same admission test the conversation's
//! acts and its records passed.
//!
//! **[`ATTENTION`] names no workspace, so its subject is every channel this box
//! holds** — read off its own empty `params` and nowhere else
//! (`crate::verbs::Verb::addresses_a_workspace`), exactly as `workspaces` is.
//! The window asks it once per channel and paints the union; `lernie attention`
//! fans over the same set and prints each answer under the channel it came
//! down.
//!
//! **[`SEEN`] answers with a queue rather than a receipt**, which is why it is
//! here beside the read instead of among the conversation's acts: the reply is
//! `reply/attention`, the queue that remains, and a seat that filed it as a
//! receipt would discard the one statement it gets about what is still asking.
//!
//! **The rows are the wire's own field names**, in the order the envelope
//! spells them; a `params` drifted off the wire's spelling fails in the corpus
//! round trip rather than on a connection.

use serde_json::Value;

use super::Verb;

/// **The whole queue.** One row per conversation asking, anywhere.
pub const ATTENTION: Verb = Verb {
    word: "attention",
    params: &[],
    summary: "everything waiting on you, across every workspace",
    detail: "One row per conversation asking for you, anywhere an engine this \
             box holds can see: why it is asking, what it last said, how long \
             it has waited, whether a flag was raised on it and in whose \
             words, and the workspace and conversation to aim an answer at. It \
             takes no address, so its subject is EVERY channel this box holds: \
             it asks each in turn and prints the union under the name of the \
             channel each answer came from. Answer a row with `message`, \
             `stop` or `seen`.",
};

/// **Raise one.** The write that puts a conversation on the queue above.
pub const FLAG: Verb = Verb {
    word: "flag",
    params: &["workspace", "agent", "reason"],
    summary: "raise an attention item on a conversation, with a reason",
    detail: "It records that this conversation wants a human look, and why, \
             in the raiser's own words. It changes nothing else — it does not \
             stop, message or touch the conversation — and what it changed \
             shows up as a `flag` on that conversation's `attention` row. The \
             reason is required: a flag with no words is a row an operator \
             cannot triage.",
};

/// **Answer one.** What takes a row off the queue.
pub const SEEN: Verb = Verb {
    word: "seen",
    params: &["workspace", "agent"],
    summary: "answer a conversation's place in the attention queue",
    detail: "It records what the conversation is currently asking about as \
             seen, which is what takes it off the queue — the same watermarks \
             a window writes simply by having that conversation open. It \
             quiets what the conversation has SAID; undelivered mail is not a \
             watermark and clears only when a driver reads it, and new \
             evidence re-raises the row. It answers with the queue that \
             remains, not with a receipt.",
};

/// The queue read, typed — a door whose arity is its signature, on the same
/// terms as every other row's.
pub fn attention() -> Value {
    ATTENTION.built(Vec::new())
}

/// The raise, typed.
pub fn flag(workspace: String, agent: String, reason: String) -> Value {
    FLAG.built(vec![workspace, agent, reason])
}

/// The answer, typed.
pub fn seen(workspace: String, agent: String) -> Value {
    SEEN.built(vec![workspace, agent])
}
