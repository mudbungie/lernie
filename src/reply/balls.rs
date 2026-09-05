//! **The balls themselves** — the whole world's binding table, one wall's own
//! balls with what each has cost, and the branch a wall tracks its tasks on
//! (yog's `docs/REMOTE.md` §9.7; bl-d2af).
//!
//! Three readings and one subject, so they share a file: what the task store
//! holds, seen from the box, from one wall, and from the branch a wall's `bl`
//! verbs are pointed at. The board's own fold is [`super::board`]'s — the
//! richer answer to the same question, and the two are separate ops upstream.
//!
//! # The binding table is the join, and every field but three is optional
//!
//! `balls` answers *which ball is claimed by which workspace, in which state*
//! over the whole box. A ball nobody holds has no claimant and no workspace,
//! and a ball whose title the store could not read has no title — three
//! absences that are readings rather than tolerances, and each is a different
//! claim from an empty string.
//!
//! # One wall's balls carry a figure and the world's binding table does not
//!
//! That is upstream's division rather than this seat's: `/balls` is the join
//! and `workspace-balls` is *"the balls the focused workspace holds, with what
//! each has cost"*. So the figure ([`super::spend`]) is required on a bound
//! ball and absent from a binding row, and a seat that carried one on both
//! would be inventing a number for the rows that have none.
//!
//! # The marks answer is one field, and it is the branch re-read
//!
//! `marks` reads, or amends, the branch a wall tracks its tasks on. The reply
//! is *the branch re-read afterwards, never an echo of what was asked* — which
//! is why the read and the write answer with the same kind and this seat needs
//! only the one reading.

use serde_json::{Map, Value};

use super::spend::Figure;
use super::{fields, spend};

/// The kind token the world's binding table answers to.
pub(crate) const KIND: &str = "balls";
/// The kind token one wall's own balls answer to.
pub(crate) const HELD: &str = "workspace-balls";
/// The kind token the tracking branch answers to.
pub(crate) const MARKS: &str = "marks";

/// **One ball⇄workspace binding fact**, over the whole box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BallRow {
    pub ball_id: String,
    pub project: String,
    /// The §3.5 binding, carried verbatim ([`super`]'s rung 3).
    pub state: String,
    /// What it is called, where the store could read a title.
    pub title: Option<String>,
    /// Who holds it, where anybody does.
    pub claimant: Option<String>,
    /// The wall its verbs run in, where one holds it.
    pub workspace: Option<String>,
}

/// **One ball a wall holds**, with what its conversations have spent on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundBall {
    pub id: String,
    /// The §3.5 badge, absent — not empty — for a state that needs none.
    pub badge: Option<String>,
    /// The project its `bl` verbs run in.
    pub project: String,
    /// The name they stamp `--as`.
    pub owner: String,
    /// The binding, carried verbatim.
    pub state: String,
    /// What it has cost. Required here — see the module doc.
    pub spend: Figure,
}

/// One binding row, strictly.
pub(crate) fn row(value: &Value) -> Result<BallRow, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("balls row: not an object")?;
    Ok(BallRow {
        ball_id: fields::text(obj, "ball_id")?,
        project: fields::text(obj, "project")?,
        state: fields::text(obj, "state")?,
        title: fields::opt_text(obj, "title")?,
        claimant: fields::opt_text(obj, "claimant")?,
        workspace: fields::opt_text(obj, "workspace")?,
    })
}

/// One bound ball, strictly.
pub(crate) fn bound(value: &Value) -> Result<BoundBall, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("ball row: not an object")?;
    Ok(BoundBall {
        id: fields::text(obj, "id")?,
        badge: fields::opt_text(obj, "badge")?,
        project: fields::text(obj, "project")?,
        owner: fields::text(obj, "owner")?,
        state: fields::text(obj, "state")?,
        spend: spend::figure(obj.get("spend").ok_or("ball row: missing spend")?)?,
    })
}

/// The tracking branch, which is the whole of that answer.
pub(crate) fn marks(obj: &Map<String, Value>) -> Result<String, String> {
    fields::text(obj, "branch")
}

#[cfg(test)]
mod tests;
