//! **The machines registered in one workspace** — who participates in it, who
//! is connected right now, and what each of them offers (yog's
//! `docs/REMOTE.md` §5, §5.1; PROTOCOL 2's two fields).
//!
//! # Presence is an observation and the set is a statement
//!
//! The two facts on a row have different lifetimes and upstream says so:
//! [`ClientRow::present`] is true *at the moment the question was answered* and
//! nothing durable records it, while [`ClientRow::tools`] was written when that
//! machine last presented its set and stands whether or not it is connected. So
//! a row with no connection and a full set is the ordinary reading of a foot
//! that is not currently waiting for work — not a stale answer — and the pane
//! says both rather than folding them into one word.
//!
//! # `subject_cwd` is the consent, and its absence is the answer
//!
//! REMOTE §5.1: *"`true` states that the advertising box consents to run this
//! tool at a working directory the invocation names … Absent reads false,
//! rides only when true, and a mistyped value refuses at the read."* This
//! reader is that sentence exactly, including the strictness: `null` refuses
//! here as it does upstream, because the two ends must agree on what an absence
//! is and only one of them can be the author of that rule.
//!
//! # The schema is not decoded, because nothing paints it
//!
//! A tool's `input_schema` is its statement to a MODEL — yog carries it
//! verbatim and never validates it — and an operator looking at a roster of
//! machines is asking what a box can do, not what shape its arguments take. So
//! it rides through unread ([`super`]'s rung 4). The commit that paints a
//! schema is the commit that decodes one.

use serde_json::Value;

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "clients";

/// One machine registered in the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRow {
    /// Its name — the certificate common name an operator registered it under,
    /// and the address `invoke` routes a call by.
    pub client: String,
    /// **Whether it held a live connection at the moment this was answered.**
    /// True only then: a machine that answers here may be gone a second later,
    /// which is why nothing on either end writes it down.
    pub present: bool,
    /// What it has advertised, as of the last set it presented.
    pub tools: Vec<ToolRow>,
}

/// One tool a machine offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRow {
    /// The handle a call addresses, a single path component.
    pub name: String,
    /// The host's own words for what it does.
    pub description: String,
    /// **Whether the advertising box consents to run it at a working directory
    /// the invocation names** (REMOTE §5.1) — the fact the engine's worktree
    /// lane routes on, and the one an operator cannot otherwise see.
    pub subject_cwd: bool,
}

/// One row of the listing.
pub(crate) fn row(value: &Value) -> Result<ClientRow, String> {
    let obj = value.as_object().ok_or("client: not a JSON object")?;
    Ok(ClientRow {
        client: fields::text(obj, "client")?,
        present: fields::flag(obj, "present")?,
        tools: fields::list(obj, "tools", tool)?,
    })
}

/// One advertised tool.
fn tool(value: &Value) -> Result<ToolRow, String> {
    let obj = value.as_object().ok_or("tool: not a JSON object")?;
    Ok(ToolRow {
        name: fields::text(obj, "name")?,
        description: fields::text(obj, "description")?,
        subject_cwd: fields::absent_is_false(obj, "subject_cwd")?,
    })
}

impl ClientRow {
    /// **What the row says about itself in one clause** — connected or not, and
    /// how much it offers. The count rather than the set, because the set is
    /// painted under it and a number is what a glance wants.
    pub fn line(&self) -> String {
        format!(
            "{}  — {}, {} tool(s)",
            self.client,
            if self.present { HERE } else { AWAY },
            self.tools.len()
        )
    }
}

/// What a row says of a machine holding a connection at the moment it answered.
pub const HERE: &str = "connected now";
/// And of one that was not. **Not an error and not a stale set**: a tool host
/// holds its connection only while it is waiting for work, so a busy machine
/// looks exactly like an absent one from here.
pub const AWAY: &str = "not connected at this moment";

#[cfg(test)]
mod tests;
