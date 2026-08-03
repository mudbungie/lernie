//! Hunk location and splicing for [`super::apply`]'s stage phase,
//! split out to honor the per-file line cap. Pure over the file's text:
//! every hunk is located through the [`super::seek`] matching ladder —
//! cursor moving forward, uniqueness required — and spliced, yielding
//! the post-state content and the per-hunk report rows.

use super::apply::Error;
use super::parse::Hunk;
use super::report::HunkReport;
use super::seek::{self, LADDER};

/// Locate and splice every hunk, in order, cursor moving forward.
pub(super) fn run_hunks(
    text: &str,
    hunks: &[Hunk],
    path: &str,
) -> Result<(String, Vec<HunkReport>), Error> {
    let had_nl = text.ends_with('\n');
    let mut lines: Vec<String> = if text.is_empty() {
        Vec::new()
    } else {
        text.split('\n').map(String::from).collect()
    };
    if had_nl {
        lines.pop();
    }
    let mut cursor = 0usize;
    let mut applied = Vec::new();
    for (n, hunk) in hunks.iter().enumerate() {
        let hunk_no = n + 1;
        for anchor in &hunk.anchors {
            let what = format!("anchor {anchor:?}");
            let (pos, _) = seek::seek(&lines, std::slice::from_ref(anchor), cursor, false)
                .map_err(located(path, hunk_no, what))?;
            cursor = pos + 1;
        }
        if hunk.old.is_empty() && !hunk.eof && hunk.anchors.is_empty() {
            return Err(Error::UnanchoredInsertion {
                path: path.to_string(),
                hunk: hunk_no,
            });
        }
        // An empty needle (pure insertion) always locates — at the end
        // under `eof`, else right after the anchor — so the miss arm is
        // only reachable with context lines present.
        let (pos, rung) = seek::seek(&lines, &hunk.old, cursor, hunk.eof).map_err(located(
            path,
            hunk_no,
            "context".to_string(),
        ))?;
        let matched: Vec<String> = lines[pos..pos + hunk.old.len()].to_vec();
        lines.splice(pos..pos + hunk.old.len(), hunk.new.iter().cloned());
        cursor = pos + hunk.new.len();
        applied.push(HunkReport {
            rung: rung.label(),
            line: pos + 1,
            matched: (rung != LADDER[0]).then_some(matched),
        });
    }
    let mut content = lines.join("\n");
    if had_nl && !content.is_empty() {
        content.push('\n');
    }
    Ok((content, applied))
}

/// Convert a seek miss into the right decline, with its location facts.
fn located(path: &str, hunk: usize, what: String) -> impl FnOnce(seek::Error) -> Error {
    let path = path.to_string();
    move |source| match source {
        seek::Error::NotFound => Error::NotFound {
            path,
            hunk,
            what,
            source,
        },
        seek::Error::Ambiguous { .. } => Error::Ambiguous {
            path,
            hunk,
            what,
            source,
        },
    }
}
