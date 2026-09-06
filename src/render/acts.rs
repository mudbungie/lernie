//! **What an ACT came back saying**: a captured run, a staged start, a
//! sign-in, an enrollment, and the receipts that carry one fact or two.
//!
//! A receipt is a sentence rather than a listing, and that is not brevity: the
//! reply vocabulary is emphatic that a receipt carries nothing it would have
//! to predict — what changed is on the read the operator takes next. So the
//! rendering says what landed and stops, exactly as the frame does.

use crate::reply::Outcome;
use crate::reply::enrolled::Enrolled;
use crate::reply::login::Signin;
use crate::reply::providers::ProviderRow;
use crate::reply::start::Prepared;

use super::parts::{brief, clause, line, line_over, listing, when};

/// **A captured run**: the verdict off the exit code, and what the child said.
///
/// Both streams are printed and neither is elided — a run's output is the
/// whole of what it answered, and a `bl close` that failed says why in the
/// last line of a long stderr.
pub(super) fn outcome(outcome: &Outcome) -> String {
    listing(
        &line(vec![
            Some(if outcome.ok() { "ok" } else { "failed" }.to_owned()),
            when(!outcome.ok(), &format!("exit {}", outcome.exit)),
        ]),
        [
            when(!outcome.stdout.is_empty(), outcome.stdout.trim_end()),
            when(!outcome.stderr.is_empty(), outcome.stderr.trim_end()),
        ]
        .into_iter()
        .flatten()
        .collect(),
        "",
    )
}

/// **A start, staged.** The goal the rung composed is the operator's next
/// edit, so it is what the line says.
pub(super) fn prepared(prepared: &Prepared) -> String {
    line_over(
        &format!("staged in {}", prepared.workspace),
        Some(prepared.goal.clone()),
    )
}

/// **A spread**, which is one staged body per candidate (§4.36).
pub(super) fn fanned(candidates: &[Prepared]) -> String {
    listing(
        "candidates",
        candidates.iter().map(prepared).collect(),
        "the spread produced no candidate",
    )
}

/// **A new box's material.** The listing names the six facts and prints none
/// of the three secrets: the reply carries a private key, and the destination
/// of this stream is a terminal's scrollback (REMOTE §8.4). `lernie enroll`
/// draws the symbol instead ([`crate::seat::enroll`]), which is what this
/// answer is for; a rendering that spilled the key into a log would defeat the
/// one act on this surface that deliberately writes nothing down.
pub(super) fn enrolled(enrolled: &Enrolled) -> String {
    format!(
        "enrolled {} at {} ({} grade) — its material is in this frame and is not printed; \
         `lernie enroll` shows the symbol that carries it",
        enrolled.name, enrolled.address, enrolled.grade
    )
}

/// **One sign-in run**, as the engine streamed it.
pub(super) fn login(signin: &Signin) -> String {
    listing(
        &line(vec![
            Some("sign-in".to_owned()),
            clause(
                "exit",
                signin.outcome.map(|code| code.to_string()).as_deref(),
            ),
            clause("fallback:", signin.fallback.as_deref()),
        ]),
        signin.lines.iter().map(|said| said.text.clone()).collect(),
        "it said nothing",
    )
}

/// **What a wall can sign in to.**
pub(super) fn providers(rows: &[ProviderRow]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.name.clone()),
                when(row.signable(), "signable"),
                clause("blocked:", row.blocked.as_deref().map(brief).as_deref()),
                row.takes(),
                Some(row.fact.clone()),
            ])
        })
        .collect();
    listing("providers", painted, "this wall can sign in to nothing")
}

/// **What one provider row is offering** — the list `lernie model` is checked
/// against (bl-1e5a).
pub(super) fn models(rows: &[String]) -> String {
    listing("models", rows.to_vec(), "that provider offers no model")
}

/// **A parked call was answered**, and whether the release drove the
/// conversation on.
pub(super) fn answered(tool: &str, tool_use: &str, verdict: &str, advanced: bool) -> String {
    line(vec![
        Some(format!("{tool} ({tool_use}): {verdict}")),
        Some(if advanced { "advanced" } else { "still parked" }.to_owned()),
    ])
}

/// **A candidate was accepted**, and the identities its delivery acted on.
pub(super) fn delivered(
    base: &str,
    target: &str,
    source: Option<&str>,
    commit: Option<&str>,
) -> String {
    line(vec![
        Some(format!("delivered onto {target}")),
        Some(format!("base {base}")),
        clause("from", source),
        clause("as", commit),
        when(commit.is_none(), "(it landed nothing)"),
    ])
}

/// **A candidate's worktree was released**, and what the declared retention
/// did with its source ref.
pub(super) fn retired(discarded: bool) -> String {
    if discarded {
        "retired, and its source ref went with it".to_owned()
    } else {
        "retired; its source ref is kept".to_owned()
    }
}

/// **A capability floor was written** — re-derived, so it says what STANDS
/// rather than what was asked.
pub(super) fn floored(standing: bool) -> String {
    if standing {
        "a floor stands over it".to_owned()
    } else {
        "no floor stands over it".to_owned()
    }
}

/// **Whether a standing thing now stands** — the receipt four ops share, which
/// is why it says neither which loop nor which monitor.
pub(super) fn armed(standing: bool) -> String {
    if standing {
        "it is armed".to_owned()
    } else {
        "it is not armed".to_owned()
    }
}
