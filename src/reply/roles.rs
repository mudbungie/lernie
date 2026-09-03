//! **What one workspace's roles are set to** — the read half of the tuning
//! family (yog's `docs/REMOTE.md` §3; PROTOCOL 6).
//!
//! One row per role the workspace's config declares. It is the same
//! `providers.yaml` assignment [`crate::verbs::tuning`]'s three writes land in,
//! read back from where they wrote it — which is what lets a control open
//! showing what is in force instead of blank.
//!
//! # `effort` is an OPTION and not a closed set, on purpose
//!
//! The gesture that sets it takes one of four things and the boundary refuses a
//! fifth by name. This does not, and upstream states why: *a gesture asserts a
//! level out of a closed set, while this reports what the file holds.* A config
//! written by a hand, or by a yog that knows a word this seat does not, holds
//! whatever it holds — so the reading is [`super`]'s rung 3 and the word is
//! carried **verbatim**, for a pane to paint as itself rather than as a
//! neighbour it is not.
//!
//! `None` is the absence, and the absence is a reading rather than a gap:
//! nothing was requested and the provider's own default governs. There is no
//! word for it on the wire, which is why there is none in the type.
//!
//! # `priority` is a required bool
//!
//! `false` and absent are one fact upstream — off is the provider's own default
//! lane — so the field is always written and a reader must not be made to tell
//! them apart. It is [`super::fields::flag`] and not an option.

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "roles";

/// One role, and the tuning in force on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleRow {
    /// The role's name, as this workspace's config declares it. It is the
    /// address every write in the family takes.
    pub role: String,
    /// The provider row its model calls go down.
    pub provider: String,
    /// The model id bound to it.
    pub model: String,
    /// Whether it asks for the provider's priority lane.
    pub priority: bool,
    /// How much reasoning it asks for, or `None` where it asks for none.
    pub effort: Option<String>,
}

/// One row, strictly. Every field is required except the effort, and each
/// refusal names the field it refused on ([`super::fields`]).
pub(crate) fn row(value: &Value) -> Result<RoleRow, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("role row: not an object")?;
    Ok(RoleRow {
        role: fields::text(obj, "role")?,
        provider: fields::text(obj, "provider")?,
        model: fields::text(obj, "model")?,
        priority: fields::flag(obj, "priority")?,
        effort: fields::opt_text(obj, "effort")?,
    })
}

impl RoleRow {
    /// **What this role runs on**, as one line: the provider row and the model
    /// id, in the spelling `model` takes them back in.
    ///
    /// A method rather than a format string at the one call site, because the
    /// pane that paints it and the editor that pre-fills from it are two
    /// places, and a second spelling of *what a role runs on* would drift the
    /// first time either was reworded.
    pub fn runs_on(&self) -> String {
        format!("{}  {}", self.provider, self.model)
    }
}

#[cfg(test)]
mod tests;
