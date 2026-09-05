//! **The conversation's own row, whole** (yog's `docs/REMOTE.md` §9.4, §9.10;
//! bl-3257) — the deepest read this seat takes of one conversation, and the
//! header the records pane opens with.
//!
//! Upstream calls it *"the fat `Agent`… a projection, not a diet"*: what a
//! list row says about a conversation is a glance, and this is the whole of
//! what the engine holds — its descent, its liveness and why, what it has
//! spent, how full its context is, what is in flight on it right now, and the
//! invocation parked at its capability boundary.
//!
//! # Absent-not-null, and the reading is the absence
//!
//! Upstream's own rule for this shape, verbatim: *"an unmarked conversation
//! carries no `marks` key and one at rest no `flight`, because a reader must
//! never have to tell an empty list from a fact the encoder declined to
//! state."* So every optional field here is a **reading** — a conversation
//! with nothing in flight, a seat mark at rest, a call that did not fail — and
//! none of them is a tolerance.
//!
//! # Three facts ride as the engine's own rendering, and none is re-derived
//!
//! [`Strip::facts`] is prose one derivation assembles with per-segment
//! omission rules of its own, and upstream is explicit that *"a wire spelling
//! of the parts would be a second place that decides what a strip says"*.
//! [`Fullness::percent`] is the same call one number down: it is the engine's
//! rounding of the two figures beside it and it is deliberately **not
//! clamped**, so a context that has outgrown its window reads as `140%` — a
//! seat that divided the two itself would be re-taking a decision upstream
//! already took. And [`Attribution::label`] is the sentence for a spend that
//! is not this conversation's alone, absent exactly when there is nothing to
//! say. Each is carried, none is computed.
//!
//! # The two shapes shared with other readings, and why sharing is right
//!
//! The priced figure is [`crate::reply::spend::Figure`] — the ball pane reads
//! the identical object off the identical encoder, so a second reading of it
//! here would be a second protocol. The parked invocation is
//! [`crate::reply::queue::Held`], the type the decision queue already paints;
//! the two encoders spell its id differently (`tool_use` on a queue row,
//! litany's own blob key `tool_use_id` here), so there are two readers and one
//! type, which is what keeps the seat saying one sentence about a park.
//!
//! **What that sharing costs is three fields this seat does not read**, and
//! the cost is the point: `micro_usd` is the integer the engine rendered `usd`
//! from, `unpriced_tokens` says the money is a floor, and the attribution's
//! `count` is the number its clause already spells. Nothing paints any of
//! them, so [`super`]'s rung 4 rides them through unread — a field held for no
//! glass is a field this vocabulary does not carry (DESIGN §4.9).

use serde_json::{Map, Value};

use super::convs::AgentState;
use super::fields;
use super::queue::Held;
use super::spend::Figure;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "agent";

/// The conversation as its own row: who it is, what it is doing, what may be
/// done to it, and what it has cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    /// The conversation asked about, echoed back.
    pub agent: String,
    /// The conversation root it belongs to — itself, for a root.
    pub root: String,
    /// The descent chain above it, outermost first. Empty for a root.
    pub ancestors: Vec<String>,
    /// What it is called.
    pub display: String,
    /// **Whether that name is display-only** — prose no stored fact backs, so
    /// peers cannot address the conversation by it.
    pub display_only: bool,
    /// Its branch tip, empty for a conversation the snapshot does not carry.
    pub tip: String,
    /// Its **own** liveness, not the subtree aggregate a list row carries.
    pub state: AgentState,
    /// **Whether the latest turn was refused at the provider rung** — the fact
    /// that tells an operator's own stop from a provider saying no, both of
    /// which come to rest `stopped`.
    pub refused: bool,
    /// Why that call failed, in one clause. Absent where it did not.
    pub failure: Option<String>,
    /// The marks it wears, in the engine's badge order and carried verbatim.
    pub marks: Vec<String>,
    /// What is in flight anywhere in its conversation, verbatim. Absent at rest.
    pub flight: Option<String>,
    /// The invocation parked at its capability boundary.
    pub held: Option<Held>,
    /// Whether the published snapshot carries it at all.
    pub present: bool,
    /// **What the engine offers on it**, as a list rather than as a bool
    /// apiece. Upstream calls them *"the four §8.2 gates"* and they are one
    /// answer to one question — *what may be done to this conversation* — so
    /// the reading is the set they describe. Three independent flags would be
    /// three representations of one fact, and the paint would rebuild this
    /// list from them anyway.
    pub offers: Vec<Offer>,
    /// The live mark's seats — the conversation, then its subagents in descent
    /// order, each with what it is doing. Empty is the mark at rest.
    pub seats: Vec<Seat>,
    /// The in-flight strip, absent at rest.
    pub strip: Option<Strip>,
    /// **What the branch has spent**, always present: a conversation that has
    /// spent nothing has spent zero, and that is a reading.
    pub spend: Figure,
    /// How full its context is. Absent when nothing measured can be said,
    /// which is not the same claim as a context at 0%.
    pub context: Option<Fullness>,
}

/// **How full the context is**, and the engine's own percent.
///
/// The percent rides rather than being divided out again: it is upstream's
/// rounding of the two figures beside it and it is deliberately **not
/// clamped**, so a context that has outgrown its declared window reads as
/// `140%` — which says *the row's window is wrong or the provider compacted*.
/// A seat that computed one would be re-taking a decision upstream took on
/// purpose, and would disagree with it exactly where the disagreement matters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fullness {
    pub model: String,
    pub prompt_tokens: u64,
    pub window: u64,
    pub percent: u64,
}

/// **One thing the engine says may be done** to the conversation. A closed
/// set, because the wire spells it as a fixed set of gates rather than as a
/// vocabulary that can grow — a new gate is a new key and therefore a new
/// reading, not an unknown token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Offer {
    /// An advance may be started on it.
    Nudge,
    /// Its driver may be killed.
    Stop,
    /// …and the cascade onto its children is offered beside that kill.
    Children,
}

/// One seat of the live mark: an agent, and what it is doing right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seat {
    pub name: String,
    /// Verbatim ([`super`]'s rung 3) — this vocabulary is one agent's own
    /// activity and is not the conversation-wide class beside it.
    pub doing: String,
}

/// The in-flight strip: the class, and the live characteristics as one
/// rendered run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Strip {
    pub class: String,
    pub facts: String,
}

/// The whole row, strictly ([`super`]'s rung 1).
pub(crate) fn agent(obj: &Map<String, Value>) -> Result<Agent, String> {
    Ok(Agent {
        agent: fields::text(obj, "agent")?,
        root: fields::text(obj, "root")?,
        ancestors: optional_list(obj, "ancestors")?,
        display: fields::text(obj, "display")?,
        display_only: fields::flag(obj, "display_only")?,
        tip: fields::text(obj, "tip")?,
        state: AgentState::of(&fields::text(obj, "state")?),
        refused: fields::flag(obj, "refused")?,
        failure: fields::opt_text(obj, "failure")?,
        marks: optional_list(obj, "marks")?,
        flight: fields::opt_text(obj, "flight")?,
        held: fields::nested(obj, "held", held)?,
        present: fields::flag(obj, "present")?,
        offers: offers(obj)?,
        seats: match obj.get("seats") {
            None => Vec::new(),
            Some(_) => fields::list(obj, "seats", seat)?,
        },
        strip: fields::nested(obj, "strip", strip)?,
        spend: super::spend::figure(obj.get("spend").ok_or("agent: missing \"spend\"")?)?,
        context: fields::nested(obj, "context", fullness)?,
    })
}

/// **The gates, read as the set they describe.** Every one of the three is a
/// required key, so an engine that stopped writing one refuses here rather
/// than answering a shorter list.
fn offers(obj: &Map<String, Value>) -> Result<Vec<Offer>, String> {
    let gates = [
        ("nudgeable", Offer::Nudge),
        ("stoppable", Offer::Stop),
        ("stop_children", Offer::Children),
    ];
    let mut offered = Vec::new();
    for (key, offer) in gates {
        if fields::flag(obj, key)? {
            offered.push(offer);
        }
    }
    Ok(offered)
}

/// An array whose absence is the empty list, and whose presence is read
/// strictly — the absent-not-null rule, in the one place two fields share it.
fn optional_list(obj: &Map<String, Value>, key: &str) -> Result<Vec<String>, String> {
    match obj.get(key) {
        None => Ok(Vec::new()),
        Some(_) => fields::list(obj, key, word),
    }
}

/// One word of a list, verbatim.
fn word(value: &Value) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "a non-string word in a list".to_owned())
}

/// The parked invocation under litany's own key spelling.
fn held(obj: &Map<String, Value>) -> Result<Held, String> {
    Ok(Held {
        tool: fields::text(obj, "tool")?,
        tool_use: fields::text(obj, "tool_use_id")?,
        reason: fields::text(obj, "reason")?,
    })
}

/// One live-mark seat.
fn seat(value: &Value) -> Result<Seat, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("seat: not an object")?;
    Ok(Seat {
        name: fields::text(obj, "name")?,
        doing: fields::text(obj, "doing")?,
    })
}

/// The context reading, with the engine's own percent carried rather than
/// divided out again.
fn fullness(obj: &Map<String, Value>) -> Result<Fullness, String> {
    Ok(Fullness {
        model: fields::text(obj, "model")?,
        prompt_tokens: fields::count(obj, "prompt_tokens")?,
        window: fields::count(obj, "window")?,
        percent: fields::count(obj, "percent")?,
    })
}

/// The in-flight strip.
fn strip(obj: &Map<String, Value>) -> Result<Strip, String> {
    Ok(Strip {
        class: fields::text(obj, "class")?,
        facts: fields::text(obj, "facts")?,
    })
}

#[cfg(test)]
mod tests;
