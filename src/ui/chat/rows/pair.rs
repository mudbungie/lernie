//! **A call and its result, met on one row** (DESIGN §4.42, item 4; bl-eb4e).
//!
//! The engine writes a call and its result as two entries, and the live lane
//! carries them as two transitions; the operator reads them as one act —
//! `bash("cargo test") ✔`, with what came back folded beneath it. So the
//! pairing is a pass over the projection, keyed on the tool-use id both
//! halves carry, and the id stops there: it is the wire's bookkeeping.
//!
//! **Nothing is dropped.** A result whose call is not on this read stands as a
//! step of its own, and a call still in flight is a call row with no mark —
//! the two cases the pairing cannot make one, painted as what they are.

use super::{Kind, Row};
use crate::ui::theme::glyph;

/// How much of a call's input its head carries, in characters.
const HEAD: usize = 60;

/// **The rows with every answered call met with its result.** A result joins
/// the latest unanswered call with its id; the row keeps the call's place, so
/// the trail still reads in the order things were asked.
pub(super) fn met(rows: Vec<Row>) -> Vec<Row> {
    let mut out: Vec<Row> = Vec::with_capacity(rows.len());
    for row in rows {
        let asked = (row.kind == Kind::Result)
            .then(|| {
                out.iter_mut()
                    .rev()
                    .find(|call| call.kind == Kind::Call && call.call == row.call)
            })
            .flatten();
        match asked {
            Some(call) => {
                call.kind = Kind::Answered;
                call.said = row.said;
                call.fold = row.fold;
                call.failed = row.failed;
            }
            None => out.push(row),
        }
    }
    out
}

/// **A call's head**: its name and the head of its input, one line, elided.
/// A JSON object's first value stands for the whole — `bash("ls -la")` reads
/// where `bash({"command":"ls -la"})` was a record — and anything else is
/// its own first line.
pub(super) fn head(name: &str, input: &str) -> String {
    if input.is_empty() {
        return name.to_owned();
    }
    let argument = match serde_json::from_str::<serde_json::Value>(input) {
        Ok(serde_json::Value::Object(map)) => map
            .values()
            .next()
            .map_or_else(String::new, ToString::to_string),
        Ok(value) => value.to_string(),
        Err(_) => input.to_owned(),
    };
    let line = argument.lines().next().unwrap_or_default();
    let mut said: String = line.chars().take(HEAD).collect();
    if said.len() < argument.len() {
        said.push_str(glyph::ELIDED);
    }
    format!("{name}({said})")
}

#[cfg(test)]
mod tests;
