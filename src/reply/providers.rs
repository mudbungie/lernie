//! **What a wall can sign in to** — the provider table, and what one row is
//! offering (yog's `docs/REMOTE.md` §8.3; PROTOCOL 13).
//!
//! Two kinds in one module because they are one subject read at two depths:
//! `providers` is every row brazen routes for this workspace, and `models` is
//! what one of those rows will answer to. A seat that filed them apart would
//! have split the provider in half.
//!
//! # `blocked` is the whole of whether a row can be signed in to
//!
//! It is a sentence when the row cannot take a sign-in and absent when it can,
//! and the absence is the reading: nothing is wrong with this row. The seat
//! never composes that sentence — the engine knows which auth model a row
//! declares and this end does not — so a control is offered off the option's
//! shape and the reason is painted in the engine's own words.
//!
//! # `effort` and `priority` are CAPABILITIES, not settings
//!
//! They say whether this row takes the §9.4 tuning pair at all, which is a
//! different fact from what [`super::roles`] reports about a role. Upstream
//! states the division: *"it gates a control, never a write"*. So they belong
//! to the row rather than to any assignment, and the pane says what a row
//! takes rather than deciding anything with them.
//!
//! # What is NOT here is the flow a row serves
//!
//! yog's own `ProviderRow` carries a `device` column — the headless flow's
//! capability — and its rendered view does not, so the fact does not cross the
//! wire (REMOTE §8.3, bl-7c9f). This seat therefore cannot tell a
//! device-capable row from a browser-only one, and the pane says the loopback
//! remedy for **every** row on a wall held elsewhere rather than guessing which
//! rows need it. A stated remedy that is sometimes unnecessary beats a silence
//! that is sometimes wrong.

use serde_json::{Map, Value};

use super::fields;

/// The kind token the table answers to.
pub(crate) const KIND: &str = "providers";
/// The kind token one row's offering answers to.
pub(crate) const MODELS: &str = "models";

/// One provider row, as this workspace's wall resolves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRow {
    /// The row's name, which is the address every act in this family takes.
    pub name: String,
    /// What the engine says about this row's credential, in its own words.
    pub fact: String,
    /// Why a sign-in cannot be started here, or `None` where one can.
    pub blocked: Option<String>,
    /// Whether this row takes an effort level at all.
    pub effort: bool,
    /// Whether it takes the priority lane.
    pub priority: bool,
}

impl ProviderRow {
    /// **Whether a sign-in can be started on this row** — the option's shape
    /// and nothing else, so the control and the sentence beside it read one
    /// fact.
    pub fn signable(&self) -> bool {
        self.blocked.is_none()
    }

    /// **What this row takes beyond a credential**, as one line, or none where
    /// it takes neither. A row that takes both says so once rather than in two
    /// badges an operator has to read together.
    pub fn takes(&self) -> Option<String> {
        let said: Vec<&str> = [(self.effort, EFFORT), (self.priority, PRIORITY)]
            .into_iter()
            .filter_map(|(takes, word)| takes.then_some(word))
            .collect();
        (!said.is_empty()).then(|| format!("takes {}", said.join(" and ")))
    }
}

/// The two capability words, in the order a row states them.
const EFFORT: &str = "effort";
const PRIORITY: &str = "priority";

/// One row, strictly. Every field is required except the block, whose absence
/// is the reading ([`super::fields`]).
pub(crate) fn row(value: &Value) -> Result<ProviderRow, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("provider row: not an object")?;
    Ok(ProviderRow {
        name: fields::text(obj, "name")?,
        fact: fields::text(obj, "fact")?,
        blocked: fields::opt_text(obj, "blocked")?,
        effort: fields::flag(obj, EFFORT)?,
        priority: fields::flag(obj, PRIORITY)?,
    })
}

/// One model id, which is a bare string rather than an object: the listing has
/// exactly one fact per element and an envelope around it would be a field
/// nobody reads.
pub(crate) fn offered(value: &Value) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "model row: not a string".to_owned())
}

#[cfg(test)]
mod tests;
