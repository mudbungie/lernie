//! **Text found across the world an engine can see** — balls, workspaces and
//! conversations, one row per hit (yog's `docs/REMOTE.md` §3).
//!
//! A hit says where it is, which field of that thing carried the needle, how
//! far into it, and the words around it. The engine also says what it could
//! **not** read, per subject, which is a different claim from finding nothing
//! there — so the two ride as two fields and this seat keeps them two.
//!
//! # A hit is read and not acted on, and that is upstream's defect
//!
//! yog bl-ef16 records that a search row addresses its workspace and its
//! project by the **engine's own absolute path** while every gesture this box
//! composes carries a name (REMOTE §8's *"paths never cross the wire"*, and
//! its identify/locate rule). So the keys a row spells are the keys the acts
//! take and the values are not, and feeding one back earns `unknown
//! workspace`. This seat therefore paints the address as the text it is and
//! offers no control that spends it — see `crate::ui::find`. Guessing a name
//! off a path here would be the mis-aim `crate::ui::Model::wall` exists to
//! refuse, one layer down.
//!
//! # Every address field is optional, because a hit is one of three things
//!
//! A ball hit carries `project` and `id`, a workspace hit carries `workspace`,
//! and a conversation hit carries `workspace` and `agent`. Rather than branch
//! on `at` — which is [`super`]'s rung 3 and rides verbatim, so a subject a
//! newer engine grew would fall off a closed set — the fields are read as
//! options and whatever is there is painted in order.

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "search";

/// One hit: what it is in, where in it, and the words around the needle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// What kind of thing carried it — `ball`, `workspace`, `conversation`.
    /// Verbatim: a word this build has never seen paints as itself.
    pub at: String,
    /// Which field of that thing the needle was in.
    pub field: String,
    /// The words around it, as the engine cut them.
    pub excerpt: String,
    /// How far into the field the needle starts.
    pub offset: u64,
    /// The project a ball hit is in, as the engine names it.
    pub project: Option<String>,
    /// The ball's id.
    pub id: Option<String>,
    /// The workspace, as the engine names it.
    pub workspace: Option<String>,
    /// The conversation.
    pub agent: Option<String>,
}

/// **One answer to one needle**: what was found, and what could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// The needle the engine searched for, as it read it. It is echoed rather
    /// than assumed, so a pane says what these rows are an answer TO instead
    /// of what the box currently holds.
    pub needle: String,
    /// The hits, in the engine's own order.
    pub rows: Vec<Hit>,
    /// **What the engine could not read**, one sentence each. Absent from the
    /// hits is not the same claim as unreadable, and a seat that folded the
    /// two would report a clean search over a store it never opened.
    pub unreadable: Vec<String>,
}

/// One hit, strictly. The four facts about the match are required; the four
/// address fields are optional because a hit is one of three shapes.
pub(crate) fn row(value: &Value) -> Result<Hit, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("search row: not an object")?;
    Ok(Hit {
        at: fields::text(obj, "at")?,
        field: fields::text(obj, "field")?,
        excerpt: fields::text(obj, "excerpt")?,
        offset: fields::count(obj, "offset")?,
        project: fields::opt_text(obj, "project")?,
        id: fields::opt_text(obj, "id")?,
        workspace: fields::opt_text(obj, "workspace")?,
        agent: fields::opt_text(obj, "agent")?,
    })
}

/// The whole answer.
pub(crate) fn found(obj: &Map<String, Value>) -> Result<Found, String> {
    Ok(Found {
        needle: fields::text(obj, "needle")?,
        rows: fields::rows(obj, row)?,
        unreadable: fields::list(obj, "unreadable", |value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "unreadable: not a string".to_owned())
        })?,
    })
}

impl Hit {
    /// **What this hit is in**, as the engine addresses it: the subject word
    /// and then whatever address fields the row carried, in order.
    ///
    /// It is prose rather than an address a control can spend, and the pane
    /// says so once above the list rather than on every row — see the module
    /// doc on bl-ef16.
    pub fn subject(&self) -> String {
        std::iter::once(self.at.clone())
            .chain(
                [
                    self.project.clone(),
                    self.id.clone(),
                    self.workspace.clone(),
                    self.agent.clone(),
                ]
                .into_iter()
                .flatten(),
            )
            .collect::<Vec<String>>()
            .join("  ")
    }

    /// **Where in it**: the field the needle was in, and how far into it.
    pub fn at_field(&self) -> String {
        format!("{} +{}", self.field, self.offset)
    }
}

#[cfg(test)]
mod tests;
