//! **Which config commit governs this conversation** (yog's `docs/REMOTE.md`
//! §9.7, §9.12; bl-b52c).
//!
//! # The shape whose MEANING moved, which is the one drift a corpus cannot see
//!
//! At PROTOCOL 5 (REMOTE §9.12) this reply *"lost `branch` and gained
//! `follows` and `diverged_lineages`, and its `oid` changed meaning"*:
//!
//! > **`oid` was the fork commit and is the resolved one.** Before, it named
//! > the `config/*` ancestor an agent's branch forked off, a commit that never
//! > moved. Now it names the commit control actually reads at every step
//! > boundary: the followed lineage's head. A seat that painted the old value
//! > as *what this conversation runs* would keep painting, and would keep
//! > being wrong, which is why the number has to move even though the key did
//! > not.
//!
//! The bytes stayed well-formed across that bump, so nothing mechanical here
//! could have caught it — which is why the trap is written at `PROTOCOL` and
//! restated at the one reader that spends it. This decoder is the first thing
//! in this crate to read the field at all, and it reads it under the new
//! meaning.
//!
//! # One enum, two keys, and neither is redundant
//!
//! Upstream, verbatim: *"`follows` is the lineage's name and
//! `diverged_lineages` is `0`; or `follows` is `null` and the count is how
//! many distinct lineage tips reached the conversation and therefore held it
//! on its fork commit. The decoder rebuilds the enum off `follows` alone and
//! reads the count only where it can be non-zero, so the pair cannot decode to
//! a state the encoder could not have written."* [`Governance`] is that enum,
//! rebuilt off `follows` alone for exactly that reason.
//!
//! # The sentence is upstream's two wordings, not one composed here
//!
//! REMOTE §9.7 rules it directly — *"a seat renders it through
//! `GoverningConfig::label()`'s two wordings and composes no sentence of its
//! own"* — so [`Governing::label`] is that function's text, and the pane
//! paints what it answers.

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "governing";

/// The config commit a conversation resolves its policy from, and the lineage
/// that settled it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Governing {
    /// The resolved commit, whole — what a `git show` outside yog takes.
    pub oid: String,
    /// The same commit clipped, which is what a label wears. Carried rather
    /// than derived: unlike a rail notch's, this one is the engine's own
    /// clipping of a commit it resolved, and both keys ride the wire.
    pub short_oid: String,
    /// Which lineage governs, and how.
    pub governance: Governance,
    /// Every path the governing commit's tree holds — the souls, the workflow,
    /// the provider table this conversation is actually running under.
    pub files: Vec<String>,
}

/// **How the policy is settled**: one lineage followed, or several diverged
/// over the fork commit and none may be guessed between.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Governance {
    /// Exactly one lineage reaches it, named bare.
    Follows(String),
    /// Two or more distinct tips reach it, so it is held on the fork commit.
    Held { diverged: u64 },
}

impl Governing {
    /// **The engine's own sentence about it**, in its two wordings — never a
    /// third composed here (REMOTE §9.7).
    pub fn label(&self) -> String {
        match &self.governance {
            Governance::Follows(lineage) => {
                format!("policy follows config/{lineage}, now at {}", self.short_oid)
            }
            Governance::Held { diverged } => format!(
                "policy held at {} — {diverged} diverged config lineages",
                self.short_oid
            ),
        }
    }
}

/// The whole answer, strictly ([`super`]'s rung 1). `follows` is `null` rather
/// than absent, matching the key it replaced, and [`fields::opt_text`] reads
/// both spellings alike.
pub(crate) fn governing(obj: &Map<String, Value>) -> Result<Governing, String> {
    let governance = match fields::opt_text(obj, "follows")? {
        Some(lineage) => Governance::Follows(lineage),
        None => Governance::Held {
            diverged: fields::count(obj, "diverged_lineages")?,
        },
    };
    Ok(Governing {
        oid: fields::text(obj, "oid")?,
        short_oid: fields::text(obj, "short_oid")?,
        governance,
        files: fields::list(obj, "files", path)?,
    })
}

/// One path out of the governing tree's listing.
fn path(value: &Value) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "governing: a non-string path in \"files\"".to_owned())
}

#[cfg(test)]
mod tests;
