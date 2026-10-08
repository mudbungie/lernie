//! **Whether this box is wired** — the one read that asks every wiring fact at
//! once (yog's `docs/REMOTE.md`, *"And one read asks all of it at once"*,
//! bl-28f4; PROTOCOL 17).
//!
//! One row per check: what was checked, the fact found, whether it holds, and —
//! where it does not and there is something to do — the remedy, in the
//! engine's own words. Naming a workspace adds that workspace's checks; the
//! bare form is the box's alone, because the box it is for may hold none.
//!
//! # No tally, on either end
//!
//! Upstream carries none — *"a seat that wants one counts the rows it was
//! handed"* — and the frame's own top-level `ok` is the field every reply
//! carries, not a verdict over the rows: the corpus's own frame says `ok:
//! true` over a failing row. So nothing here reads it as one, and a summary a
//! pane wants is [`failing`], a count of rows, never a second field.
//!
//! # The remedy is verbatim and its absence is the answer
//!
//! A diagnostic with its own opinion is a second authority for a settled
//! decision (the passage above), so the sentence the engine wrote is the
//! sentence painted. A row with no remedy states none, and nothing here
//! composes one.

use serde_json::Value;

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "doctor";

/// One check the engine ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// What was checked — `listener`, `address`, … — carried as the word sent.
    pub check: String,
    /// What was found, in the engine's words.
    pub fact: String,
    /// Whether it holds.
    pub ok: bool,
    /// **What to do about it**, verbatim, where the engine named something.
    pub remedy: Option<String>,
}

/// One row, strictly: three required fields and the optional remedy, each
/// refusal naming the field it refused on ([`super::fields`]).
pub(crate) fn row(value: &Value) -> Result<Check, String> {
    let obj = value.as_object().ok_or("doctor row: not a JSON object")?;
    Ok(Check {
        check: fields::text(obj, "check")?,
        fact: fields::text(obj, "fact")?,
        ok: fields::flag(obj, "ok")?,
        remedy: fields::opt_text(obj, "remedy")?,
    })
}

impl Check {
    /// **The row's one headline**: whether it holds, what was checked, and the
    /// fact — a method for [`super::help::HelpRow::headline`]'s reason, so the
    /// pane, the command line and the suite read one spelling.
    pub fn headline(&self) -> String {
        let mark = if self.ok { "ok" } else { "NOT OK" };
        format!("{mark}  {}: {}", self.check, self.fact)
    }
}

/// **How many rows do not hold** — the count a seat makes of what it was
/// handed, which is the only tally this answer has.
pub fn failing(rows: &[Check]) -> usize {
    rows.iter().filter(|row| !row.ok).count()
}

#[cfg(test)]
mod tests;
