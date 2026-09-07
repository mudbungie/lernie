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
//! # The answer has a REACH, and it is required in both directions
//!
//! [`SCOPES`] (PROTOCOL 18, yog bl-94a5) is how far one answer stands: the
//! held `call`, the `conversation` and its descent, or the whole `workspace`.
//! An answer with no scope settled one call, so an operator holding a
//! conversation answered the same question for every call of a kind they had
//! already decided about — the wider two are how that stops.
//!
//! **`scope` is stated on every gesture this seat composes**, `call` included,
//! because the wire requires it in both directions rather than defaulting: an
//! absent field would let the two ends disagree about how wide the instruction
//! was, and *wider* is the reading nobody may arrive at by accident. What is
//! optional is the operator's TYPING of it ([`answer`]'s `None`), which is a
//! different thing from the field being optional.
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
    flags: &[],
    summary: "release, decline or keep parked the tool call held at this conversation",
    detail: "Answers the invocation the capability boundary parked before it \
             ran. `pass` lets that one call through, `refuse` declines it in \
             band — the model is told the decision stands and told not to \
             retry it, rephrase it or reach the same outcome another way — and \
             `hold` keeps it parked even if the policy later would have passed \
             it. Which call is answered is read from the conversation's own \
             hold mark, so nothing is typed and nothing can be spent by a \
             different call. A fourth word says how far the answer stands. \
             Left off it settles that one call, which is the safe reading and \
             the one you get without asking for anything; `conversation` \
             settles the whole class of the held call — the same tool at the \
             same reach — for this conversation and everything below it, and \
             `workspace` settles that class for every conversation there. A \
             wider answer is how you stop being asked eleven times about one \
             narrow tool; `revoke` suspends every standing answer, which is \
             how you take one back. A destructive or credential-reaching call \
             takes the bare form only. Passing or refusing then drives the \
             conversation on, which is what actually lifts the hold. Nothing \
             here stops the agent. Refused when nothing is held there.",
};

/// **The floor, raised.**
pub const REVOKE: Verb = Verb {
    word: "revoke",
    params: &["workspace", "agent"],
    flags: &[],
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
    flags: &[],
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

/// **The gesture's fourth field**, spelled once so the door that states it and
/// the reading that comes back cannot disagree about its name.
pub const SCOPE: &str = "scope";

/// **The three reaches an answer can have** (PROTOCOL 18), narrowest first —
/// which is the order they may be offered in and the order they are safe in.
///
/// The wire's own words again, on [`VERDICTS`]'s terms, so no table exists to
/// drift. [`CALL`] is the first of them and the one an unstated answer means.
pub const SCOPES: [&str; 3] = ["call", "conversation", "workspace"];

/// **What an answer that says nothing about its reach means**: the one held
/// call, which is what every answer was before 18 and is still the reading an
/// operator gets without asking for another.
pub const CALL: &str = "call";

/// The answer, typed. **The scope is always stated** — `None` is the operator
/// not typing one, not the field going missing — because the wire requires it
/// in both directions and an absent field would let the two ends disagree
/// about how wide the instruction was.
pub fn answer(workspace: String, agent: String, verdict: String, scope: Option<String>) -> Value {
    ANSWER.stating(
        vec![workspace, agent, verdict],
        SCOPE,
        Some(scope.unwrap_or_else(|| CALL.to_owned())),
    )
}

/// The floor, raised.
pub fn revoke(workspace: String, agent: String) -> Value {
    REVOKE.built(vec![workspace, agent], &[])
}

/// The floor, lowered.
pub fn restore(workspace: String, agent: String) -> Value {
    RESTORE.built(vec![workspace, agent], &[])
}

#[cfg(test)]
mod tests;
