//! **The conversation list** one workspace answers with (yog's
//! `docs/REMOTE.md` §8, §9.7) — the middle pane's rows.
//!
//! **Two names, and they are not interchangeable.** [`ConvRow::display`] is
//! what the row paints; [`ConvRow::name`] is what a later act may **address**,
//! and it is absent exactly when the engine has a name it will not answer to.
//! [`ConvRow::root_id`] is the address that always works, so a seat that
//! stayed with the id can never post a gesture the engine will refuse.
//!
//! **[`ConvRow::uncertain`] is not derivable from the state beside it.** The
//! engine's classifier answers a state *and* whether it could observe one, and
//! the two are separate facts: a conversation it cannot probe reads as settled
//! and is not known to be. A seat that dropped the flag would paint a definite
//! reading of something nothing looked at — which is also the shape this
//! window's own pending row wears (`crate::ui::model::claim`).
//!
//! **What this build does not carry.** The row also spells the stop cascade's
//! two gates, the strict child count, a bound ball, an alignment verdict and
//! the in-flight class. Each belongs to a control or a badge that does not
//! exist here yet; rung 4 of [`super`]'s policy rides them through unread, and
//! each arrives with the surface that paints it.

use serde_json::{Map, Value};

use super::fields;

/// This reply's kind token.
pub(crate) const KIND: &str = "conversations";

/// One conversation as the list paints it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConvRow {
    /// The conversation root's id — the address that always resolves.
    pub root_id: String,
    /// What the row is labelled with.
    pub display: String,
    /// **The addressable name**, when the engine has one it will answer to.
    /// Absent where it holds a name that is display-only, which is a name no
    /// stored fact backs: handing it over would be handing over a target the
    /// engine is going to refuse.
    pub name: Option<String>,
    /// The badge state, aggregated over the whole subtree.
    pub state: AgentState,
    /// **Whether the engine could observe the state above.** A conversation it
    /// cannot probe — no lock to read, no step to frame — answers a state
    /// anyway and flags it here, so a reading nothing witnessed never paints as
    /// a definite one.
    pub uncertain: bool,
    /// The row's first-line preview.
    pub preview: String,
    /// How long since the conversation last moved. Signed, because clock skew
    /// between two machines is a fact and not an error.
    pub age_secs: i64,
    /// Attention-bearing members under it.
    pub attention: u64,
    /// How many conversations the subtree holds.
    pub members: u64,
    /// How far the row hangs under its conversation root — the list's indent.
    pub depth: u64,
    /// How solidly the row paints. Not derivable from
    /// [`state`](Self::state): a row can be settled and provisional at once.
    pub tone: Tone,
}

/// The badge state of a conversation. **Rung 3**: an unknown word keeps its
/// spelling rather than becoming one of the four, because painting an
/// unrecognised state as `Quiescent` would tell an operator nothing is
/// happening on the strength of a word this build has never seen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentState {
    /// A driver is held on it.
    Live,
    /// It is streaming.
    InFlight,
    /// Settled.
    Quiescent,
    /// Stopped.
    Stopped,
    /// A word this build does not know, verbatim.
    Unknown(String),
}

/// How solidly a row paints, on the same terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tone {
    Plain,
    Weak,
    Good,
    Bad,
    Live,
    InFlight,
    /// A word this build does not know, verbatim.
    Unknown(String),
}

impl AgentState {
    /// The word the badge paints.
    pub fn label(&self) -> String {
        match self {
            Self::Live => "live".to_owned(),
            Self::InFlight => IN_FLIGHT.to_owned(),
            Self::Quiescent => "quiescent".to_owned(),
            Self::Stopped => "stopped".to_owned(),
            Self::Unknown(word) => word.clone(),
        }
    }

    /// One state token, total by construction.
    fn of(word: &str) -> Self {
        match word {
            "live" => Self::Live,
            IN_FLIGHT => Self::InFlight,
            "quiescent" => Self::Quiescent,
            "stopped" => Self::Stopped,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

impl Tone {
    /// The word this tone is written as — the one an unstyled row falls back
    /// to painting.
    pub fn label(&self) -> String {
        match self {
            Self::Plain => "plain".to_owned(),
            Self::Weak => "weak".to_owned(),
            Self::Good => "good".to_owned(),
            Self::Bad => "bad".to_owned(),
            Self::Live => "live".to_owned(),
            Self::InFlight => IN_FLIGHT.to_owned(),
            Self::Unknown(word) => word.clone(),
        }
    }

    /// One tone token, total by construction.
    fn of(word: &str) -> Self {
        match word {
            "plain" => Self::Plain,
            "weak" => Self::Weak,
            "good" => Self::Good,
            "bad" => Self::Bad,
            "live" => Self::Live,
            IN_FLIGHT => Self::InFlight,
            other => Self::Unknown(other.to_owned()),
        }
    }
}

/// The one token both enums spell, written once so they cannot disagree.
const IN_FLIGHT: &str = "in-flight";

/// Read one row.
pub(crate) fn row(v: &Value) -> Result<ConvRow, String> {
    let o: &Map<String, Value> = v.as_object().ok_or("conversation row: not an object")?;
    Ok(ConvRow {
        root_id: fields::text(o, "root_id")?,
        display: fields::text(o, "display")?,
        name: fields::opt_text(o, "name")?,
        state: AgentState::of(&fields::text(o, "state")?),
        uncertain: fields::flag(o, "uncertain")?,
        preview: fields::text(o, "preview")?,
        age_secs: fields::secs(o, "age_secs")?,
        attention: fields::count(o, "attention")?,
        members: fields::count(o, "members")?,
        depth: fields::count(o, "depth")?,
        tone: Tone::of(&fields::text(o, "tone")?),
    })
}

#[cfg(test)]
mod tests;
