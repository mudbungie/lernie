//! **A procedure run is a marker on the message that fired it** (§4.47;
//! bl-1d79 — the lernie bl-454d ruling: a titler, a compactor, a guard or an
//! eval is not a conversation and never a row of the list, and what it did
//! shows where it happened).
//!
//! The server already places every child: a `reply/rail` card hangs from its
//! provenance notch, and a notch carries its chat seat — the entry its rule
//! is drawn above (yog `docs/REMOTE.md` §9.28). So a card that carries
//! `procedure` is painted **above the first row of that entry**, in weak ink,
//! one line: the role, the binding that forked it, and what came of it — the
//! landed file and its words, or the state while it runs. Nothing is derived
//! here that the card does not say; a card no notch seats paints nowhere,
//! because there is nowhere honest to put it.

use crate::reply::rail::{Card, Rail};
use crate::ui::theme;

/// The marker lines that stand above `entry`, in card order.
pub(super) fn markers(rail: Option<&Rail>, entry: &str) -> Vec<String> {
    let Some(rail) = rail else {
        return Vec::new();
    };
    rail.cards
        .iter()
        .filter(|card| card.procedure.is_some())
        .filter(|card| {
            usize::try_from(card.notch)
                .ok()
                .and_then(|at| rail.notches.get(at))
                .and_then(|notch| notch.seat.as_ref())
                .is_some_and(|seat| seat.row == entry)
        })
        .map(line)
        .collect()
}

/// One run as a line: `titler · user_message · landed title: “…” · 12 tokens`.
pub fn line(card: &Card) -> String {
    let role = card.role.as_deref().unwrap_or(&card.name);
    let binding = card.procedure.as_deref().unwrap_or_default();
    let outcome = match (&card.landed, &card.tail) {
        (Some(file), Some(said)) => format!("landed {file}: \u{201c}{said}\u{201d}"),
        (Some(file), None) => format!("landed {file}"),
        (None, _) => card.state.label().clone(),
    };
    format!(
        "{role} \u{b7} {binding} \u{b7} {outcome} \u{b7} {}",
        crate::render::counted(card.tokens, "token")
    )
}

/// Paint the markers, each its own weak line.
pub(super) fn paint(ui: &mut egui::Ui, lines: &[String]) {
    for said in lines {
        ui.add_space(theme::space::S);
        ui.colored_label(theme::INK_WEAK, said);
    }
}

#[cfg(test)]
mod tests;
