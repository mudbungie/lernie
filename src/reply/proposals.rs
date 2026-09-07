//! **What a reviewer has staged for one workspace's config, and one of them
//! whole** (yog's `docs/REMOTE.md` §9.22, PROTOCOL 18; yog bl-dd88).
//!
//! litany's reviewer agent writes what it learned as a real config patch and
//! parks it on `proposal/<reviewer-id>`, a branch no lineage points at. The
//! whole learning loop turns on a person reading that patch and vetoing it,
//! and until this shape existed nothing on this wire could: `lineages`
//! enumerates `config/*` only and `governing` answers the commit a
//! conversation resolves, never a candidate. So the veto lived at `litany
//! proposal` on the engine's own box — an `ssh` and a container exec on a
//! server install, which is the thing the §8.5 boundary exists to make
//! unnecessary.
//!
//! # `fresh` crosses rather than being inferred
//!
//! A seat could read freshness off an empty `lineages` and would be right,
//! but that is a RULE, and REMOTE §9.4's discipline is that the wire states a
//! derivation so no seat has to own one. Only the engine can be sure the two
//! were read in one pass, which is what makes the pair honest — so `fresh` is
//! carried as answered and this end holds no second opinion about it.
//!
//! # One op at two depths, and `whole` is absent rather than null
//!
//! Naming an id answers the listing AND that proposal's message and diff, the
//! shape `files` already uses: a seat that named one has already been given
//! the row it names, so a second ask would be a second derivation of one
//! subject. The bare listing carries no `whole` key at all, because absence is
//! the fact.

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "proposals";

/// One staged proposal, as a row of the listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    /// The reviewer that staged it — the branch is `proposal/<id>`, and the id
    /// is the address `/proposal` settles it by.
    pub id: String,
    /// The config lineages whose head is this patch's parent. Empty is the
    /// stale case: somebody advanced the lineage after the reviewer read it.
    pub lineages: Vec<String>,
    /// The commit the reviewer read and staged against.
    pub parent: String,
    /// **Whether that parent is still a lineage head**, answered by the engine
    /// in the pass that read the lineages.
    pub fresh: bool,
    /// git's own `--shortstat` for the patch: how much it changes.
    pub diffstat: String,
    /// What the reviewer called it — the commit's subject line.
    pub subject: String,
}

/// What a proposals read answers: the listing, and — when the read named one —
/// that proposal whole.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposals {
    pub rows: Vec<Proposal>,
    /// The named proposal's message and diff. Absent for a bare listing.
    pub whole: Option<String>,
}

/// The reply, read.
pub(crate) fn proposals(obj: &Map<String, Value>) -> Result<Proposals, String> {
    Ok(Proposals {
        rows: fields::rows(obj, row)?,
        whole: fields::opt_text(obj, "whole")?,
    })
}

/// One lineage name off a row. The list is read STRICTLY — required, empty
/// included — because empty is the stale case and therefore a fact, where an
/// absent key is an engine that did not answer the question: collapsing the
/// two would read *"nobody has advanced this lineage"* as *"this patch is
/// stale"*, which are opposite readings of one proposal.
fn lineage(value: &Value) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "proposal: a lineage is not a string".to_owned())
}

/// One row of the listing.
fn row(value: &Value) -> Result<Proposal, String> {
    let obj = value.as_object().ok_or("proposal: not a JSON object")?;
    Ok(Proposal {
        id: fields::text(obj, "id")?,
        lineages: fields::list(obj, "lineages", lineage)?,
        parent: fields::text(obj, "parent")?,
        fresh: fields::flag(obj, "fresh")?,
        diffstat: fields::text(obj, "diffstat")?,
        subject: fields::text(obj, "subject")?,
    })
}

impl Proposal {
    /// **The word the row is in**, and it is the engine's own reading rather
    /// than this seat's: `fresh` means the lineage still stands where the
    /// reviewer read it, so the patch can be taken as it is, and `stale` means
    /// somebody advanced it since — a patch written against a config that no
    /// longer governs anything, whose answer is to reject it and let the next
    /// checkpoint re-derive from where things now stand.
    pub fn standing(&self) -> String {
        if self.fresh { FRESH } else { STALE }.to_owned()
    }

    /// **Which lineage it would move**, or the sentence that says none does.
    /// An empty list is exactly the stale case, so it is not a missing value
    /// and nothing stands in for one.
    pub fn moves(&self) -> String {
        if self.lineages.is_empty() {
            return NO_LINEAGE.to_owned();
        }
        self.lineages.join(", ")
    }
}

/// The two words the engine's own help spells a proposal's standing in.
const FRESH: &str = "fresh";
const STALE: &str = "stale";

/// What a stale proposal moves, said rather than left blank.
const NO_LINEAGE: &str = "no lineage still heads at its parent";

#[cfg(test)]
mod tests;
