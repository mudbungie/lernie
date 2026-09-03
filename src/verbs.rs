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

use serde_json::{Map, Value};

/// The conversation's own acts — what an operator does TO one, as rows.
pub mod conversation;
/// The words this binary answers itself — a page and a usage line, no envelope.
pub mod doors;
/// The roster and one word's page, answered here rather than by an engine.
pub mod help;
/// The decision queue's three ops — the read, the answer and the raise.
pub mod queue;
/// The conversation's records — the reads under one, as rows.
pub mod records;
/// The reads and the deposit, as data — the rows this seat had first.
mod rows;
/// The start family's two envelopes, which are doors without rows.
pub mod start;
/// The role-tuning family: one read, and the three writes it reads back.
pub mod tuning;
/// The window's own reads — the two ops whose subject is every channel.
pub mod window;
/// The wall's own act — the one row whose product is that its subject is gone.
pub mod workspace;

pub use conversation::{
    DELETE_AGENT, INTERRUPT, RETARGET, STOP, delete_agent, interrupt, retarget, stop,
};
pub use queue::{ATTENTION, FLAG, SEEN, attention, flag, seen};
pub use records::{FILES, STEPS, files, steps};
use rows::TABLE;
pub use rows::{
    CONVERSATIONS, ENROLL, FOLLOW, MESSAGE, NUDGE, TRANSCRIPT, WORKSPACES, conversations, enroll,
    follow, message, nudge, transcript, workspaces,
};
pub use start::{PREPARE, PROMPT, prepare, prompt};
pub use tuning::{EFFORT, MODEL, PRIORITY, ROLES, effort, model, priority, roles};
pub use window::{HELP, SEARCH, search};
pub use workspace::{DELETE_WORKSPACE, delete_workspace};

/// One verb: the word, what it takes, and what it is for.
///
/// **No usage string is stored.** [`Verb::usage`] computes it from the word and
/// the parameters, so a parameter added to a row cannot leave a usage line
/// behind saying otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verb {
    /// The word typed, which is also the envelope's `op`. One fact.
    pub word: &'static str,
    /// The envelope field each argument fills, in the order they are typed.
    /// These are the **wire's** own field names, so what an operator reads in
    /// the usage is what REMOTE calls it.
    pub params: &'static [&'static str],
    /// One line: what the verb is for.
    pub summary: &'static str,
    /// The page: what it answers with, and what to know before typing it.
    pub detail: &'static str,
}

/// Every verb, in roster order.
pub fn table() -> Vec<Verb> {
    TABLE.to_vec()
}

/// The verb that word names, if it is one.
pub fn find(word: &str) -> Option<Verb> {
    TABLE.iter().find(|verb| verb.word == word).copied()
}

impl Verb {
    /// **Whether this verb addresses one workspace**, read off its own
    /// parameters rather than listed a second time.
    ///
    /// It is the predicate two surfaces need and it has one home. A verb with
    /// no `workspace` parameter has no way to name one, so its subject is
    /// *every* channel this box holds — which is why `lernie workspaces` fans
    /// (bl-0d54) where every other word goes down one channel. And a
    /// `workspace` field written onto such a gesture by hand is therefore a
    /// pure channel selector, with no reader at the far end, so a name no entry
    /// holds refuses at the seat instead of answering `ok` from a channel
    /// nobody named (bl-d574, [`crate::seat::route`]).
    pub fn addresses_a_workspace(&self) -> bool {
        self.params.contains(&crate::envelope::WORKSPACE)
    }

    /// The line an operator types, computed rather than stored.
    pub fn usage(&self) -> String {
        std::iter::once(format!("lernie {}", self.word))
            .chain(self.params.iter().map(|p| format!("<{p}>")))
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// **The envelope this verb becomes** — the one serialization, built from
    /// the row rather than by an arm of its own.
    ///
    /// Arity is exact and refuses by name (see the module doc on why the tail
    /// is one argument here). The refusal carries this verb's own usage, so an
    /// operator learns the grammar from the mistake rather than from the source.
    pub fn envelope(&self, args: Vec<String>) -> Result<Value, String> {
        if args.len() != self.params.len() {
            return Err(format!(
                "`lernie {}` takes {} argument(s) and got {} — usage: {}",
                self.word,
                self.params.len(),
                args.len(),
                self.usage()
            ));
        }
        Ok(self.built(args))
    }

    /// The envelope proper, with the arity already settled. **The one
    /// builder**: [`envelope`](Self::envelope) is the checked door for argv,
    /// [`message`] and [`nudge`] are the typed doors for the window, and both
    /// arrive here — so a gesture has one spelling however it was composed.
    fn built(&self, args: Vec<String>) -> Value {
        let mut map = Map::new();
        map.insert(
            crate::envelope::OP.to_owned(),
            Value::String(self.word.to_owned()),
        );
        for (key, value) in self.params.iter().zip(args) {
            map.insert((*key).to_owned(), Value::String(value));
        }
        Value::Object(map)
    }
}

#[cfg(test)]
mod tests;
