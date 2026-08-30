//! **The rows** — the six verbs, as data.
//!
//! Split from [`super`] at the design-time budget on the seam the module's own
//! doc already draws: [`super`] is what a verb *is* and what it does with its
//! arguments, and this is *which verbs there are*. A verb added moves this
//! file and nothing else, which is the test that a seam is real.
//!
//! **Every row is also a `pub const`, with a typed door beside it**, because
//! the window and the off-frame threads compose gestures by name at compile
//! time rather than by a table lookup that could miss. That is what lets each
//! door take its parameters as named arguments — an arity that cannot be wrong,
//! so no caller has an arm for a refusal that cannot happen — while the rows
//! themselves stay in the one table `lernie help` prints.

use serde_json::Value;

use super::Verb;

/// **The deposit the window composes**, and the one place the two faces meet.
///
/// A typed door onto the same row `lernie message` spends, so a click and a
/// typed command build one object. **Its arity cannot be wrong**, because the
/// parameters are named arguments in the signature — which is why the window
/// has no arm for a refusal that cannot happen. `src/verbs/tests.rs` pins it
/// against the row, so a reordered `params` fails there rather than silently
/// mis-addressing a deposit.
pub fn message(workspace: String, agent: String, content: String) -> Value {
    MESSAGE.built(vec![workspace, agent, content])
}

/// The advance, on the same terms.
pub fn nudge(workspace: String, agent: String) -> Value {
    NUDGE.built(vec![workspace, agent])
}

/// The `workspaces` read's row.
pub const WORKSPACES: Verb = Verb {
    word: "workspaces",
    params: &[],
    summary: "every workspace this engine holds, with its rollups",
    detail: "The roster, and the whole of what a window's first pane is. It \
                 names each workspace, how it is classified, how many \
                 conversations it holds, how many want attention, whether \
                 anything is running, and where the operator pinned it. It takes \
                 no address: a read with no workspace goes to this box's own \
                 engine, and a workspace held elsewhere is reached by naming it \
                 to one of the verbs below.",
};

/// The `conversations` read's row.
pub const CONVERSATIONS: Verb = Verb {
    word: "conversations",
    params: &["workspace"],
    summary: "one workspace's conversations",
    detail: "The rows a window's middle pane paints: each conversation's \
                 label, its state, a first-line preview, its age and how far it \
                 hangs under its root. The id it answers with is the address \
                 every other verb here takes.",
};

/// The `transcript` read's row.
pub const TRANSCRIPT: Verb = Verb {
    word: "transcript",
    params: &["workspace", "agent"],
    summary: "one conversation, committed entries and the live tail",
    detail: "The whole conversation as of now — the delivered messages, the \
                 model's turns and their tool calls, the results, whatever the \
                 compactor squashed, and the tail of a turn still in flight. It \
                 answers once and returns; `follow` is the same subject held \
                 open.",
};

/// The `follow` read's row.
pub const FOLLOW: Verb = Verb {
    word: "follow",
    params: &["workspace", "agent"],
    summary: "hold the line on one conversation's live tail",
    detail: "A read that deliberately never finishes: the connection stays \
                 open and the engine writes a frame every time the tail moves. \
                 Each frame is the WHOLE accumulated fold rather than a delta, \
                 so a frame missed is nothing missed. It ends when the engine \
                 ends it, or when this end hangs up.",
};

/// The roster read, typed. **Four reads and two acts, each a door whose arity
/// is its signature** — so the window composes a gesture without a table lookup
/// that could miss, and without an arm for a refusal that cannot happen.
pub fn workspaces() -> Value {
    WORKSPACES.built(Vec::new())
}

/// One workspace's conversations.
pub fn conversations(workspace: String) -> Value {
    CONVERSATIONS.built(vec![workspace])
}

/// One conversation, as committed.
pub fn transcript(workspace: String, agent: String) -> Value {
    TRANSCRIPT.built(vec![workspace, agent])
}

/// The held read on one conversation's live tail.
pub fn follow(workspace: String, agent: String) -> Value {
    FOLLOW.built(vec![workspace, agent])
}

/// The deposit's row.
pub const MESSAGE: Verb = Verb {
    word: "message",
    params: &["workspace", "agent", "content"],
    summary: "deposit a message into a conversation",
    detail: "The content crosses verbatim — nothing here trims, wraps or \
             normalises it — so quote it as one argument. It answers with the \
             deposit's captured run, and the turn it triggers arrives on the \
             transcript at its own pace.",
};

/// The advance's row.
pub const NUDGE: Verb = Verb {
    word: "nudge",
    params: &["workspace", "agent"],
    summary: "start a driver on a conversation that has gone quiet",
    detail: "It launches the advance and answers at once, carrying nothing \
             else, because there is nothing else yet: what the model does with \
             the turn arrives on the transcript, and a receipt that guessed at \
             it here would be a receipt that lied.",
};

/// Every verb, in the order the roster prints them: the reads first, widest
/// first, then the two acts.
pub(super) const TABLE: &[Verb] = &[
    WORKSPACES,
    CONVERSATIONS,
    TRANSCRIPT,
    FOLLOW,
    MESSAGE,
    NUDGE,
];
