//! **What one wall's agents changed** — the diff a person reads, rather than
//! the row of counters the wire carries (bl-2e1d).
//!
//! # This wire is a numstat and never a patch
//!
//! yog reads an attempt with `git diff --numstat` and answers a path and two
//! counts a file (`src/workdiff/read.rs`), so there is no hunk on this surface
//! to print and no diff LINE to omit. The named form that answers one file's
//! patch is not composed by this seat at all — [`crate::verbs::WORK_DIFF`]
//! says so in its own detail. What a diff rendering asks for that this frame
//! can answer is three things, and each of them was missing:
//!
//! - **the two ends being compared**, which is this wire's `a/… → b/…`. One
//!   path a changed file is the whole of what a churn carries, so the header
//!   naming both sides is the ROW's — and it is where the two oids belong,
//!   which were decoded and then printed nowhere.
//! - **a totals line**, folded over the churn, so how big a change is reads
//!   off one line rather than out of a column to add up.
//! - **an explicit cut.** `truncated` was decoded and dropped, so a listing
//!   the engine stopped early read as the whole change. That is the one
//!   reading on this surface that states something the frame does not (DESIGN
//!   §4.37), and it is the rule the worktree listing already holds with its
//!   own `(truncated)`.
//!
//! # Every absence is said, and none of the three is an empty list
//!
//! `unreadable`, `absent` and a `diff` that changed nothing are three
//! different facts and the wire spells all three ([`crate::reply::diff`]).
//! The first two are the row's own clauses; the third is a `diff` row whose
//! `files` is empty, which is *the attempt has not written anything yet* and
//! not *this row has no listing*. The shape says which — `truncated` is
//! written by the `diff` state alone — so the sentence is read off the frame
//! rather than off the state word, which rides verbatim.

use crate::reply::diff::{Churn, Diff};

use super::parts::{clause, line, line_over, listing, things, when};

/// **What one wall's agents changed** — a header a row, the churn under it,
/// and an explicit mark wherever something was left out.
pub(super) fn work(rows: &[Diff]) -> String {
    let painted = rows.iter().map(row).collect();
    listing("work", painted, "nothing has changed")
}

/// One attempt: which ball, which two ends, how much moved, and what moved.
fn row(row: &Diff) -> String {
    let head = line(vec![
        Some(row.ball_id.clone()),
        Some(row.project.clone()),
        Some(row.state.clone()),
        clause("→", row.target.as_deref()),
        clause("from", row.source.as_deref()),
        clause("at", ends(row).as_deref()),
        when(
            !row.missing.is_empty(),
            &format!("missing {}", row.missing.join(", ")),
        ),
        clause("delivered", row.delivered.as_deref()),
    ]);
    line_over(&head, body(row))
}

/// **The two commits the comparison is between**, which is the one fact a
/// header of a diff is for: the refs above say what was asked, and these say
/// what was read. Absent unless the frame resolved both — a half-named
/// comparison is not one.
fn ends(row: &Diff) -> Option<String> {
    let (target, source) = (row.target_oid.as_ref()?, row.source_oid.as_ref()?);
    Some(format!("{}..{}", short(target), short(source)))
}

/// How much of an object name a person reads. Long enough to tell two apart
/// on one screen, and the whole of it is one `--json` away.
fn short(oid: &str) -> String {
    oid.chars().take(SHORT_OID).collect()
}

/// Characters of an object name a header carries.
const SHORT_OID: usize = 12;

/// **The totals, the churn and the cut**, under the header — or the sentence
/// a `diff` row that changed nothing earns instead.
fn body(row: &Diff) -> Option<String> {
    if row.truncated.is_some() && row.files.is_empty() {
        return Some("nothing changed on this branch yet".to_owned());
    }
    let lines = std::iter::once(totals(&row.files))
        .chain(row.files.iter().map(changed))
        .chain(when(row.truncated == Some(true), CUT))
        .filter(|said| !said.is_empty())
        .collect::<Vec<String>>();
    (!lines.is_empty()).then(|| lines.join("\n"))
}

/// **The line an elision is said on.** The count it dropped is genuinely not
/// on this wire — the engine truncates to its own bound and says only that it
/// did — so the sentence states what it knows and names what it does not,
/// rather than implying a whole listing by saying nothing.
const CUT: &str = "… the engine stopped listing here — how many more files \
                   changed is not on the wire";

/// **How big the change is, in one line.** The file count, the churn summed
/// over the files that carry one, and the binaries counted apart, because no
/// line count describes them and folding them in would understate every
/// total they were part of.
fn totals(files: &[Churn]) -> String {
    let sum = |added: bool| -> u64 {
        files
            .iter()
            .filter_map(|churn| if added { churn.added } else { churn.removed })
            .sum()
    };
    let binaries = files
        .iter()
        .filter(|churn| churn.binary == Some(true))
        .count() as u64;
    line(vec![
        things(files.len() as u64, "file"),
        when(
            files
                .iter()
                .any(|churn| churn.added.is_some() || churn.removed.is_some()),
            &format!("+{} -{}", sum(true), sum(false)),
        ),
        things(binaries, "binary"),
    ])
}

/// One changed file: its path, and its churn in the shape upstream wrote it.
fn changed(churn: &Churn) -> String {
    line(vec![
        Some(churn.path.clone()),
        counts(churn),
        when(churn.binary == Some(true), "binary"),
    ])
}

/// A file's own churn, absent on the file that carries none.
fn counts(churn: &Churn) -> Option<String> {
    let said = [
        churn.added.map(|n| format!("+{n}")),
        churn.removed.map(|n| format!("-{n}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<String>>()
    .join(" ");
    (!said.is_empty()).then_some(said)
}

#[cfg(test)]
mod tests;
