//! **The typed gesture surface**: the verbs an operator types, and the one
//! envelope each becomes (yog's `docs/REMOTE.md` §3; DESIGN §4.10).
//!
//! `lernie ask` takes a gesture envelope as JSON, which is the honest shape for
//! a transport with no vocabulary and a poor one for a keyboard. This is the
//! same gestures, typed — and it is a **serialization, never a second
//! implementation** (REMOTE §3: *"one dispatch surface, N serializations, never
//! two implementations"*). A verb builds the envelope
//! [`crate::envelope`] already defines and hands it to the same
//! [`crate::seat::ask`]; there is no second spelling of a gesture anywhere in
//! this crate, and `ask` stays the escape hatch for every op the table below
//! does not name — including one this build has never heard of, which REMOTE §3
//! says is not a protocol bump.
//!
//! # The table is declarative, and that is the design
//!
//! Every row is a word and its parameters **in order, all of them named
//! strings**. So there is one builder for all of them and no per-verb code to
//! drift: a verb is data. A gesture whose parameters are not all strings — a
//! boolean, a nested body — is **not added as a special case**; it goes through
//! `ask` until there is a reframe that keeps this one table, because the arm
//! that would carry it is exactly the second implementation this module exists
//! not to be.
//!
//! # The roster and the table are not one list
//!
//! The **roster** is the gestures whose replies [`crate::reply`] paints, so the
//! ask surface and the paint surface grow together: the ball that lands a pane
//! adds its kind and its gesture in the same breath. **A gesture whose reply is
//! a captured run is already painted**, which is why the conversation's four
//! acts ([`conversation`]) could be rows the day the ledger asked for them and
//! its *records* could not (bl-213c) — `steps` and `files` became rows
//! ([`records`]) in the breath that landed their decoders and the records
//! pane (bl-2cf7) — and why the tuning family's three
//! writes ([`tuning`]) could be composed the day the ledger asked for those.
//! The **table** is the subset of the roster a word can spell, and four
//! gestures cannot be one. Two modules hold them and each says why. [`start`]:
//! `prepare` carries a payload rung and `prompt` carries a prepared body, and a
//! nested object is not a word an operator types — so what argv types instead
//! is `lernie start`, the composite that spends both. [`tuning`]: `effort`
//! carries a level that is a string **or null**, where null is the whole of
//! what *off* means, and `priority` carries a bool. Each is exactly the case
//! the paragraph above refuses to special-case, and each is a typed door with
//! no row.
//!
//! **Two more doors landed with the ball pane's acts** ([`balls::edit`]):
//! `create` and `update` carry text that may be **absent**, and absence is a
//! value on this wire — a row would have to send `""` where the operator wrote
//! nothing, which asks upstream to blank a field nobody touched. Same rule as
//! the two above, third application.
//!
//! # Positional and context-free, unlike the engine's own line
//!
//! yog's line reader is terse and **context-bearing**: `/message ship it`
//! carries no address because the seat's focus supplies one. A one-shot process
//! has no focus, and REMOTE §8.5 says so directly — *"a seat with no selection
//! (argv, a fresh TUI) spells its targets out"*. Copying the line's grammar here
//! would mint a selection type that is always empty, which is a mechanism with
//! no input.
//!
//! # The four doors are words too, and they have pages
//!
//! `start`, `ask`, `entries` and `help` are answered by this binary and cannot
//! be rows here — `prepare` carries a payload rung, `prompt` carries a prepared
//! body, `ask` carries a whole envelope, and `help` takes an OPTIONAL word. The
//! table stays rows of named strings, whatever their number. What they get is
//! [`doors`]: a word, a usage line and prose, with no envelope builder behind
//! it, so `lernie help ask` answers and `lernie entries x y` is told what
//! `entries` takes rather than that it is not an argument this binary
//! recognises (bl-6bda).
//!
//! **A verbatim payload is one argument, and the shell is what makes it one.**
//! The line takes a message's content as its whole tail because a line has no
//! quoting; argv does. So `params` is exact, `lernie message w a "ship it"`
//! is the spelling, and an unquoted tail refuses by arity rather than being
//! silently joined — which would make three typed words indistinguishable from
//! one quoted sentence.

/// The balls family: four reads, three acts, and the two authoring doors.
pub mod balls;
/// The n-attempt path: spread a prepared start, accept one, release the rest.
pub mod candidates;
/// The capability boundary's three acts: the parked call, and the floor.
pub mod capability;
/// The machines registered in one workspace — the tool-host surface's one read.
pub mod clients;
/// The config family: the lineages a workspace holds, and one file's bytes.
pub mod config;
/// The conversation's own acts — what an operator does TO one, as rows.
pub mod conversation;
/// The words this binary answers itself — a page and a usage line, no envelope.
pub mod doors;
/// The one act whose subject is a box that has never connected.
pub mod enroll;
/// The fleet loop, the alignment monitor, and what a wall's agents changed.
pub mod fleet;
/// The roster and one word's page, answered here rather than by an engine.
pub mod help;
/// The sign-in family: the provider table, the offering, the act and its lane.
pub mod login;
/// The decision queue's three ops — the read, the answer and the raise.
pub mod queue;
/// The conversation's records — the reads under one, as rows.
pub mod records;
/// The reads and the deposit, as data — the rows this seat had first.
mod rows;
/// The conversation's spine — its two reads, and the fork composed off them.
pub mod spine;
/// The start family's two envelopes, which are doors without rows.
pub mod start;
/// The trail's read — a door without a row, because its bound is a number.
pub mod trail;
/// The role-tuning family: one read, and the three writes it reads back.
pub mod tuning;
/// What a verb is: the row, its usage, its flags, and the envelope it becomes.
mod verb;
/// The window's own reads — the two ops whose subject is every channel.
pub mod window;
/// The wall's own act — the one row whose product is that its subject is gone.
pub mod workspace;

pub use balls::edit::{CREATE, UPDATE, create, update};
pub use balls::{
    ASSIGN, BALLS, BOARD, CLOSE, MARKS, RELEASE, WORKSPACE_BALLS, assign, balls, board, close,
    marks, release, workspace_balls,
};
pub use candidates::{DELIVER, FAN, RETIRE, deliver, fan, retire};
pub use capability::{ANSWER, RESTORE, REVOKE, VERDICTS, answer, restore, revoke};
pub use clients::{CLIENTS, clients};
pub use config::{CONFIG, LINEAGES, Where, config, lineages, write};
pub use conversation::{
    DELETE_AGENT, INTERRUPT, RETARGET, STOP, delete_agent, interrupt, retarget, stop,
};
pub use enroll::{ADDRESS, ENROLL, enroll};
pub use fleet::{
    ARM, DISARM, DISBAND, FLEET, SCAN, SCIENCE, WORK_DIFF, arm, disarm, disband, fleet, scan,
    science, work_diff,
};
pub use login::{LOGIN, LOGIN_TAIL, MODELS, PROVIDERS, login, login_tail, models, providers};
pub use queue::{ATTENTION, FLAG, SEEN, attention, flag, seen};
pub use records::{AGENT, FILES, INBOX, STEP, STEPS, agent, files, inbox, step, steps};
pub use rows::{
    CONVERSATIONS, FOLLOW, MESSAGE, NUDGE, TRANSCRIPT, WORKSPACES, conversations, follow, message,
    nudge, transcript, workspaces,
};
pub use spine::{FORK, GOVERNING, RAIL, fork, governing, rail};
pub use start::{PREPARE, PROMPT, prepare, prompt};
pub use trail::{ACK, CLEAR_TRAIL, DEPTH, OPS, ack, clear_trail, ops};
pub use tuning::{EFFORT, MODEL, PRIORITY, ROLES, effort, model, priority, roles};
pub use verb::{Verb, find, table};
pub use window::{HELP, SEARCH, search};
pub use workspace::{DELETE_WORKSPACE, PIN, UNPIN, delete_workspace, pin, unpin};

#[cfg(test)]
mod tests;
