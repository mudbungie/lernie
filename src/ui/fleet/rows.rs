//! **The words a fleet row wears** — every line the pane paints, as a pure
//! function of the row it is about (bl-a43a).
//!
//! Split from [`super`] at the design-time budget on the seam that module's
//! own doc draws: [`super`] is *the pane* — the five acts and where each
//! stands — and this is *what a row says*. Every token rides verbatim
//! (`crate::reply` rung 3), so an outcome or a diff state this build has never
//! seen paints as itself.

use crate::reply::diff::{Churn, Diff};
use crate::reply::science::Attempt;
use crate::ui::model::Armed;

/// **The receipt four ops share, said in the op's own name.**
///
/// The op leads because it is the only thing that distinguishes the fleet loop
/// from the alignment monitor here: one `armed` reply answers both families,
/// and a sentence that dropped the op would be a sentence about neither.
pub fn receipt(said: &Armed) -> String {
    format!(
        "{}: {}",
        said.op,
        if said.armed {
            "it is standing"
        } else {
            "it is not standing"
        }
    )
}

/// **One attempt, as the lines it earns** — never a fixed number of them,
/// because every column upstream leaves out is a fact nobody recorded rather
/// than a blank to print.
pub fn attempt(row: &Attempt) -> Vec<String> {
    let mut said = vec![format!(
        "{}  {}  [{}]",
        row.diff.ball_id,
        row.diff.project,
        ending(row)
    )];
    if let Some(goal) = &row.goal {
        said.push(goal.clone());
    }
    said.push(format!(
        "{} steps  {}s  {} in  {} out  {} cache-read  {} cache-write",
        row.steps,
        row.wall_secs,
        row.usage.input,
        row.usage.output,
        row.usage.cache_read,
        row.usage.cache_write
    ));
    for (what, held) in [
        ("in", &row.conversation),
        ("from", &row.base),
        ("governed by", &row.governing),
        ("said", &row.response),
    ] {
        if let Some(value) = held {
            said.push(format!("{what} {value}"));
        }
    }
    if !row.pins.is_empty() {
        said.push(format!("pinned {}", row.pins.join("  ")));
    }
    for verdict in &row.verdicts {
        said.push(format!("{} — {}", verdict.sender, verdict.body));
    }
    if let Some(n) = row.compacted {
        said.push(format!("{n} entries compacted out from under it"));
    }
    said.push(changed(&row.diff));
    said
}

/// **How an attempt ended**, in the engine's own token plus whatever that
/// token could say — and the seat adds no reading of its own to either.
pub fn ending(row: &Attempt) -> String {
    let mut said = vec![row.outcome.state.clone()];
    for (what, held) in [("at", &row.outcome.commit), ("by", &row.outcome.by)] {
        if let Some(value) = held {
            said.push(format!("{what} {value}"));
        }
    }
    said.join(" ")
}

/// **One diff row's own line**: which ball, in which state, between which two
/// refs — and what that state has to say instead where it has no refs.
pub fn changed(row: &Diff) -> String {
    let mut said = vec![format!("{}  [{}]", row.ball_id, row.state)];
    if let (Some(source), Some(target)) = (&row.source, &row.target) {
        said.push(format!("{source} → {target}"));
    }
    if let Some(handle) = &row.handle {
        said.push(format!("candidate {handle}"));
    }
    if let Some(delivered) = &row.delivered {
        said.push(format!("delivered {delivered}"));
    }
    if !row.missing.is_empty() {
        said.push(format!("no such ref: {}", row.missing.join("  ")));
    }
    if row.truncated == Some(true) {
        said.push("(the listing was cut)".to_owned());
    }
    said.join("  ")
}

/// **One changed file**, with binary read off the SHAPE rather than a token —
/// upstream writes a count or it writes `binary`, never both, so the reading
/// that asks which fields are there cannot disagree with the encoder.
pub fn churn(file: &Churn) -> String {
    match (file.added, file.removed) {
        (Some(added), Some(removed)) => format!("{}  +{added} −{removed}", file.path),
        _ => format!("{}  binary", file.path),
    }
}

#[cfg(test)]
mod tests;
