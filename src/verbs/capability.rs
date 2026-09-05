//! **The capability boundary's three acts** — what an operator does about a
//! conversation's tool calls (yog's `docs/REMOTE.md` §5, §9.11; bl-bce2).
//!
//! A fifth rows file, on the seam [`super::rows`]'s own doc keeps drawing.
//! [`super::conversation`] is what an operator does TO a conversation,
//! [`super::records`] is what they look under one for, [`super::spine`] is
//! what its history is anchored to — and this is what they do about the
//! **boundary** it runs behind: release the one call that is waiting, or take
//! away and give back the standing permission to make them at all.
//!
//! **One call and one policy, which is why there are two receipts.**
//! [`ANSWER`] is about the invocation parked right now — read off the
//! conversation's own hold mark at fire time, so nothing is typed and no other
//! call can be spent by it — and it DRIVES the conversation on where it
//! releases. [`REVOKE`] and [`RESTORE`] write standing policy and launch
//! nothing; upstream is explicit that a restore *"drives nothing — a
//! conversation parked at a held call is released by answering that call"*.
//!
//! **Neither of the two can be refused, and that is why both are always
//! offered.** A floor is a row appended to the engine's trail and the reply is
//! re-derived from that trail afterwards, so `restore` on a conversation that
//! was never floored is not an error and `restore` under a still-floored
//! ancestor leaves the floor standing and SAYS so. There is no rank to read
//! and nothing to guess (`crate::ui::composer::acts`).

use serde_json::Value;

use super::Verb;

/// **The answer to the one call that is parked.**
pub const ANSWER: Verb = Verb {
    word: "answer",
    params: &["workspace", "agent", "verdict"],
    summary: "release, decline or keep parked the tool call held at this conversation",
    detail: "Answers the invocation the capability boundary parked before it \
             ran. `pass` lets that one call through, `refuse` declines it in \
             band — the model reads why and carries on — and `hold` keeps it \
             parked even if the policy later would have passed it. The answer \
             is scoped to the exact call that is held, which is read from the \
             conversation's own hold mark, so nothing is typed and nothing can \
             be spent by a different call. Passing or refusing then drives the \
             conversation on, which is what actually lifts the hold. Nothing \
             here stops the agent. Refused when nothing is held there.",
};

/// **The floor, raised.**
pub const REVOKE: Verb = Verb {
    word: "revoke",
    params: &["workspace", "agent"],
    summary: "take away this conversation's tool auto-approval, and its descendants'",
    detail: "Stops letting the conversation act on its own: from its next tool \
             call, everything but a read waits for you. It keeps running, \
             keeps its branch and keeps reading, so nothing is lost and \
             nothing is killed. It covers the conversation and everything \
             below it, including children it has not spawned yet. Anything the \
             policy already refuses stays refused, and a call passed with \
             `answer` still goes through.",
};

/// **The floor, lowered.**
pub const RESTORE: Verb = Verb {
    word: "restore",
    params: &["workspace", "agent"],
    summary: "give this conversation's tool auto-approval back",
    detail: "Lifts a floor `revoke` put on the conversation: its calls are \
             adjudicated by the ordinary policy again, from its next one. It \
             drives nothing — a conversation parked at a held call is released \
             by answering that call. If an ancestor is still revoked the \
             conversation stays floored under it, and the reply says so rather \
             than claiming a restore it did not make.",
};

/// **The three verdicts, in the order a control offers them.**
///
/// The wire's own words and not a translation of them, on
/// [`super::tuning::levels`]'s own terms: what a control paints is what the
/// envelope carries, so no table exists to drift. They are ordered by what
/// they do to the conversation — the two that move it, then the one that does
/// not — which is the composer's own split between its two rows.
pub const VERDICTS: [&str; 3] = ["pass", "refuse", "hold"];

/// The answer, typed.
pub fn answer(workspace: String, agent: String, verdict: String) -> Value {
    ANSWER.built(vec![workspace, agent, verdict])
}

/// The floor, raised.
pub fn revoke(workspace: String, agent: String) -> Value {
    REVOKE.built(vec![workspace, agent])
}

/// The floor, lowered.
pub fn restore(workspace: String, agent: String) -> Value {
    RESTORE.built(vec![workspace, agent])
}

#[cfg(test)]
mod tests;
