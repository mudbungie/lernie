//! **The fleet board** — every live ball in its column, and the loops that are
//! running them (yog's `docs/REMOTE.md` §9.7, VISION §5 V4; bl-d2af).
//!
//! It is the engine's own fold over its task store: the three familiar rungs
//! `bl list` derives, plus `gated` — a ball you could claim but could not
//! deliver, shown with the ball whose close mints its gate. Each claimed row
//! names the conversations working it and carries what it has cost.
//!
//! # The column and the state are two facts, so both ride
//!
//! `column` is where the board puts the row and `state` is the §3.5 binding;
//! upstream writes them side by side because they say different things, and
//! both are carried verbatim ([`super`]'s rung 3). A column this build has
//! never heard of paints as its own word rather than being sorted into a
//! neighbour, which is the one thing rung 3 forbids outright.
//!
//! # The fleet rides on the board, and it is ABSENT rather than empty
//!
//! There is no `fleet` read on this wire and no `fleet` reply kind. A loop's
//! own facts — how full it is, how often it looks, when it last acted, and the
//! ceiling that would stop it — arrive on this answer, in an array upstream
//! leaves OUT when nothing is armed. So the absence is the reading, and
//! [`Board::fleet`] is empty for a box with no loop rather than optional: the
//! two claims are the same claim here, which is the one case where a `Vec` and
//! an `Option<Vec>` do not differ.
//!
//! **The loop's rendered line rides too**, and this seat paints that rather
//! than composing one. `label` is upstream's own sentence about the cap, the
//! count, the tick and the lease; a seat that re-derived it from the numbers
//! beside it would be a second opinion about a loop it does not run.

use serde_json::{Map, Value};

use super::spend::Figure;
use super::{fields, spend};

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "board";

/// **One engine's board**: the rows, and whatever loops are running them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    pub rows: Vec<BoardRow>,
    /// The armed loops, empty where none is — see the module doc.
    pub fleet: Vec<Fleet>,
}

/// One live ball, in the column the board put it in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardRow {
    pub id: String,
    /// Where the board put it — `ready`, `gated`, `claimed`, `blocked`,
    /// carried verbatim.
    pub column: String,
    /// The §3.5 binding, carried verbatim beside the column.
    pub state: String,
    pub title: String,
    /// Lower is more urgent, and it may be negative.
    pub priority: i64,
    pub project: String,
    /// The wall its verbs run in, where one holds it.
    pub workspace: Option<String>,
    /// Who holds it, where anybody does.
    pub claimant: Option<String>,
    /// The epic it hangs under, where it hangs under one.
    pub parent: Option<String>,
    /// The balls whose close would ungate this one.
    pub gates: Vec<Gate>,
    /// The conversations working it.
    pub drones: Vec<Drone>,
    /// What it has cost, where the engine could price it.
    pub spend: Option<Figure>,
    /// What its live subtree has cost, on an epic.
    pub rollup: Option<Figure>,
}

/// A ball whose close mints this row's gate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub id: String,
    pub title: String,
    /// Which act of the gating ball opens the gate, carried verbatim.
    pub mints: String,
}

/// A conversation working a claimed ball.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drone {
    pub root_id: String,
    pub name: String,
}

/// **One armed loop's facts**, as the engine derived them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fleet {
    pub workspace: String,
    pub project: String,
    /// How many of that project's balls it will run at once.
    pub cap: u64,
    /// How many it is holding now.
    pub count: u64,
    /// Whether it has room to claim another.
    pub room: bool,
    /// The spend gate that would stop it, where one is standing.
    pub ceiling: Option<String>,
    /// The engine's own line about the whole of the above.
    pub label: String,
}

/// The whole answer, strictly ([`super`]'s rung 1: every refusal names its
/// field).
pub(crate) fn board(obj: &Map<String, Value>) -> Result<Board, String> {
    Ok(Board {
        rows: fields::rows(obj, row)?,
        fleet: match obj.get("fleet") {
            None => Vec::new(),
            Some(_) => fields::list(obj, "fleet", fleet)?,
        },
    })
}

/// One row. Every optional field is a reading — upstream's own spelling of a
/// fact nobody recorded — never a tolerance.
fn row(value: &Value) -> Result<BoardRow, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("board row: not an object")?;
    Ok(BoardRow {
        id: fields::text(obj, "id")?,
        column: fields::text(obj, "column")?,
        state: fields::text(obj, "state")?,
        title: fields::text(obj, "title")?,
        priority: fields::secs(obj, "priority")?,
        project: fields::text(obj, "project")?,
        workspace: fields::opt_text(obj, "workspace")?,
        claimant: fields::opt_text(obj, "claimant")?,
        parent: fields::opt_text(obj, "parent")?,
        gates: fields::list(obj, "gates", gate)?,
        drones: fields::list(obj, "drones", drone)?,
        spend: obj.get("spend").map(spend::figure).transpose()?,
        rollup: obj.get("rollup").map(spend::figure).transpose()?,
    })
}

/// One gate.
fn gate(value: &Value) -> Result<Gate, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("board gate: not an object")?;
    Ok(Gate {
        id: fields::text(obj, "id")?,
        title: fields::text(obj, "title")?,
        mints: fields::text(obj, "mints")?,
    })
}

/// One drone.
fn drone(value: &Value) -> Result<Drone, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("board drone: not an object")?;
    Ok(Drone {
        root_id: fields::text(obj, "root_id")?,
        name: fields::text(obj, "name")?,
    })
}

/// One armed loop.
fn fleet(value: &Value) -> Result<Fleet, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("fleet facts: not an object")?;
    Ok(Fleet {
        workspace: fields::text(obj, "workspace")?,
        project: fields::text(obj, "project")?,
        cap: fields::count(obj, "cap")?,
        count: fields::count(obj, "count")?,
        room: fields::flag(obj, "room")?,
        ceiling: fields::opt_text(obj, "ceiling")?,
        label: fields::text(obj, "label")?,
    })
}

#[cfg(test)]
mod tests;
