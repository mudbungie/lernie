//! **The structural doors**: the words this binary answers itself, which are
//! not gestures and cannot be rows of the gesture table.
//!
//! [`super`] is emphatic that its table is data — every row a word and its
//! parameters, *"all of them named strings"* — and that `start` and `ask`
//! cannot be rows of it precisely because their arguments are not: a nested
//! object is not a word an operator types. That reasoning holds and this
//! module does not widen it. What it fixes is that the help surface did not
//! know it (bl-6bda, bl-81dd): the usage listed eleven words, `lernie help
//! <verb>` answered a page for seven, and the other four were refused with
//! *"no verb named `ask`"* — a sentence that is false in the only sense the
//! operator means it, byte for byte identical to what a typo earns, on the one
//! surface whose whole job is to answer with no engine up.
//!
//! **A door is a word, a usage line and prose, with no envelope behind it.** It
//! builds nothing and routes nothing, so it costs no second implementation of a
//! gesture — which is the whole objection the gesture table's own doc raises
//! against admitting these four.
//!
//! **Its arguments are STORED, where a gesture's are computed**, and the
//! difference is the point rather than an inconsistency. A gesture's parameters
//! are envelope fields, so its usage line is derived from the thing that must
//! not drift from it. A door fills no envelope, so its arguments exist only to
//! be printed — the printed line IS the fact, including the one shape a list of
//! field names cannot spell, `help`'s optional `[<verb>]`.

/// One word this binary answers itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Door {
    /// The word typed.
    pub word: &'static str,
    /// The arguments exactly as the usage line prints them; empty for none.
    pub takes: &'static str,
    /// How many arguments it accepts, fewest and most. `help` is the only door
    /// where the two differ, and it is why this is a pair rather than a count.
    pub arity: (usize, usize),
    /// One line: what the word is for.
    pub summary: &'static str,
    /// The page, and the paragraph the usage prints under the word. **One
    /// home**: the usage used to carry this prose by hand beside a table that
    /// could not reach it.
    pub detail: &'static str,
}

impl Door {
    /// The line an operator types.
    pub fn usage(&self) -> String {
        if self.takes.is_empty() {
            format!("lernie {}", self.word)
        } else {
            format!("lernie {} {}", self.word, self.takes)
        }
    }

    /// **The arity refusal**, in the same words the gesture table's is — so a
    /// real word typed with the wrong number of arguments is told what it takes
    /// rather than that it is not an argument this binary recognises.
    pub fn refused(&self, got: usize) -> String {
        let (least, most) = self.arity;
        let takes = if least == most {
            format!("{least}")
        } else {
            format!("at most {most}")
        };
        format!(
            "`lernie {}` takes {takes} argument(s) and got {got} — usage: {}",
            self.word,
            self.usage()
        )
    }
}

/// The door that word names, if it is one.
pub fn find(word: &str) -> Option<Door> {
    TABLE.iter().find(|door| door.word == word).copied()
}

/// Every door, in the order the usage prints them.
pub fn table() -> Vec<Door> {
    TABLE.to_vec()
}

/// The composite that begins a conversation.
pub const START: Door = Door {
    word: "start",
    takes: "<workspace> <goal>",
    arity: (2, 2),
    summary: "begin a conversation on that workspace",
    detail: "The start family's two acts (yog's REMOTE §8.1), staged and \
             fired in one process. Not a gesture but both of them: a \
             `prepare`, then a `prompt` carrying the body that answered it \
             straight back. Both reply streams print; the exit code is the \
             fire's. It is one word because the thing between the two acts is \
             a local — the staged body, held while the second is composed — \
             and a nested object is not a word an operator types, which is why \
             it is a door here rather than a row of the gesture table.",
};

/// The escape hatch, which is the surface.
pub const ASK: Door = Door {
    word: "ask",
    takes: "<envelope>",
    arity: (1, 1),
    summary: "one gesture envelope, written out",
    detail: "Any op — including one this build has never heard of — as the \
             JSON object the boundary carries, with `op` the discriminant and \
             every parameter a named field. This is not a fallback: it is the \
             surface, and the typed verbs are its shorthand. Quote it as one \
             argument. It goes down the channel its `workspace` names, exactly \
             as a typed verb does, and an op the far end does not know refuses \
             in band naming it.",
};

/// What this box holds, said without dialling any of it.
pub const ENTRIES: Door = Door {
    word: "entries",
    takes: "",
    arity: (0, 0),
    summary: "describe every channel this box holds, without dialling any",
    detail: "This box's own engine first, then one row per workspace held \
             elsewhere, each with its address or the reason it has none. It \
             opens no socket, so it answers with every engine down — and a \
             half-provisioned channel says which file is missing beside the \
             ones that are fine. Material reaches these directories by the \
             operator's hand, out of channel, always.",
};

/// The word whose subject is this binary.
pub const HELP: Door = Door {
    word: "help",
    takes: "[<verb>]",
    arity: (0, 1),
    summary: "what a word takes and what it answers with",
    detail: "With no argument, the usage and every word this binary answers. \
             With one, that word's own page — a gesture verb or one of the \
             four doors alike, because a page an operator is offered has to be \
             reachable for every word the list names. Its subject is this \
             binary rather than a world, so it answers with no engine up and \
             no channel provisioned. `--help` and `-h` are the same word.",
};

/// Every door, in the order the usage prints them: the two that act, then the
/// two that describe.
const TABLE: &[Door] = &[START, ASK, ENTRIES, HELP];

#[cfg(test)]
mod tests;
