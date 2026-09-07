//! **What a verb IS**: the row, the usage line it computes, the flags its own
//! word can raise, and the one envelope it becomes.
//!
//! Split from [`super`] at the 300-line wall on the seam that module's own doc
//! draws: [`super`] is the SURFACE — which words exist, which module each row
//! lives in, and what the whole table is for — and this is the one type every
//! row is an instance of. The first moves whenever a gesture is added; the
//! second only when the shape of a gesture changes, which has happened twice.

use serde_json::{Map, Value};

use super::rows::TABLE;

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
    /// **The boolean fields this word can raise, each spelled as itself.**
    ///
    /// A gesture parameter that is a `bool` cannot be one of [`params`] — the
    /// table's rule is *a word and its parameters, all of them named strings*
    /// — and until bl-9fd1 that meant such a gesture had no typed spelling at
    /// all: `stop`'s `children`, the one control an operator reaches for when
    /// something is wrong, was reachable only by hand-writing the envelope.
    /// This is the reframe [`super`]'s own doc says to look for rather than a
    /// special case: a flag is typed AFTER the parameters, as its own field
    /// name, and raising it writes `true` under that name. Absent is absent —
    /// never `false`, which would be this seat asserting into a field nobody
    /// touched.
    ///
    /// [`params`]: Self::params
    pub flags: &'static [&'static str],
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

    /// The line an operator types, computed rather than stored — the
    /// parameters in order, then each flag in brackets because each is
    /// optional.
    pub fn usage(&self) -> String {
        std::iter::once(format!("lernie {}", self.word))
            .chain(self.params.iter().map(|p| format!("<{p}>")))
            .chain(self.flags.iter().map(|f| format!("[{f}]")))
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// **The arity refusal**, over the range the flags open. It names the
    /// range and not just its floor, for [`crate::verbs::doors::Door`]'s own
    /// reason: a sentence about one end of a range says nothing about the
    /// other.
    fn arity(&self, got: usize) -> String {
        let least = self.params.len();
        let most = least + self.flags.len();
        let takes = if least == most {
            format!("{least}")
        } else {
            format!("{least} to {most}")
        };
        format!(
            "`lernie {}` takes {takes} argument(s) and got {got} — usage: {}",
            self.word,
            self.usage()
        )
    }

    /// **The envelope this verb becomes** — the one serialization, built from
    /// the row rather than by an arm of its own.
    ///
    /// Arity is exact and refuses by name (see the module doc on why the tail
    /// is one argument here). The refusal carries this verb's own usage, so an
    /// operator learns the grammar from the mistake rather than from the source.
    pub fn envelope(&self, args: Vec<String>) -> Result<Value, String> {
        let least = self.params.len();
        if args.len() < least || args.len() > least + self.flags.len() {
            return Err(self.arity(args.len()));
        }
        // **A word past the parameters is a flag or it is a mistake**, and the
        // mistake earns the flag's own name rather than the arity sentence: a
        // tail of the right LENGTH and the wrong word is a different error
        // from a tail of the wrong length, and only one of them is answered by
        // counting.
        let mut raised: Vec<&str> = Vec::new();
        for word in args.iter().skip(least) {
            let Some(flag) = self.flags.iter().find(|known| *known == word) else {
                return Err(self.stray(word));
            };
            raised.push(flag);
        }
        Ok(self.built(args.into_iter().take(least).collect(), &raised))
    }

    /// **The refusal a tail that is none of this verb's flags earns**, which
    /// names the word it takes rather than counting: a tail of the right
    /// LENGTH and the wrong word is a different mistake from a tail of the
    /// wrong length, and only one of them is answered by an arity sentence.
    fn stray(&self, word: &str) -> String {
        format!(
            "`lernie {}` has no word {word:?} — it takes {} after its arguments; usage: {}",
            self.word,
            self.flags
                .iter()
                .map(|flag| format!("{flag:?}"))
                .collect::<Vec<String>>()
                .join(" or "),
            self.usage()
        )
    }

    /// The envelope proper, with the arity already settled. **The one
    /// builder**: [`envelope`](Self::envelope) is the checked door for argv,
    /// [`message`] and [`nudge`] are the typed doors for the window, and both
    /// arrive here — so a gesture has one spelling however it was composed.
    pub(super) fn built(&self, args: Vec<String>, raised: &[&str]) -> Value {
        Value::Object(self.fields(args, raised))
    }

    /// **The same envelope with one optional string field stated**, for the
    /// one gesture whose request carries one: `enroll`'s `address`, the route
    /// the DEVICE will dial (REMOTE §8.4 as amended, yog bl-fec6).
    ///
    /// It is a door beside [`built`](Self::built) rather than a fourth kind of
    /// row, because an optional string is not a parameter — the table's rule
    /// is *a word and its parameters, all of them named strings*, and a
    /// parameter is required by being one. **Absent is absent**: a field
    /// nobody stated is not written, never written as `null`, which would be a
    /// second spelling of the same absence the far end would then have to
    /// read (yog's codec says so from the other side).
    pub(super) fn stating(&self, args: Vec<String>, field: &str, value: Option<String>) -> Value {
        let mut map = self.fields(args, &[]);
        if let Some(stated) = value {
            map.insert(field.to_owned(), Value::String(stated));
        }
        Value::Object(map)
    }

    /// The fields themselves — the op, the parameters in order, and each flag
    /// raised. Both doors above are this map, wrapped.
    fn fields(&self, args: Vec<String>, raised: &[&str]) -> Map<String, Value> {
        let mut map = Map::new();
        map.insert(
            crate::envelope::OP.to_owned(),
            Value::String(self.word.to_owned()),
        );
        for (key, value) in self.params.iter().zip(args) {
            map.insert((*key).to_owned(), Value::String(value));
        }
        for flag in raised {
            map.insert((*flag).to_owned(), Value::Bool(true));
        }
        map
    }
}
