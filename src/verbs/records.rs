//! **The conversation's records, as rows** — the reads under one conversation
//! rather than of it (bl-2cf7, bl-3257).
//!
//! A third rows file, on the seam [`super::rows`]'s own doc draws twice over:
//! that file is the reads the window's three panes stand on, and
//! [`super::conversation`] is what an operator does *to* a conversation. This
//! is what an operator looks *under* one for — what the loop did, what it
//! touched — and the records pane (`crate::ui::records`) is the surface both
//! rows exist for: each answers a kind `crate::reply` now decodes, which is
//! the same admission test the conversation's four acts passed (a reply this
//! seat cannot paint makes a control that only looks actionable).
//!
//! **The rows are the wire's own field names**, in the order the envelope
//! spells them; a `params` drifted off the wire's spelling fails in the corpus
//! round trip rather than on a connection.

use serde_json::Value;

use super::Verb;

/// **The steps ledger.** One row per step the conversation's loop has taken.
pub const STEPS: Verb = Verb {
    word: "steps",
    params: &["workspace", "agent"],
    summary: "the steps a conversation's loop has taken",
    detail: "One row per step, in sequence order: how it ended, how many \
             attempts it took, what it cost in tokens, its timestamps and \
             read-state commit where the step's record carried them, whether \
             a sign-in is wanted, and the wound where one was taken. The \
             orphaned-tail state rides at the top because it is the \
             conversation's fact rather than any one step's. The `seq` it \
             answers with is the address the `step` drill-in takes.",
};

/// **The worktree listing.** What the conversation's working tree holds now.
pub const FILES: Verb = Verb {
    word: "files",
    params: &["workspace", "agent"],
    summary: "what a conversation's worktree holds",
    detail: "The walked worktree: each entry's path, size and kind, whether \
             the walk was cut short, and — where the conversation's work \
             lands somewhere this listing does not reach — the working \
             directory that does hold it. A torn-down worktree answers as \
             its absence, never as an empty listing. This is the bare form; \
             the wire also takes `at` (pin a commit) and `path` (preview one \
             file), which this seat does not compose — \
             `lernie ask` carries them.",
};

/// The steps ledger, typed — a door whose arity is its signature, on the same
/// terms as every other row's.
pub fn steps(workspace: String, agent: String) -> Value {
    STEPS.built(vec![workspace, agent])
}

/// The worktree listing, typed. The bare read only: a commit pin or a file
/// preview is a control the records pane does not have yet (see [`FILES`]).
pub fn files(workspace: String, agent: String) -> Value {
    FILES.built(vec![workspace, agent])
}

/// **The conversation's own row.** The deepest read of the same subject, and a
/// row here rather than in [`super::conversation`] for this file's own reason:
/// an operator does not do it TO the conversation, they look at it.
pub const AGENT: Verb = Verb {
    word: "agent",
    params: &["workspace", "agent"],
    summary: "the conversation's own row, whole",
    detail: "Everything the engine holds about one conversation rather than \
             the glance a listing gives: its descent, the name it answers to \
             and whether that name is addressable, its own liveness and \
             whether the last turn was refused at the provider, what it has \
             spent and how full its context is, what is in flight on it right \
             now, the marks it wears and the tool call parked at its \
             capability boundary. Every field is a fold the engine already \
             owned; what was missing was the spelling.",
};

/// **One step's drill-in**, addressed by the sequence the steps ledger paints.
pub const STEP: Verb = Verb {
    word: "step",
    params: &["workspace", "agent", "seq"],
    summary: "one step's records, drilled in",
    detail: "The tier under `steps`, named by the sequence that list shows \
             (`001`): that step's metadata, the wire request that was sent, \
             the transcript entry that was staged, every event of the \
             response stream, and every tool call's input and output — each \
             as parsed data with the bytes it parsed from beside it — plus \
             the captured logs that have bytes. A record that is missing says \
             so, and one that is not JSON comes back verbatim and framed as \
             unparseable rather than dropped. A step the tree does not hold \
             answers absent records rather than refusing.",
};

/// **The undelivered mail.**
pub const INBOX: Verb = Verb {
    word: "inbox",
    params: &["workspace", "agent"],
    summary: "the mail still waiting in a conversation's inbox",
    detail: "Every deposit sitting in the conversation's inbox: who sent it, \
             when, the body, and — on a subagent's result message — how that \
             agent ended and the commit it ended at. The parse is forgiving, \
             so a half-written or hand-edited deposit is rendered with \
             whatever fields it actually stated rather than refused, and each \
             row carries its file's bytes beside the reading. Delivered mail \
             is not here; it has moved into the transcript.",
};

/// The conversation's own row, typed.
pub fn agent(workspace: String, agent: String) -> Value {
    AGENT.built(vec![workspace, agent])
}

/// One step's drill-in, typed — `seq` is the address the ledger's rows paint.
pub fn step(workspace: String, agent: String, seq: String) -> Value {
    STEP.built(vec![workspace, agent, seq])
}

/// The undelivered mail, typed.
pub fn inbox(workspace: String, agent: String) -> Value {
    INBOX.built(vec![workspace, agent])
}
