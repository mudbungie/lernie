//! **The undelivered mail** (yog's `docs/REMOTE.md` §8.5; bl-3257) — one
//! conversation's deposit files, each parsed beside its verbatim bytes.
//!
//! Delivered mail is not here: it has moved into the transcript. What is here
//! is what is still waiting to be read by the conversation itself.
//!
//! # The parse is forgiving, and the raw is why that is safe
//!
//! Upstream: *"a file without a well-formed `---` frontmatter block renders as
//! a raw body with every field absent, so a half-written or hand-edited
//! deposit never becomes an error."* So a [`Deposit`] with nothing but a body
//! is not a malformed answer — it is the engine reporting what the file
//! actually stated. And [`Row::raw`] rides beside it because *"the parsed view
//! drops the envelope"*: without the bytes, a hand-edited deposit would be
//! unreachable rather than merely unrendered.
//!
//! # Absent fields are absent keys, never empty strings
//!
//! Upstream, verbatim: *"a forgiving parse of a hand-edited file says 'this
//! was not stated', and an empty `from:` would be a different claim."* Two of
//! the four are stated only on a **result** message — a subagent's report of
//! how it ended and the commit it ended at — so their absence is the ordinary
//! deposit rather than a gap.

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "inbox";

/// One deposit file: what it is called, what it says, and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The deposit's filename.
    pub name: String,
    /// The file's bytes, unaltered.
    pub raw: String,
    /// What the frontmatter stated, as far as it stated anything.
    pub deposit: Deposit,
}

/// One deposit as parsed: four optional frontmatter facts, and the body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deposit {
    /// Who sent it.
    pub from: Option<String>,
    /// When it was deposited.
    pub deposited_at: Option<String>,
    /// **How the sending agent ended**, on a result message — its epitaph,
    /// carried verbatim ([`super`]'s rung 3), because the engine passes a word
    /// it does not know through rather than refusing one.
    pub epitaph: Option<String>,
    /// The commit it ended at, beside that epitaph.
    pub terminal_ref: Option<String>,
    /// The content. Always stated, and empty is a real reading: a result
    /// whose agent never spoke.
    pub body: String,
}

/// One row, strictly ([`super`]'s rung 1). The deposit object is required —
/// the encoder writes it for every row — and everything inside it is not.
pub(crate) fn row(value: &Value) -> Result<Row, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("inbox row: not an object")?;
    Ok(Row {
        name: fields::text(obj, "name")?,
        raw: fields::text(obj, "raw")?,
        deposit: fields::object(obj, "deposit", deposit)?,
    })
}

/// The parsed deposit.
fn deposit(obj: &Map<String, Value>) -> Result<Deposit, String> {
    Ok(Deposit {
        from: fields::opt_text(obj, "from")?,
        deposited_at: fields::opt_text(obj, "deposited_at")?,
        epitaph: fields::opt_text(obj, "epitaph")?,
        terminal_ref: fields::opt_text(obj, "terminal_ref")?,
        body: fields::text(obj, "body")?,
    })
}

#[cfg(test)]
mod tests;
