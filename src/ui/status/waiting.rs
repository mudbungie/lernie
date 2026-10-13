//! **The seat-wide waiting count, off the rule** (DESIGN §4.42, item 11):
//! the queue's door where seat-wide facts stand — the seat panel and the
//! switcher's head — beside `/attention` itself.
//!
//! It counts only channels that dial (bl-4cac): a channel that will not keeps
//! the walls it last answered, and a green read off them is a count nobody
//! can act on.

use crate::ui::{Model, theme};

/// **How many conversations are asking**, across every engine that dials.
pub fn waiting(model: &Model) -> u64 {
    model
        .roster
        .iter()
        .filter(|chunk| !matches!(chunk.held, crate::ui::Held::Unheld(_)))
        .flat_map(|chunk| chunk.walls.iter())
        .map(|wall| wall.attention)
        .sum()
}

/// **The count as the queue's door**: `N waiting` in the attention accent,
/// tagged as `/attention` is, and nothing while none is asking. Answers
/// whether it was clicked; the caller fires the word.
pub fn door(ui: &mut egui::Ui, model: &Model) -> bool {
    let asking = waiting(model);
    if asking == 0 {
        return false;
    }
    let ink = theme::accent(theme::State::Attention);
    let seat = theme::paint::link(ui, &format!("{asking} waiting"), ink);
    crate::ui::act::tag(&seat, &[crate::verbs::ATTENTION.word]);
    seat.clicked()
}
