//! **The config lineages one workspace holds** (yog's `docs/REMOTE.md` §9.3) —
//! the policy branches a conversation is born on, each with its tip and the
//! files that tip carries.
//!
//! It is the listing a config read indexes into: a lineage names a branch and
//! its `files` are the paths a `config` gesture may address on it, so the two
//! pickers above the editor are this one answer read twice (DESIGN §4.30).
//!
//! # The oid is carried in both spellings, and neither is derived
//!
//! `oid` and `short_oid` arrive together and this seat abbreviates nothing: how
//! many characters are unambiguous is a property of the repository the engine
//! holds, so a seat that truncated the full oid itself would be guessing at a
//! collision domain it cannot see.

use serde_json::Value;

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "lineages";

/// One config lineage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineage {
    /// The branch's name — the address a config gesture on it carries.
    pub name: String,
    /// Its tip commit, whole.
    pub oid: String,
    /// The same commit as the engine abbreviates it.
    pub short_oid: String,
    /// When that commit was made, in seconds since the epoch. Signed, because
    /// clock skew between two machines is a fact and not an error.
    pub committed: i64,
    /// Every path the tip holds — what a read on this lineage may address.
    pub files: Vec<String>,
}

/// One row of the listing.
pub(crate) fn row(value: &Value) -> Result<Lineage, String> {
    let obj = value.as_object().ok_or("lineage: not a JSON object")?;
    Ok(Lineage {
        name: fields::text(obj, "name")?,
        oid: fields::text(obj, "oid")?,
        short_oid: fields::text(obj, "short_oid")?,
        committed: fields::secs(obj, "committed")?,
        files: fields::list(obj, "files", path)?,
    })
}

/// One path on a lineage's tip.
fn path(value: &Value) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "lineage: a file is not a string".to_owned())
}

impl Lineage {
    /// **What the row says in one clause** — the branch, its tip as the engine
    /// abbreviates it, and how much it holds.
    pub fn line(&self) -> String {
        format!(
            "{}  @{}  {} file(s)",
            self.name,
            self.short_oid,
            self.files.len()
        )
    }
}

#[cfg(test)]
mod tests;
