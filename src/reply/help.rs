//! **What one engine answers to** — its own verb table, published as a reply
//! (yog's `docs/REMOTE.md` §3; PROTOCOL 7).
//!
//! One row per op the engine has a word for: the line to type, one sentence on
//! what it is for, the page under that, and the classification saying whether
//! the op is spoken by an operator or by a program.
//!
//! # This is the same table this seat is JUDGED by
//!
//! `crate::snapshot::parity::roster` reads the `surface` field off the vendored
//! fixture of this very shape, because it is the one home for the suite-level
//! fact *which ops owe every seat a discoverable interactable* (yog's
//! `docs/PARITY.md` §2). So the pane an operator reads and the ledger that
//! reddens when a control is missing come off one answer, and there is no
//! second list anywhere in this crate.
//!
//! # `surface` rides verbatim
//!
//! [`super`]'s rung 3: a classification is carried as the word the engine sent
//! and never narrowed to a closed set here. The parity roster has to refuse a
//! word it has no reading for — it decides what this seat *owes*, and guessing
//! there would quietly shrink the obligation — but a pane only shows what an
//! op is for, and a word a newer engine grew paints as itself rather than as a
//! neighbour it is not.
//!
//! # Why the wire `help` is not `lernie help`
//!
//! `crate::verbs::help` answers *what does this BINARY take*, from a table
//! compiled into it, with nothing provisioned and no engine up. This answers
//! *what does that ENGINE offer*, which is a different question with a
//! different author — and the two are painted in different places for that
//! reason (`crate::ui::commands`).

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "help";

/// One op the engine has a word for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpRow {
    /// The op itself — the same token the envelope's discriminant, the corpus
    /// filename and an `act:` tag all spell (`crate::ui::act`).
    pub verb: String,
    /// The line an operator types on the engine's own control line.
    pub usage: String,
    /// One sentence: what the op is for.
    pub summary: String,
    /// The page: what it answers with, and what to know before spending it.
    pub detail: String,
    /// **Who the op is for** — `control` for an op every seat owes an
    /// interactable, `machine` for one spoken by programs. Carried verbatim.
    pub surface: String,
}

/// One row, strictly. Every field is required, and each refusal names the
/// field it refused on ([`super::fields`]).
pub(crate) fn row(value: &Value) -> Result<HelpRow, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("help row: not an object")?;
    Ok(HelpRow {
        verb: fields::text(obj, "verb")?,
        usage: fields::text(obj, "usage")?,
        summary: fields::text(obj, "summary")?,
        detail: fields::text(obj, "detail")?,
        surface: fields::text(obj, "surface")?,
    })
}

impl HelpRow {
    /// **The row's one headline**: what to type, and who it is for.
    ///
    /// A method rather than a format string at the call site, for
    /// [`super::roles::RoleRow::runs_on`]'s reason: the pane paints it and the
    /// suite reads it back, and a second spelling would drift the first time
    /// either was reworded.
    pub fn headline(&self) -> String {
        format!("{}  [{}]", self.usage, self.surface)
    }
}

#[cfg(test)]
mod tests;
