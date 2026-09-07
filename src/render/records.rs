//! **The conversation's own records**: the steps its loop took, one step
//! drilled into, what its worktree holds, its undelivered mail, its spine, and
//! the config commit it resolves policy from. What that policy SAYS is next
//! door ([`super::policy`]).

use crate::reply::files::{Files, Preview};
use crate::reply::governing::Governing;
use crate::reply::inbox::Row as InboxRow;
use crate::reply::rail::Rail;
use crate::reply::steps::Steps;

use super::parts::{brief, clause, line, line_over, listing, things, when};

/// **The steps one loop has taken.** One row a step, and the orphan class
/// beside the heading, because it is a statement about the ledger rather than
/// about any row in it.
pub(super) fn steps(steps: &Steps) -> String {
    let rows = steps
        .rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.seq.clone()),
                Some(row.framing.clone()),
                things(row.attempts, "attempt"),
                Some(format!("{} tokens", row.tokens.total)),
                clause("commit", row.commit.as_deref()),
                clause("started", row.started_at.as_deref()),
                clause("ended", row.ended_at.as_deref()),
                clause("as", row.auth_row.as_deref()),
                when(row.wound != "none", &row.wound),
                clause("—", row.wound_reason.as_deref()),
            ])
        })
        .collect();
    let head = line(vec![
        Some("steps".to_owned()),
        when(steps.orphan != "none", &format!("orphan: {}", steps.orphan)),
        clause("—", steps.orphan_reason.as_deref()),
    ]);
    listing(&head, rows, "this loop has taken no step")
}

/// **What a worktree holds**, and the one file a preview was asked for.
pub(super) fn files(files: &Files) -> String {
    let rows = files
        .listing
        .as_ref()
        .map(|listing| {
            listing
                .rows
                .iter()
                .map(|row| {
                    line(vec![
                        Some(format!("{}{}", row.path, if row.dir { "/" } else { "" })),
                        when(!row.dir, &format!("{} bytes", row.size)),
                    ])
                })
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let head = line(vec![
        Some("files".to_owned()),
        clause("in", files.working_dir.as_deref()),
        when(
            files.listing.as_ref().is_some_and(|l| l.truncated),
            "(truncated)",
        ),
    ]);
    line_over(
        &listing(&head, rows, "no worktree listing came back"),
        files.preview.as_ref().map(|body| preview("preview", body)),
    )
}

/// A bounded log or a file preview, as the three things one can be.
pub(super) fn preview(name: &str, preview: &Preview) -> String {
    match preview {
        Preview::Text(text) => line_over(name, Some(text.clone())),
        Preview::Truncated { text, size } => line_over(
            &format!("{name} (head of {size} bytes)"),
            Some(text.clone()),
        ),
        Preview::Binary { size } => format!("{name}: {size} bytes, binary"),
        Preview::Unknown(word) => {
            format!("{name}: {word} (this seat has no reading of that preview)")
        }
    }
}

/// **The undelivered mail** waiting in one conversation's inbox.
pub(super) fn inbox(rows: &[InboxRow]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            line_over(
                &line(vec![
                    Some(row.name.clone()),
                    clause("from", row.deposit.said_by().as_deref()),
                    clause("at", row.deposit.deposited_at.as_deref()),
                    clause("epitaph:", row.deposit.epitaph.as_deref()),
                    clause("ref", row.deposit.terminal_ref.as_deref()),
                ]),
                Some(row.deposit.body.clone()),
            )
        })
        .collect();
    listing("inbox", painted, "no mail is waiting")
}

/// **The spine**: the commits a fork may be taken off, and the children
/// already taken off them.
pub(super) fn rail(rail: &Rail) -> String {
    let notches = rail
        .notches
        .iter()
        .map(|notch| {
            line(vec![
                Some(notch.seq.clone()),
                Some(notch.short()),
                when(!notch.operable(), "(not operable)"),
                Some(format!("{} tokens", notch.budget)),
                clause(
                    "seat",
                    notch
                        .seat
                        .as_ref()
                        .map(|seat| format!("{} cut {}", seat.row, seat.cut))
                        .as_deref(),
                ),
            ])
        })
        .chain(rail.cards.iter().map(|card| {
            line(vec![
                Some(format!("↳ {}", card.name)),
                Some(card.state.label()),
                Some(format!("off {}", card.fork)),
                Some(format!("notch {}", card.notch)),
                Some(format!("{} tokens", card.tokens)),
                clause("—", card.tail.as_deref().map(brief).as_deref()),
            ])
        }))
        .collect();
    listing("spine", notches, "no operable commit yet")
}

/// **Which config commit a conversation resolves its policy from.**
pub(super) fn governing(governing: &Governing) -> String {
    listing(
        &line(vec![Some("governing".to_owned()), Some(governing.label())]),
        governing.files.clone(),
        "no file governs it",
    )
}
