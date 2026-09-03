//! **The conversation's own acts** — the four gestures an operator makes *to* a
//! conversation rather than *into* one (bl-213c).
//!
//! A second rows file, on the seam [`super::rows`]'s own doc already draws.
//! That file is the reads the window paints and the deposit it composes; this
//! is what an operator does to the conversation as an object — cut its turn
//! off, kill its driver, settle it onto another lineage, unmake it. A verb
//! added to either moves one file, which is the test that a seam is real.
//!
//! **Every one of them answers a captured run**, which is the reason these four
//! could land first: [`crate::reply::Reply::Outcome`] is a kind this
//! seat already reads and already paints, so a control that fires one of these
//! is answered rather than met with *"this build cannot read that kind"*. The
//! conversation's two everyday *records* — its steps and its files — passed
//! the same admission test when bl-2cf7 gave each a decoder and the records
//! pane ([`super::records`]). The deeper reads — its own row whole, one
//! step's drill-in, its spine, its governing commit, its inbox, and the fork
//! whose two arguments are read off the spine — still answer kinds nothing
//! here decodes, and stay recorded absent in `parity.toml`, each cited to the
//! ball that will build its surface.
//!
//! **The rows are the wire's own field names**, in the order the envelope
//! spells them, exactly as [`super::rows`] states: a `params` that drifted off
//! the wire's spelling fails in the corpus round trip rather than on a
//! connection.

use serde_json::Value;

use super::Verb;

/// **The cut.** Two acts under one word on the engine's side — the driver is
/// killed and the content deposited, stamped as one gesture — and one row here,
/// because what crosses the wire is one envelope of three named strings.
pub const INTERRUPT: Verb = Verb {
    word: "interrupt",
    params: &["workspace", "agent", "content"],
    summary: "cut a conversation off mid-work and say this instead",
    detail: "The driver is killed and the content deposited, both stamped as \
             one gesture, and the deposit's own driver-start is what carries \
             the turn on — so nothing else has to be asked for afterwards. The \
             content crosses verbatim, so quote it as one argument. The cascade \
             onto descendants is deliberately not offered: this gesture's \
             subject is the conversation being talked to, and taking a subtree \
             down is `stop`'s question, not this one's.",
};

/// **The kill.** The bare form only — see [`STOP`]'s own detail on the cascade.
pub const STOP: Verb = Verb {
    word: "stop",
    params: &["workspace", "agent"],
    summary: "kill the driver held on a conversation",
    detail: "It kills the driver and leaves everything else standing: the \
             conversation's history, its worktree and its inbox are untouched, \
             and `nudge` starts a driver on it again. This is the BARE form. \
             The wire also carries a `children` flag that takes the whole \
             subtree down, and this seat composes no gesture that raises it — a \
             cascade is a second control with a second confirmation, and it \
             belongs beside the conversation records that would say what is \
             under there.",
};

/// **The change of lineage.**
pub const RETARGET: Verb = Verb {
    word: "retarget",
    params: &["workspace", "agent"],
    summary: "settle a conversation onto the head of the config lineage governing it",
    detail: "Marks the conversation to be re-forked onto the head of the \
             config lineage that governs it, which its own executor lands at \
             the next step boundary. Nothing is rewritten now: what comes back \
             is the captured run of the marking, and the conversation keeps \
             running whatever it was running.",
};

/// **The unmaking**, and the one row here whose third parameter is an *arming*
/// rather than content: the name typed back is what admits the descendants.
pub const DELETE_AGENT: Verb = Verb {
    word: "delete-agent",
    params: &["workspace", "agent", "typed"],
    summary: "delete a conversation; the typed name arms taking its children too",
    detail: "Removes the conversation and everything the engine holds for it — \
             its ref, worktree, steps and inbox. Refused while it is live, so a \
             conversation still working is not deleted out from under itself. \
             An EMPTY `typed` deletes the one conversation; typing its name \
             exactly is what arms taking its descendants with it, and without \
             that the substrate declines a subtree nobody confirmed.",
};

/// The cut, typed. **Four doors whose arity is their signature**, on the same
/// terms as [`super::rows`]'s: the window composes each by name at compile time
/// rather than by a table lookup that could miss, so it carries no arm for a
/// refusal that cannot happen.
pub fn interrupt(workspace: String, agent: String, content: String) -> Value {
    INTERRUPT.built(vec![workspace, agent, content])
}

/// The kill, typed.
pub fn stop(workspace: String, agent: String) -> Value {
    STOP.built(vec![workspace, agent])
}

/// The change of lineage, typed.
pub fn retarget(workspace: String, agent: String) -> Value {
    RETARGET.built(vec![workspace, agent])
}

/// The unmaking, typed. `typed` is the arming and an empty string is the bare
/// form, which is the wire's own grammar rather than a spelling invented here.
pub fn delete_agent(workspace: String, agent: String, typed: String) -> Value {
    DELETE_AGENT.built(vec![workspace, agent, typed])
}
