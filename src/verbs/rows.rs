//! **The rows** — the reads, the deposit, the advance and the enrollment, as
//! data. The conversation's own four acts are [`super::conversation`]'s and the
//! tuning family's two are [`super::tuning`]'s.
//!
//! Split from [`super`] at the design-time budget on the seam the module's own
//! doc already draws: [`super`] is what a verb *is* and what it does with its
//! arguments, and this is *which verbs there are*. A verb added moves this
//! file and nothing else, which is the test that a seam is real.
//!
//! **[`TABLE`] is the whole roster and twenty-two of its rows are declared
//! elsewhere**, in the eight files that own their subjects. The balls family's
//! four — the box-wide binding table, the fleet board, one wall's own balls
//! and the branch it tracks them on — are [`super::balls`]'s, on the terms
//! every group below is: one subject, one file, whatever the mix of widths.
//! The conversation's
//! four are [`super::conversation`]'s, its records' two are
//! [`super::records`]'s, its spine's two are [`super::spine`]'s — beside the
//! `fork` door that shares their subject and cannot be a row, because it
//! carries a list — and the decision queue's three are [`super::queue`]'s.
//! The tuning family's `roles` and `model`
//! are [`super::tuning`]'s, beside the two doors that share their subject and
//! cannot be rows at all — the read and the three writes are one
//! `providers.yaml` assignment seen from both ends, and splitting them across
//! two files to keep this one tidy would have split the fact. The sign-in
//! family's four are [`super::login`]'s, on the same terms: the table, one
//! row's offering, the act that starts a run in the wall and the lane that
//! streams it are one subject, and three of the four are reads while one is an
//! act. What stays here is the enumeration: every verb this binary has a word
//! for is in the one list below.
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

/// The enrollment, on the same terms — the window composes it from a name and
/// a grade the operator chose, and argv from three words.
pub fn enroll(workspace: String, name: String, grade: String) -> Value {
    ENROLL.built(vec![workspace, name, grade])
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
                 no address, so its subject is EVERY channel this box holds: it \
                 asks each in turn — this box's own engine first, then the \
                 entries in leaf order — and prints the union under the name of \
                 the channel each answer came from, which is the roster the \
                 window paints. A channel that will not answer says so in its \
                 own section while the others still answer. To ask exactly one, \
                 name it: `lernie ask '{\"op\":\"workspaces\",\"workspace\":\"<entry>\"}'`.",
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

/// **The enrollment's row.** Three named strings, so it is a row like any
/// other — but the *reply* is not like any other, and `lernie enroll` therefore
/// has its own arm in [`crate::cli`] rather than printing the reply stream the
/// way every other verb does. The material must not reach a scrollback; what
/// the arm prints is the symbol.
pub const ENROLL: Verb = Verb {
    word: "enroll",
    params: &["workspace", "name", "grade"],
    summary: "mint a new box's material and show it as a code to photograph",
    detail: "The engine mints a leaf on its own CA, seats the client in that \
             workspace, answers the material and shreds the key. This seat \
             prints the answer as a QR symbol and keeps NOTHING: not a file, \
             not a cache, not a log line. `grade` is `operator` or `foot`. It \
             is refused unless this box's own leaf is operator-grade — the new \
             box says nothing and performs no act, which is why this is not the \
             in-channel bootstrap REMOTE §1.4 forbids.",
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
/// first — and the widest of all is the decision queue, which names no
/// workspace at all; then the conversation's own acts, the deposit first and
/// the unmaking last, with the queue's two writes among them because their
/// subject is one conversation; then the one act whose subject is a box.
///
/// The conversation's four live in [`super::conversation`], its records' two in
/// [`super::records`], the queue's three in [`super::queue`] and the wall's one
/// in [`super::workspace`], named here rather than defined here — one table,
/// six files, on the seams those files' own docs draw. [`super::window`]'s
/// other member has no row and says why: its word is `lernie help`'s.
pub(super) const TABLE: &[Verb] = &[
    WORKSPACES,
    super::queue::ATTENTION,
    super::balls::BALLS,
    super::balls::BOARD,
    super::window::SEARCH,
    CONVERSATIONS,
    super::balls::WORKSPACE_BALLS,
    super::balls::MARKS,
    super::fleet::SCIENCE,
    super::fleet::WORK_DIFF,
    TRANSCRIPT,
    FOLLOW,
    super::records::AGENT,
    super::records::STEPS,
    super::records::STEP,
    super::records::FILES,
    super::records::INBOX,
    super::spine::RAIL,
    super::spine::GOVERNING,
    super::tuning::ROLES,
    super::clients::CLIENTS,
    super::config::LINEAGES,
    super::login::PROVIDERS,
    super::login::MODELS,
    super::login::LOGIN_TAIL,
    MESSAGE,
    super::conversation::INTERRUPT,
    NUDGE,
    super::conversation::STOP,
    super::conversation::RETARGET,
    super::queue::FLAG,
    super::queue::SEEN,
    super::capability::ANSWER,
    super::capability::REVOKE,
    super::capability::RESTORE,
    super::conversation::DELETE_AGENT,
    ENROLL,
    super::login::LOGIN,
    super::tuning::MODEL,
    super::workspace::PIN,
    super::workspace::UNPIN,
    super::trail::ACK,
    super::fleet::SCAN,
    super::fleet::ARM,
    super::fleet::DISARM,
    super::fleet::DISBAND,
    super::trail::CLEAR_TRAIL,
    super::workspace::DELETE_WORKSPACE,
];
