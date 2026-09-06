//! **One step, drilled into** — the four records its loop wrote, the tool
//! calls it made, and its two bounded logs.
//!
//! Split from [`super::records`] at the design-time budget, and the seam is
//! real: every other record is a LISTING of steps or files, scanned; this one
//! is a single step's captured documents, printed. The reply vocabulary makes
//! the same distinction — it is one of the two shapes big enough to be boxed.
//!
//! **A document is printed as what it is and never parsed.** `src/reply/step.rs`
//! decodes the class and carries the bytes, deliberately, so a rendering that
//! reformatted them would be inventing a reading the seat does not have.

use crate::reply::step::{Doc, Step};

use super::parts::{line, line_over, listing, when};
use super::records::preview;

/// **One step, drilled into.** Four records, the tools it called, and the two
/// bounded logs — each document printed as what it is rather than parsed,
/// which is the reply vocabulary's own posture toward them.
pub(super) fn step(step: &Step) -> String {
    let parts = [
        Some(doc("meta", &step.meta)),
        Some(doc("request", &step.request)),
        Some(doc("staging", &step.staging)),
        when(
            !step.response.is_empty(),
            &step
                .response
                .iter()
                .map(|body| doc("response", body))
                .collect::<Vec<String>>()
                .join("\n"),
        ),
        when(
            !step.tools.is_empty(),
            &step
                .tools
                .iter()
                .map(|call| {
                    line_over(
                        &line(vec![
                            Some(call.tool_id.clone()),
                            when(call.is_error, "ERROR"),
                        ]),
                        Some(format!(
                            "{}\n{}",
                            doc("in", &call.input),
                            doc("out", &call.output)
                        )),
                    )
                })
                .collect::<Vec<String>>()
                .join("\n"),
        ),
        step.stderr.as_ref().map(|log| preview("stderr", log)),
        step.driver.as_ref().map(|log| preview("driver", log)),
    ];
    listing(
        &format!("step {}", step.seq),
        parts.into_iter().flatten().collect(),
        "",
    )
}

/// One captured document, under the name of the record it is.
fn doc(name: &str, doc: &Doc) -> String {
    match doc {
        Doc::Json { raw } => line_over(name, Some(raw.clone())),
        Doc::Absent => format!("{name}: not written"),
        Doc::Unparsed { note, raw } => line_over(&format!("{name} ({note})"), Some(raw.clone())),
        Doc::Unknown(word) => format!("{name}: {word} (this seat has no reading of that record)"),
    }
}
