//! **What an ACT came back saying**: a captured run, a staged start, a
//! sign-in, an offer and the pending ones, and the receipts that carry one fact or two.
//!
//! A receipt is a sentence rather than a listing, and that is not brevity: the
//! reply vocabulary is emphatic that a receipt carries nothing it would have
//! to predict — what changed is on the read the operator takes next. So the
//! rendering says what landed and stops, exactly as the frame does.

use crate::reply::Outcome;
use crate::reply::login::Signin;
use crate::reply::offers::{OfferRow, Offered};
use crate::reply::providers::ProviderRow;
use crate::reply::start::Prepared;

use super::parts::{brief, clause, line, line_over, listing, when};

/// **A captured run**: what the child said, under a verdict only where the
/// words do not already carry it.
///
/// Both streams are printed and neither is elided — a run's output is the
/// whole of what it answered, and a `bl close` that failed says why in the
/// last line of a long stderr. **A run that succeeded and spoke is told in
/// its words alone** (bl-401b): `ok` over them restated an exit code the
/// sentence already carried, and in the state slot it read as this seat's
/// own prefix on the engine's receipt. The words are the child's verbatim
/// (REMOTE §8.2) and nothing here rewrites them — a sentence that names a
/// conversation by an id this seat paints nowhere else is the engine's to
/// change, not a prefix to strip. `ok` is said only over silence, because a
/// run that said nothing must still read apart from one that never fired
/// (bl-9760).
pub(super) fn outcome(outcome: &Outcome) -> String {
    let said: Vec<String> = [&outcome.stdout, &outcome.stderr]
        .into_iter()
        .map(|stream| stream.trim_end())
        .filter(|stream| !stream.is_empty())
        .map(str::to_owned)
        .collect();
    if outcome.ok() {
        return if said.is_empty() {
            "ok".to_owned()
        } else {
            said.join("\n")
        };
    }
    listing(&format!("failed  exit {}", outcome.exit), said, "")
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

/// **A minted offer** (REMOTE §15.1): the URL as text and as a symbol — the
/// same bytes, one for a paste and one for a camera — and the expiry in
/// words, then what happens next. Nothing in it is a secret, so all of it is
/// printed.
pub(super) fn offered(offered: &Offered) -> String {
    let symbol = match crate::qr::Symbol::encode(offered.url.as_bytes()) {
        Ok(symbol) => symbol.block(),
        Err(too_long) => format!("(no symbol: the URL will not fit one — {too_long})"),
    };
    format!(
        "offer minted — the new device opens this URL, or scans the symbol:\n\n{}\n\n{symbol}\n\
         It works once, and only until {} — ten minutes from when it was minted. The device \
         then shows a six-character confirmation word: `lernie offers <workspace>` shows the \
         same word on its row, and `lernie approve <workspace> <name>` seats it.",
        offered.url,
        crate::reply::offers::clock(offered.expires)
    )
}

/// **A workspace's pending offers**: one line each, and under an unclaimed
/// one, when its code expires.
pub(super) fn offers(rows: &[OfferRow]) -> String {
    listing(
        "offers",
        rows.iter()
            .map(|row| {
                line(vec![
                    Some(row.line()),
                    when(row.word.is_none(), "code expires"),
                    when(
                        row.word.is_none(),
                        &crate::reply::offers::clock(row.expires),
                    ),
                    when(row.proved(), "— can be approved"),
                ])
            })
            .collect(),
        "no offer is pending in this workspace",
    )
}

/// **One sign-in run**, as the server streamed it.
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

/// **What a workspace routes to** — the table.
pub(super) fn providers(rows: &[ProviderRow]) -> String {
    table(rows, "providers", "this workspace routes to no provider")
}

/// **What a workspace can sign in to** — the catalog, the same rows painted
/// the same way under its own heading (bl-8c03).
pub(super) fn catalog(rows: &[ProviderRow]) -> String {
    table(rows, "catalog", "this workspace can sign in to nothing")
}

/// One provider listing: a line per row, or `empty` where there is none.
fn table(rows: &[ProviderRow], heading: &str, empty: &str) -> String {
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
    listing(heading, painted, empty)
}

/// **What one provider row is offering** — the list `lernie model` is checked
/// against (bl-1e5a).
pub(super) fn models(rows: &[String]) -> String {
    listing("models", rows.to_vec(), "that provider offers no model")
}

/// **A parked call was answered**, how far the answer stands, and whether the
/// release drove the conversation on.
///
/// The reach is the server's own reading and not an echo of what was asked: a
/// wide answer on a destructive or credential-reaching call is narrowed there,
/// so this line is the one place an operator learns what their instruction
/// actually covered.
pub(super) fn answered(
    tool: &str,
    tool_use: &str,
    verdict: &str,
    scope: &str,
    advanced: bool,
) -> String {
    line(vec![
        Some(format!("{tool} ({tool_use}): {verdict}")),
        Some(format!("for this {scope}")),
        Some(if advanced { "advanced" } else { "still parked" }.to_owned()),
    ])
}

/// **`seen` answered**: what it acted on, then the queue that remains on that
/// server — the same listing `attention` prints, because it is the same rows
/// (bl-6802).
pub(super) fn acknowledged(acked: &crate::reply::queue::Acknowledged) -> String {
    format!(
        "seen {}/{}\n{}",
        acked.workspace,
        acked.agent,
        super::workspaces::attention(&acked.rows)
    )
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

#[cfg(test)]
mod tests;
