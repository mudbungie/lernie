//! **The roster** — the window's altitude-0 chrome (yog's `docs/REMOTE.md`
//! §8, §9.7): the workspaces an engine enumerates, each with its rollups, and
//! how current the derivation behind the answer is.
//!
//! **A row names a workspace; it does not locate it** (REMOTE §8: *"paths
//! never cross the wire"*). The name is the workspace's directory leaf on its
//! host and it is the token every gesture addresses it by — so a row is both
//! the label the roster paints and the address the next act carries, with no
//! table in between.
//!
//! **What this build does not carry.** The row also spells the config-lineage
//! tip a model picker advances. There is no picker here, and rung 4 of
//! [`super`]'s policy means the field rides through unread rather than
//! refusing — a kind nothing renders is a kind nobody has to carry, and the
//! field arrives with the pane that paints it.

use serde_json::{Map, Value};

use super::fields;

/// This reply's kind token.
pub(crate) const KIND: &str = "workspaces";

/// The whole answer: the enumeration, and the two notes about its currency.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Workspaces {
    /// Every enumerated workspace with its rollups.
    pub rows: Vec<WsRow>,
    /// **How stale the derivation this answer came from is**, or `None` while
    /// it is current — which is the ordinary case and paints nothing.
    ///
    /// It crosses as the **rendered line** rather than as an age and a
    /// threshold, and that is the engine's ruling rather than this seat's: the
    /// wording is one derivation's, with a bound the operator tunes in the
    /// engine's own config, and a seat that rebuilt the sentence from parts
    /// would be a second place deciding when a derivation is late.
    pub stale: Option<String>,
    /// **What grew since the previous derivation**, or `None` when nothing
    /// did. Also a rendered line, for the same reason.
    pub growth: Option<String>,
}

/// One workspace as the roster paints it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsRow {
    /// Its name — the leaf on its host, and the token a gesture addresses it
    /// by. **Not rewritten here**: the client-side rename is spent at the
    /// channel boundary and nowhere else ([`crate::seat::route`]), so a row
    /// arrives in the host's spelling and the roster is what maps it back.
    pub workspace: String,
    /// How the engine classifies it.
    pub kind: WorkspaceKind,
    /// Attention-bearing conversations in it.
    pub attention: u64,
    /// Root-and-member conversation count.
    pub agents: u64,
    /// Whether anything in it is running right now.
    pub running: bool,
    /// **Where the operator pinned it** — its rank in the durable pin list,
    /// `None` for a workspace that is not pinned.
    ///
    /// A rank rather than a flag, because the roster hoists pinned rows *in
    /// pin order*: a seat given only a boolean would have to read the pin list
    /// back to sort them, which is the seat joining an answer against a
    /// document only the engine holds.
    pub pinned: Option<u64>,
}

/// How the engine classifies a workspace.
///
/// **Rung 3 lives on this enum.** A classification this build does not know
/// keeps its word and paints it; it does not become one of the three below,
/// because a replay painted as a wall is a lie about what may be written to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceKind {
    /// The server's own — a wall an operator named.
    Named,
    /// The engine's, enumerated rather than named here.
    Foreign,
    /// A read-only replay.
    Replay,
    /// A word this build does not know, verbatim.
    Unknown(String),
}

impl WorkspaceKind {
    /// The word the roster paints — for [`Unknown`](Self::Unknown), the
    /// engine's own, which is the whole point of keeping it.
    pub fn label(&self) -> String {
        match self {
            Self::Named => NAMED.to_owned(),
            Self::Foreign => FOREIGN.to_owned(),
            Self::Replay => REPLAY.to_owned(),
            Self::Unknown(word) => word.clone(),
        }
    }
}

const NAMED: &str = "named";
const FOREIGN: &str = "foreign";
const REPLAY: &str = "replay";

/// Read the whole answer. The two notes are **absent** rather than null in the
/// ordinary case, which is what makes "current" and "the engine declined to
/// say" one reading rather than two.
pub(crate) fn workspaces(obj: &Map<String, Value>) -> Result<Workspaces, String> {
    Ok(Workspaces {
        rows: fields::rows(obj, row)?,
        stale: fields::opt_text(obj, "stale")?,
        growth: fields::opt_text(obj, "growth")?,
    })
}

/// Read one row. `pinned` is absent — never null and never a rank of its own —
/// for an unpinned workspace, so a reader never has to tell "rank 0" from "not
/// pinned", and rank 0 is the first hoisted row.
fn row(v: &Value) -> Result<WsRow, String> {
    let o = v.as_object().ok_or("workspace row: not an object")?;
    Ok(WsRow {
        workspace: fields::text(o, "workspace")?,
        kind: kind(&fields::text(o, "kind")?),
        attention: fields::count(o, "attention")?,
        agents: fields::count(o, "agents")?,
        running: fields::flag(o, "running")?,
        pinned: match o.get("pinned") {
            None | Some(Value::Null) => None,
            Some(_) => Some(fields::count(o, "pinned")?),
        },
    })
}

/// One classification token, total by construction (rung 3).
fn kind(word: &str) -> WorkspaceKind {
    match word {
        NAMED => WorkspaceKind::Named,
        FOREIGN => WorkspaceKind::Foreign,
        REPLAY => WorkspaceKind::Replay,
        other => WorkspaceKind::Unknown(other.to_owned()),
    }
}

#[cfg(test)]
mod tests;
