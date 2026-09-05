//! **The words a board row wears** — every line the pane paints, as a pure
//! function of the row it is about (bl-d2af).
//!
//! Split from [`super`] at the design-time budget on the seam that module's
//! own doc draws: [`super`] is *the pane* — what stands where, and which
//! section is silent — and this is *what a row says*. The first changes when
//! the layout does; the second when the wire grows a fact.
//!
//! `crate::ui::records`' rule, which every pane here keeps: the suite reads
//! the sentence rather than the layout, and every token rides verbatim
//! (`crate::reply` rung 3) so a word this build has never seen paints as
//! itself.

use crate::reply::balls::BallRow;
use crate::reply::board::{BoardRow, Fleet};
use crate::reply::spend::Figure;

/// **The one line a row always gets**: which ball, and what it is called.
pub fn headline(row: &BoardRow) -> String {
    format!("{}  {}", row.id, row.title)
}

/// Where the board put it, what the binding says, how urgent it is and whose
/// project it belongs to — the four facts that are true of every row.
pub fn placed(row: &BoardRow) -> String {
    format!(
        "{} · {}  priority {}  in {}",
        row.column, row.state, row.priority, row.project
    )
}

/// **Who holds it and where**, or none for a ball nobody has claimed. The epic
/// rides on the same line, because *whose it is* and *what it is part of* are
/// read together.
pub fn held(row: &BoardRow) -> Option<String> {
    let mut said: Vec<String> = Vec::new();
    if let Some(claimant) = &row.claimant {
        said.push(format!("held by {claimant}"));
    }
    if let Some(workspace) = &row.workspace {
        said.push(format!("on {workspace}"));
    }
    if let Some(parent) = &row.parent {
        said.push(format!("under {parent}"));
    }
    (!said.is_empty()).then(|| said.join("  "))
}

/// **What is blocking its delivery**, in the gating balls' own words — the
/// column `gated` exists for, said as the balls that would open it.
pub fn gated(row: &BoardRow) -> Option<String> {
    (!row.gates.is_empty()).then(|| {
        let by: Vec<String> = row
            .gates
            .iter()
            .map(|gate| format!("{} {} ({})", gate.id, gate.title, gate.mints))
            .collect();
        format!("gated by {}", by.join("  "))
    })
}

/// **The conversations working it**, or none where nothing is.
pub fn worked(row: &BoardRow) -> Option<String> {
    (!row.drones.is_empty()).then(|| {
        let on: Vec<String> = row
            .drones
            .iter()
            .map(|drone| format!("{} ({})", drone.name, drone.root_id))
            .collect();
        format!("worked by {}", on.join("  "))
    })
}

/// **What a figure says**: the engine's own money string where it priced the
/// tokens, the total, and the clause saying what the sum is over.
///
/// Nothing here computes a price. `usd` was derived on the box that holds the
/// rates, and a seat multiplying tokens by a rate of its own would disagree
/// with it quietly — REMOTE §9.17's own reason for putting a classification on
/// the wire, read one noun over.
pub fn cost(figure: &Figure) -> String {
    let mut said: Vec<String> = Vec::new();
    if let Some(usd) = &figure.usd {
        said.push(usd.clone());
    }
    said.push(format!("{} tokens", figure.tokens.total));
    said.push(
        figure
            .attribution
            .label
            .clone()
            .unwrap_or_else(|| figure.attribution.kind.clone()),
    );
    said.join("  ")
}

/// **An armed loop, in the engine's own line** — plus the ceiling that would
/// stop it, which is the one fact the line does not carry.
///
/// The sentence is upstream's: `label` folds the cap, the count, the tick and
/// the lease into words the engine wrote, and a seat re-deriving it from the
/// numbers beside it would be a second opinion about a loop it does not run.
pub fn running(loop_: &Fleet) -> String {
    let mut said = vec![format!(
        "running {} in {}: {}",
        loop_.project, loop_.workspace, loop_.label
    )];
    if !loop_.room {
        said.push("full".to_owned());
    }
    if let Some(ceiling) = &loop_.ceiling {
        said.push(format!("ceiling: {ceiling}"));
    }
    said.join("  ")
}

/// **One binding fact**: which ball, in which state, held by whom and where —
/// each absence said as an absence rather than as a blank.
pub fn binding(row: &BallRow) -> String {
    let mut said = vec![row.ball_id.clone()];
    if let Some(title) = &row.title {
        said.push(title.clone());
    }
    said.push(format!("[{}]", row.state));
    said.push(format!("in {}", row.project));
    match (&row.claimant, &row.workspace) {
        (Some(claimant), Some(workspace)) => said.push(format!("{claimant} on {workspace}")),
        (Some(claimant), None) => said.push(format!("held by {claimant}")),
        (None, _) => said.push("unheld".to_owned()),
    }
    said.join("  ")
}

#[cfg(test)]
mod tests;
