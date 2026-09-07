//! **The conversation index**, which is the one read of the four whose
//! listing has a RULE and not just a row shape (bl-96cd, bl-00f5).
//!
//! Split from [`super`] at the 300-line cap on the seam that module's own doc
//! draws: it is *what a wall holds*, four reads of it, and this is the one
//! that folds. They change for different reasons — a field added moves a row,
//! and the fold moves only when what hangs under a conversation changes.

use crate::reply::convs::ConvRow;

use super::super::parts::{
    age, brief, clause, line, line_over, listing, quoted, tally, things, when,
};

/// **One wall's conversations, folded to their roots** (bl-96cd, bl-00f5).
///
/// # An index whose majority is machinery is an index people stop reading
///
/// One session of ordinary work answered twelve rows, seven of them
/// compactors the operator never started — each with a minted name as
/// memorable as the work's, each at the same weight, three of the seven
/// belonging to one conversation. The rows were not wrong. But this is the
/// listing a person scans to find out what is happening, and the answer was
/// mostly bookkeeping.
///
/// bl-00f5 fixed it in the window and this is the same fix at the other face,
/// which is the one a headless operator, a script and an `ssh` session have.
///
/// # The rule is about DESCENT, and not about compactors
///
/// The wire carries no kind — a row spells its address, its badge, its preview
/// and how far it hangs under its root, and nothing on it says *this one is a
/// compactor*. Nothing should: a fork's candidate is a depth-1 row the
/// operator DID start, and a seat guessing from a preview would be reading
/// prose to make a structural decision. **The list is the conversations, and
/// what hangs under one is under it.**
///
/// # The subtree is NAMED rather than opened
///
/// The window folds and unfolds with a click against a set of what is open
/// (`crate::ui::Model::unfolded`). A command has no such state and should not
/// grow one: a flag would have to cross the wire to be a verb's flag
/// (`crate::verbs::Verb::flags` writes envelope fields, and yog's REMOTE §8.5
/// keeps folds off the boundary on purpose), and a second output form beside
/// `--json` would be a third [`super::Form`]. So a root says how many hang
/// under it and what they are CALLED, on one line — which is the CLI's version
/// of *one gesture away*: every name printed is the address `agent`,
/// `transcript` and `follow` already take, so nothing is out of reach and
/// nothing needs a second round trip. `--json` still answers every row, as it
/// must.
pub(crate) fn conversations(rows: &[ConvRow]) -> String {
    let painted = forest(rows)
        .into_iter()
        .map(|(row, under)| {
            let head = line(vec![
                Some(row.display.clone()),
                Some(row.state.label()),
                when(row.uncertain, "(uncertain)"),
                Some(age(row.age_secs)),
                things(row.members, "member"),
                tally(row.depth, "deep"),
                tally(row.attention, "waiting"),
                clause("failed:", row.failure.as_deref().map(brief).as_deref()),
            ]);
            line_over(&head, Some(beneath(row, &under)))
        })
        .collect();
    listing(
        "conversations",
        painted,
        "none yet — `lernie start <workspace> \"<goal>\"` begins one",
    )
}

/// **The roots, each with what hangs under it.**
///
/// The order is the engine's — a root, then its descendants, deepest last
/// (REMOTE §9.7's descent forest) — so one pass with no recursion and no
/// second index is the whole of it.
///
/// **A deep row with no root above it is its own row**, never dropped. It can
/// only mean the answer began mid-forest, and a listing that quietly lost rows
/// would be a worse failure than one that indents oddly — the same reason
/// `--json` keeps answering all of them.
fn forest(rows: &[ConvRow]) -> Vec<(&ConvRow, Vec<String>)> {
    let mut out: Vec<(&ConvRow, Vec<String>)> = Vec::new();
    for row in rows {
        match out.last_mut() {
            Some((_, under)) if row.depth > 0 => under.push(row.display.clone()),
            _ => out.push((row, Vec::new())),
        }
    }
    out
}

/// What goes under a root: its preview, and the subtree it is standing for.
///
/// The names are elided at the same width a preview is, so a fork spreading
/// twelve candidates costs one line rather than twelve rows — and the elision
/// is marked, which is what says to ask `--json`.
fn beneath(row: &ConvRow, under: &[String]) -> String {
    [
        quoted(&row.preview),
        (!under.is_empty())
            .then(|| format!("{} under it: {}", under.len(), brief(&under.join(", ")))),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<String>>()
    .join("\n")
}

#[cfg(test)]
mod tests;
