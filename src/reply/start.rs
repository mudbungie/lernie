//! **The start family's two receipts** (yog's `docs/REMOTE.md` §8.1, §9.8;
//! yog's `docs/DESIGN.md` §3.4, §8.1) — the staged body, and the name the
//! engine minted for what it started.
//!
//! Starting a conversation is **two acts, not one**. `prepare` stages the
//! start — the seed, the workspace if it has to be founded, the ball rung's
//! `bl` steps — and answers a [`Prepared`] body: the fire-time parameters, as
//! the engine settled them. `prompt` hands that body straight back with a goal
//! and fires the detached driver, answering the minted conversation name. So a
//! seat has to hold a reply between two gestures, which is the one shape
//! nothing else on this surface has.
//!
//! # The body crosses back verbatim, and that is the whole design
//!
//! [`Prepared::body`] is the object exactly as it arrived, and
//! [`crate::verbs::prompt`] hands *that* back rather than a re-encoding of the
//! two fields below. It is [`super`]'s rung 4 read in the **write** direction:
//! an unknown field rides through untouched, so a start staged with a work
//! target, a birth lineage or a banner origin this build does not paint still
//! fires with all three. A seat that re-encoded its own reading would silently
//! drop every parameter it had not learned yet — and the dropped parameter is
//! not a missing badge, it is a conversation born in the wrong directory off
//! the wrong config.
//!
//! **So only two fields are read, and the rest is carried.** The workspace,
//! because a fire is addressed and §8.2's mapping has to be spent on it; and
//! the goal, because a rung with a prefill composed one and the operator edits
//! it. `binding`, `lineage` and `origin` are carried and unpainted — each
//! belongs to a control the seat's start pane does not have (a work target
//! picker, a birth policy, the ops banner), and each arrives with the surface
//! that paints it.

use serde_json::{Map, Value};

use super::fields;

/// The staging act's reply kind.
pub(crate) const PREPARED: &str = "prepared";
/// The fire's reply kind.
pub(crate) const STARTED: &str = "started";
/// The spread's reply kind: one staged body per candidate (§4.36).
pub(crate) const FANNED: &str = "fanned";

/// The field the minted conversation name rides under.
const CONVERSATION: &str = "conversation";
/// The two fields of the body this build reads. Everything else is carried.
const WORKSPACE: &str = "workspace";
const GOAL: &str = "goal";

/// **A staged start**: what the fire will carry, as the engine settled it.
///
/// It is deliberately *not* `Eq` — [`body`](Self::body) is arbitrary JSON, and
/// the reply vocabulary gives up an equality nothing in this crate spends to
/// keep the carry lossless.
#[derive(Debug, Clone, PartialEq)]
pub struct Prepared {
    /// The workspace the start was staged in, **in the host's spelling** — it
    /// came off an engine, and §8.2's mapping runs client→host at
    /// [`crate::seat::route`] and nowhere else. So the fire is composed with
    /// this box's own name for that workspace, not with this one.
    pub workspace: String,
    /// The goal the rung composed, which the composer opens on. Empty for the
    /// bare rung, which prefills nothing.
    pub goal: String,
    /// **The body exactly as it crossed**, and what the fire hands back.
    pub body: Value,
}

/// Read the staging receipt: the envelope, then the body nested inside it.
pub(crate) fn prepared(obj: &Map<String, Value>) -> Result<Prepared, String> {
    let body = obj
        .get(PREPARED)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("missing or non-object field {PREPARED:?}"))?;
    body_of(body)
}

/// **One candidate of a spread**, which is a staged body with no envelope
/// around it: `fanned` answers a LIST of these, where `prepared` answers one
/// nested under its own name. Both go through [`body_of`], so a candidate and
/// a single start are the same value read the same way — and each candidate is
/// fired by the ordinary `prompt`, which is the whole of what makes the fan a
/// spread rather than a second start path.
pub(crate) fn candidate(value: &Value) -> Result<Prepared, String> {
    let body = value
        .as_object()
        .ok_or_else(|| "candidate: not an object".to_owned())?;
    body_of(body)
}

/// The two fields this build reads, and the whole body carried beside them.
fn body_of(body: &Map<String, Value>) -> Result<Prepared, String> {
    Ok(Prepared {
        workspace: fields::text(body, WORKSPACE)?,
        goal: fields::text(body, GOAL)?,
        body: Value::Object(body.clone()),
    })
}

/// Read the fire's receipt — the minted conversation name, and nothing else.
///
/// **The name is a barrier**: it is what the reply just made addressable, so
/// every gesture after it may name that conversation (REMOTE §8's live-enumeration
/// barrier, read one noun down).
pub(crate) fn started(obj: &Map<String, Value>) -> Result<String, String> {
    fields::text(obj, CONVERSATION)
}

#[cfg(test)]
mod tests;
