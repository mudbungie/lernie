//! **The work an engine is holding**: the binding table, the fleet board, one
//! wall's own balls and the delivery attempts. What those attempts CHANGED is
//! a diff rather than a listing, so it is next door ([`super::work`]). The three
//! reads whose subject is the engine rather than a wall are next door
//! ([`super::reads`]).

use crate::reply::balls::{BallRow, BoundBall};
use crate::reply::board::Board;
use crate::reply::science::Attempt;

use super::parts::{brief, clause, line, line_over, listing, tally, things, when};

/// **Every ball this box can see, and the wall it is bound to.**
pub(super) fn balls(rows: &[BallRow]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.ball_id.clone()),
                Some(row.project.clone()),
                Some(row.state.clone()),
                clause("→", row.workspace.as_deref()),
                clause("claimed by", row.claimant.as_deref()),
                row.title.as_deref().map(brief),
            ])
        })
        .collect();
    listing("balls", painted, "this box is bound to no ball")
}

/// **The fleet board**: every live ball in its column, and the loops running
/// them.
pub(super) fn board(board: &Board) -> String {
    let rows = board
        .rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.id.clone()),
                Some(row.column.clone()),
                Some(row.state.clone()),
                Some(format!("p{}", row.priority)),
                Some(row.project.clone()),
                clause("→", row.workspace.as_deref()),
                clause("claimed by", row.claimant.as_deref()),
                clause("under", row.parent.as_deref()),
                things(row.gates.len() as u64, "gate"),
                tally(row.drones.len() as u64, "under it"),
                Some(brief(&row.title)),
            ])
        })
        .collect();
    line_over(
        &listing("board", rows, "nothing is on the board"),
        Some(
            board
                .fleet
                .iter()
                .map(|fleet| {
                    line(vec![
                        Some(format!("loop {}", fleet.workspace)),
                        Some(fleet.project.clone()),
                        Some(format!("{}/{}", fleet.count, fleet.cap)),
                        when(!fleet.room, "at its cap"),
                        clause("ceiling", fleet.ceiling.as_deref()),
                        Some(fleet.label.clone()),
                    ])
                })
                .collect::<Vec<String>>()
                .join("\n"),
        ),
    )
}

/// **The balls one wall holds**, with what each has cost.
pub(super) fn workspace_balls(rows: &[BoundBall]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.id.clone()),
                clause("", row.badge.as_deref()),
                Some(row.project.clone()),
                Some(row.state.clone()),
                Some(format!("owner {}", row.owner)),
                Some(format!("{} tokens", row.spend.tokens.total)),
                clause("$", row.spend.usd.as_deref()),
            ])
        })
        .collect();
    listing("this wall's balls", painted, "this wall holds no ball")
}

/// **Every delivery attempt of one wall**, with what it cost and how it ended.
pub(super) fn science(rows: &[Attempt]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            line_over(
                &line(vec![
                    Some(row.diff.ball_id.clone()),
                    Some(row.outcome.state.clone()),
                    clause("as", row.conversation.as_deref()),
                    Some(format!("{} steps", row.steps)),
                    Some(format!("{}s", row.wall_secs)),
                    Some(format!("{} in / {} out", row.usage.input, row.usage.output)),
                    clause("commit", row.outcome.commit.as_deref()),
                    clause("by", row.outcome.by.as_deref()),
                    things(row.verdicts.len() as u64, "verdict"),
                ]),
                row.goal.as_deref().map(brief),
            )
        })
        .collect();
    listing("attempts", painted, "this wall has attempted nothing")
}
