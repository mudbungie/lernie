//! **What a workspace routes to** — the provider table, and what one row is
//! offering (yog's `docs/REMOTE.md` §8.3; PROTOCOL 13).
//!
//! Three kinds in one module because they are one subject read three ways:
//! `providers` is brazen's ROUTING TABLE for this workspace — the rows the
//! operator named in config plus the built-in presets holding a credential
//! (brazen bl-6feb; lernie bl-67aa) — `catalog` is its SIGN-IN CATALOG, every
//! row `bz --login` can serve whether or not it is signed in (REMOTE §9.27;
//! bl-8c03), and `models` is what one row will answer to. A seat that filed
//! them apart would have split the provider in three. The two listings share
//! one row shape under two kinds, so they share one parser — [`row`] — and a
//! row can be in either without the other: an unsigned oauth preset is
//! catalog-only, a keyed row is table-only. The login pane renders the
//! catalog; every other "which rows exist" question stays on the table.
//!
//! # `blocked` is the whole of whether a row can be signed in to
//!
//! It is a sentence when the row cannot take a sign-in and absent when it can,
//! and the absence is the reading: nothing is wrong with this row. The seat
//! never composes that sentence — the server knows which auth model a row
//! declares and this end does not — so a control is offered off the option's
//! shape and the reason is painted in the server's own words.
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
//! remedy for **every** row on a workspace held elsewhere rather than guessing which
//! rows need it. A stated remedy that is sometimes unnecessary beats a silence
//! that is sometimes wrong.

use serde_json::{Map, Value};

use super::fields;

/// The kind token the table answers to.
pub(crate) const KIND: &str = "providers";
/// The kind token the sign-in catalog answers to — the table's row shape,
/// the other membership.
pub(crate) const CATALOG: &str = "catalog";
/// The kind token one row's offering answers to.
pub(crate) const MODELS: &str = "models";

/// One provider row, as this workspace resolves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRow {
    /// The row's name, which is the address every act in this family takes.
    pub name: String,
    /// What the server says about this row's credential, in its own words.
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

    /// **Whether this row holds no credential and needs one** (bl-7a77) —
    /// read off the server's own words for the row's credential fact, which
    /// end in one of [`BARE`] exactly where its credential model needs a
    /// secret and none is there (yog's `credential_words`). The wire carries
    /// that fact as words and no flag, so the words are the reading; a row
    /// whose fact this build cannot read is not called bare, because absence
    /// of evidence claims nothing.
    pub fn bare(&self) -> bool {
        BARE.iter().any(|tail| self.fact.ends_with(tail))
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

/// The server's endings for a row that needs a credential and holds none: a
/// sign-in row's, then a keyed row's.
pub const BARE: [&str; 2] = ["not signed in", "no credential stored"];

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
