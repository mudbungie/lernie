//! **The conversation's spine** — every operable commit it has, and the
//! children dispatched off them (yog's `docs/REMOTE.md` §8.5, §9.7; bl-b52c).
//!
//! Two lists rather than a nesting, which is upstream's own shape: *"a card
//! names its notch by index and a notch with no card is still a place a
//! gesture can reach"*. So [`Rail::notches`] is the spine and [`Rail::cards`]
//! hangs off it by [`Card::notch`], an index into the first.
//!
//! # What makes a notch OPERABLE, and why the seat asks it rather than the eye
//!
//! A notch is a step, and its `commit` is the branch tip that step's model call
//! was assembled against. Upstream: *"`commit` is `None` for a step that landed
//! no `meta.json` — such a notch is a point on the spine but not a pinnable
//! one, because there is no tree to pin to."* That is exactly the fork
//! control's admission test on this pane ([`Notch::operable`]): `fork`'s
//! `from` takes a **ref**, and a notch with no commit names none.
//!
//! # Absence is a reading, in both of the shapes it takes
//!
//! `commit`/`short` and `row`/`cut` are absent — never empty — when the step
//! recorded no `meta.json` or the chat gave the call no seat, and upstream is
//! explicit that *"a reader must not have to tell that from a notch pinned at
//! the empty string"*. The pair is read as a pair ([`Seat`]) for the same
//! reason: the two keys are written together and mean one fact.
//!
//! # `short` is not read back, because the commit is its storage
//!
//! The encoder writes both; upstream's own decoder reads only `commit` and
//! derives the label. This reader does the same ([`Notch::short`]) — one fact,
//! one home, and a clipped copy carried beside the thing it clips is the
//! second home the house rule forbids.

use serde_json::{Map, Value};

use super::convs::AgentState;
use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "rail";

/// What a notch shows when its step recorded no read-state commit. The
/// engine's own spelling of it, so the two faces say the absence alike.
pub const NO_COMMIT: &str = "—";

/// git's own short-oid width — the width upstream clips its `short` to, and
/// therefore the width this seat clips to when it derives one.
const SHORT_OID: usize = 7;

/// The spine whole: the notches, and the cards hanging off them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rail {
    /// One notch per step, in the engine's own order.
    pub notches: Vec<Notch>,
    /// The children dispatched from this conversation, each naming its notch.
    pub cards: Vec<Card>,
}

/// One notch: a step, the commit its call read against, the spend as of it,
/// and its seat in the chat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notch {
    /// The step's zero-padded sequence name — the same address `step` takes.
    pub seq: String,
    /// The read-state commit, or `None` for a step that landed no record.
    pub commit: Option<String>,
    /// **The spend as of this notch** — a rollup of everything up to and
    /// including it, never this step's own figure. It rides so a seat does not
    /// fold the prefix itself, which would be deriving over an answer.
    pub budget: u64,
    /// Where the notch sits in the chat, where the chat has a seat for it.
    pub seat: Option<Seat>,
}

/// A notch's seat in the chat: the entry its rule paints above, and how much
/// of the transcript that call had read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    /// The row key of the first entry this call read that its predecessor had
    /// not.
    pub row: String,
    /// Entry count of the read state — everything ahead of this call's own
    /// model output.
    pub cut: u64,
}

/// One child dispatched from this conversation, at the notch it was born at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Card {
    /// The child's conversation id.
    pub agent: String,
    /// Its display name.
    pub name: String,
    /// **Where it forked from, in the engine's own words** — `from here`,
    /// `from config/<name>`, `from <Name>@<oid>`. It is prose the engine
    /// composes and this seat repeats: the two edges VISION draws are what
    /// this sentence says, and a seat that re-derived them would be a second
    /// author of one fact.
    pub fork: String,
    /// The child's badge state, on the conversation list's own vocabulary.
    pub state: AgentState,
    /// What the child itself has spent, never its descent's.
    pub tokens: u64,
    /// The last of its inference text, absent while it has produced none.
    pub tail: Option<String>,
    /// Which notch it hangs from — an index into [`Rail::notches`].
    pub notch: u64,
}

impl Notch {
    /// **Whether a gesture can reach this notch**: it has a commit, so `fork`
    /// has a ref to take. The fork control's admission test, asked once.
    pub fn operable(&self) -> bool {
        self.commit.is_some()
    }

    /// The notch's label: its commit clipped, or [`NO_COMMIT`]. Derived here
    /// because the commit IS this string's storage.
    pub fn short(&self) -> String {
        self.commit.as_ref().map_or_else(
            || NO_COMMIT.to_owned(),
            |oid| oid.get(..SHORT_OID).unwrap_or(oid).to_owned(),
        )
    }
}

/// The whole answer, strictly ([`super`]'s rung 1).
pub(crate) fn rail(obj: &Map<String, Value>) -> Result<Rail, String> {
    Ok(Rail {
        notches: fields::rows(obj, notch)?,
        cards: fields::list(obj, "cards", card)?,
    })
}

/// One notch. The seat is read as a pair off `row`, which is the key the
/// encoder writes `cut` beside — so the two cannot decode to a state the
/// encoder could not have written.
fn notch(value: &Value) -> Result<Notch, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("notch: not an object")?;
    let seat = match fields::opt_text(obj, "row")? {
        None => None,
        Some(row) => Some(Seat {
            row,
            cut: fields::count(obj, "cut")?,
        }),
    };
    Ok(Notch {
        seq: fields::text(obj, "seq")?,
        commit: fields::opt_text(obj, "commit")?,
        budget: fields::count(obj, "budget")?,
        seat,
    })
}

/// One card, strictly. The state is the conversation list's own reading
/// (rung 3 with it), because two tables for one vocabulary drift.
fn card(value: &Value) -> Result<Card, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("card: not an object")?;
    Ok(Card {
        agent: fields::text(obj, "agent")?,
        name: fields::text(obj, "name")?,
        fork: fields::text(obj, "fork")?,
        state: AgentState::of(&fields::text(obj, "state")?),
        tokens: fields::count(obj, "tokens")?,
        tail: fields::opt_text(obj, "tail")?,
        notch: fields::count(obj, "notch")?,
    })
}

#[cfg(test)]
mod tests;
