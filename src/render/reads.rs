//! **The three reads whose subject is an engine rather than a wall** (DESIGN
//! §4.21, §4.27): its own verb table, what a needle found across everything it
//! can see, and the trail of every act that crossed its boundary.
//!
//! Split from [`super::tasks`] at the design-time budget, on the seam the
//! gestures themselves already have: these three name no workspace, so each is
//! asked of every channel this box holds and answered per channel
//! ([`crate::seat::fanned`]), where a ball or a board is one wall's.

use crate::reply::help::HelpRow;
use crate::reply::ops::OpRow;
use crate::reply::search::Found;

use super::parts::{brief, line, line_over, listing, quoted, when};

/// **The trail**: every act that crossed one engine's boundary.
pub(super) fn ops(rows: &[OpRow]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.ts.clone()),
                // **Who made the act** (REMOTE §9.20). It is beside the
                // origin rather than in the detail because it is what tells
                // two identical rows apart, and nothing later can recover it.
                Some(row.client.clone()),
                Some(row.origin.clone()),
                Some(row.standing.clone()),
                when(row.failed, &format!("{} ({})", row.exit_label, row.exit)),
                Some(brief(&row.argv)),
            ])
        })
        .collect();
    listing("trail", painted, "the trail is empty")
}

/// **What a needle found**, and what could not be read looking for it — two
/// different claims, so they are two sections.
pub(super) fn found(found: &Found) -> String {
    let hits = found
        .rows
        .iter()
        .map(|hit| {
            line(vec![
                Some(hit.subject()),
                Some(hit.at_field()),
                quoted(&hit.excerpt),
            ])
        })
        .collect();
    line_over(
        &listing(
            &format!("found {:?}", found.needle),
            hits,
            "no match anywhere this engine can see",
        ),
        when(
            !found.unreadable.is_empty(),
            &format!("unread: {}", found.unreadable.join(", ")),
        ),
    )
}

/// **One engine's own verb table** — what that engine answers to.
pub(super) fn help(rows: &[HelpRow]) -> String {
    let painted = rows
        .iter()
        .map(|row| line(vec![Some(row.usage.clone()), Some(row.summary.clone())]))
        .collect();
    listing("the engine's own words", painted, "this engine names no op")
}
