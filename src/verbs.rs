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
//! # Six verbs, and they are the six the seat can read the answers to
//!
//! The roster is not "everything the boundary has". It is the gestures whose
//! replies [`crate::reply`] paints — the four reads and the two acts that move
//! a conversation — so the ask surface and the paint surface are one roster and
//! grow together. The ball that lands a pane adds its kind and its verb in the
//! same breath.
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
//! **A verbatim payload is one argument, and the shell is what makes it one.**
//! The line takes a message's content as its whole tail because a line has no
//! quoting; argv does. So `params` is exact, `lernie message w a "ship it"`
//! is the spelling, and an unquoted tail refuses by arity rather than being
//! silently joined — which would make three typed words indistinguishable from
//! one quoted sentence.

use serde_json::{Map, Value};

/// The roster and one verb's page, answered here rather than by an engine.
pub mod help;

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

/// Every verb, in the order the roster prints them: the reads first, widest
/// first, then the two acts.
const TABLE: &[Verb] = &[
    Verb {
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
    },
    Verb {
        word: "conversations",
        params: &["workspace"],
        summary: "one workspace's conversations",
        detail: "The rows a window's middle pane paints: each conversation's \
                 label, its state, a first-line preview, its age and how far it \
                 hangs under its root. The id it answers with is the address \
                 every other verb here takes.",
    },
    Verb {
        word: "transcript",
        params: &["workspace", "agent"],
        summary: "one conversation, committed entries and the live tail",
        detail: "The whole conversation as of now — the delivered messages, the \
                 model's turns and their tool calls, the results, whatever the \
                 compactor squashed, and the tail of a turn still in flight. It \
                 answers once and returns; `follow` is the same subject held \
                 open.",
    },
    Verb {
        word: "follow",
        params: &["workspace", "agent"],
        summary: "hold the line on one conversation's live tail",
        detail: "A read that deliberately never finishes: the connection stays \
                 open and the engine writes a frame every time the tail moves. \
                 Each frame is the WHOLE accumulated fold rather than a delta, \
                 so a frame missed is nothing missed. It ends when the engine \
                 ends it, or when this end hangs up.",
    },
    Verb {
        word: "message",
        params: &["workspace", "agent", "content"],
        summary: "deposit a message into a conversation",
        detail: "The content crosses verbatim — nothing here trims, wraps or \
                 normalises it — so quote it as one argument. It answers with \
                 the deposit's captured run, and the turn it triggers arrives on \
                 the transcript at its own pace.",
    },
    Verb {
        word: "nudge",
        params: &["workspace", "agent"],
        summary: "start a driver on a conversation that has gone quiet",
        detail: "It launches the advance and answers at once, carrying nothing \
                 else, because there is nothing else yet: what the model does \
                 with the turn arrives on the transcript, and a receipt that \
                 guessed at it here would be a receipt that lied.",
    },
];

/// Every verb, in roster order.
pub fn table() -> Vec<Verb> {
    TABLE.to_vec()
}

/// The verb that word names, if it is one.
pub fn find(word: &str) -> Option<Verb> {
    TABLE.iter().find(|verb| verb.word == word).copied()
}

impl Verb {
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
        let mut map = Map::new();
        map.insert(
            crate::envelope::OP.to_owned(),
            Value::String(self.word.to_owned()),
        );
        for (key, value) in self.params.iter().zip(args) {
            map.insert((*key).to_owned(), Value::String(value));
        }
        Ok(Value::Object(map))
    }
}

#[cfg(test)]
mod tests;
