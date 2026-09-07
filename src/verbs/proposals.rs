//! **The learning loop's operator half** (yog's `docs/REMOTE.md` §9.22,
//! PROTOCOL 18; yog bl-dd88) — what a reviewer has staged for a workspace's
//! config, and the verdict on one of them.
//!
//! A module of its own beside [`super::config`], on the seam the two subjects
//! already have: that one is the config that GOVERNS — the lineages a
//! conversation is born on and the files their tips hold — and this is a
//! CANDIDATE, a patch on `proposal/<reviewer-id>` that no lineage points at
//! until somebody says so. One is what is in force; the other is what has been
//! asked for.
//!
//! # Why the pair exists at all
//!
//! litany's reviewer stages what it learned as a real config commit, and the
//! whole loop turns on a person reading it and vetoing it. Until PROTOCOL 18
//! nothing on this wire could: the veto lived at `litany proposal` on the
//! engine's own box, which on a server install is an `ssh` and a container
//! exec — the thing the §8.5 boundary exists to make unnecessary.
//!
//! # The read's id is optional and the settle's is not
//!
//! [`PROPOSALS`] is one op at two depths — bare it lists, and naming an id
//! answers that proposal whole beside the listing, `files`' own shape. The id
//! is therefore an **optional string**, which the table's rule (*a word and
//! its parameters, all of them named strings*) says is not a parameter, so it
//! rides [`super::Verb::stating`]'s door exactly as `enroll`'s `address` does.
//!
//! [`PROPOSAL`] takes both its id and its verdict as parameters, because
//! upstream requires both and for reasons a seat must not soften: a settle
//! that took *the only one* would do something different the day a second
//! proposal was staged, and a verdict that defaulted would make throwing work
//! away the easy half.

use serde_json::Value;

use super::Verb;

/// **What is staged**, and — where an id is named — one of them whole.
pub const PROPOSALS: Verb = Verb {
    word: "proposals",
    params: &["workspace"],
    flags: &[],
    summary: "what a reviewer has staged for this workspace's config, and how big it is",
    detail: "A reviewer agent that learned something writes it as a real \
             config patch and parks it on a branch of its own, waiting for \
             you. This lists them: which reviewer staged each one, which \
             lineage it would move, how many lines it changes, and what the \
             reviewer called it. `fresh` means the lineage still stands where \
             the reviewer read it, so the patch can be taken as it is; `stale` \
             means somebody advanced that lineage since, and the answer is to \
             reject it and let the next checkpoint re-derive from where things \
             now stand. Nothing staged is an empty answer, not a refusal. The \
             wire also takes an `id`, which answers that proposal whole — \
             message and diff — beside the listing.",
};

/// **The verdict on one of them.**
pub const PROPOSAL: Verb = Verb {
    word: "proposal",
    params: &["workspace", "id", "verdict"],
    flags: &[],
    summary: "take a reviewer's staged config patch, or throw it away",
    detail: "The decision the learning loop is built around: read what an \
             agent concluded, then say yes or no. `accept` fast-forwards the \
             patch's lineage onto it and deletes the staging branch, and every \
             conversation on that lineage picks the change up at its next step \
             with no act per conversation. `reject` deletes the staging branch \
             and nothing else — the reviewer's own conversation survives as \
             the record of why it thought so. Both words are required, and \
             neither has a default. An accept whose lineage moved under it \
             refuses and names where that lineage now stands, rather than \
             merging somebody's memory forward.",
};

/// **The two verdicts, in the order a control offers them**, and the wire's
/// own words rather than a translation of them — [`super::capability`]'s rule,
/// applied again, so no table exists to drift.
///
/// The order is what they do: the one that takes the work, then the one that
/// throws it away, which is also the order that puts the destructive half
/// second.
pub const VERDICTS: [&str; 2] = ["accept", "reject"];

/// The listing, typed. **The id is stated only when one was named** — absent
/// is absent, never `null`, because the bare listing's absence is the fact.
pub fn proposals(workspace: String, id: Option<String>) -> Value {
    PROPOSALS.stating(vec![workspace], "id", id)
}

/// The settle, typed.
pub fn proposal(workspace: String, id: String, verdict: String) -> Value {
    PROPOSAL.built(vec![workspace, id, verdict], &[])
}

#[cfg(test)]
mod tests;
