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
//! # Six rows, and eight gestures — the roster and the table are not one list
//!
//! The **roster** is the gestures whose replies [`crate::reply`] paints, so the
//! ask surface and the paint surface grow together: the ball that lands a pane
//! adds its kind and its gesture in the same breath. The **table** is the
//! subset of those a word can spell, and it is six because two of them cannot
//! be one. [`start`] holds the pair and says why: `prepare` carries a payload
//! rung and `prompt` carries a prepared body, and a nested object is not a word
//! an operator types — which is exactly the case the paragraph above refuses to
//! special-case. They are typed doors with no row, and what argv types instead
//! is `lernie start`, the composite that spends both.
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
/// The rows themselves — the six verbs, as data.
mod rows;
/// The start family's two envelopes, which are doors without rows.
pub mod start;

use rows::TABLE;
pub use rows::{
    CONVERSATIONS, FOLLOW, MESSAGE, NUDGE, TRANSCRIPT, WORKSPACES, conversations, follow, message,
    nudge, transcript, workspaces,
};
pub use start::{PREPARE, PROMPT, prepare, prompt};

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
