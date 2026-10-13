//! **What the rule counts across every channel this seat holds** (`super`):
//! the conversations asking and the engines reached — the two facts read off
//! the whole roster rather than off the selection, and the two a channel that
//! will not dial changes (bl-4cac).
//!
//! Split from [`super`] at the line cap on that seam.

use super::{Run, run};
use crate::ui::{Model, theme};

/// **How many conversations are asking, across every engine this seat
/// reaches** — and only those: a channel that will not dial keeps the walls it
/// last answered (`crate::ui::roster` paints them), but a count read off them
/// would be a green that nobody can act on, so it is not counted (bl-4cac).
pub fn waiting(model: &Model) -> u64 {
    model
        .roster
        .iter()
        .filter(|chunk| !matches!(chunk.held, crate::ui::Held::Unheld(_)))
        .flat_map(|chunk| chunk.walls.iter())
        .map(|wall| wall.attention)
        .sum()
}

/// **The engines fact counts the reached, not the held** (bl-4cac): `N
/// engines` while every channel has answered, `R of N engines` while any has
/// not — in the error accent where one will not dial, weak while one has
/// merely not answered yet.
pub fn engines(model: &Model) -> Run {
    use crate::ui::Held;
    let held = model.roster.len();
    let reached = model
        .roster
        .iter()
        .filter(|chunk| chunk.held == Held::Heard)
        .count();
    let noun = if held == 1 { "engine" } else { "engines" };
    if reached == held {
        return run(format!("{held} {noun}"), theme::INK_WEAK);
    }
    let down = model
        .roster
        .iter()
        .any(|chunk| matches!(chunk.held, Held::Unheld(_)));
    let ink = if down {
        theme::accent(theme::State::Error)
    } else {
        theme::INK_WEAK
    };
    run(format!("{reached} of {held} {noun}"), ink)
}
