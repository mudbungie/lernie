//! **The conversation's records, as rows** — the reads under one conversation
//! rather than of it (bl-2cf7).
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
