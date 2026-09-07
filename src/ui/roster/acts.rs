//! **The window's own acts** — the ops whose subject is every channel this box
//! holds, and the band above the roster's channels where they hang (bl-f0ef,
//! bl-40ec, bl-4c48, bl-d2af; DESIGN §4.21).
//!
//! Split from [`super`] at the design-time budget on a seam that is a real
//! one: that module is *the list of walls this seat can aim at*, and this is
//! *the acts that address none of them*. The first changes when a roster row
//! grows a fact; the second when a window-level op lands a surface.
//!
//! **They are rows, not chips** (`docs/STYLE.md` §2): a desktop's global
//! navigation is a band at the top of the leftmost column, and the six entries
//! are places you go rather than buttons you press. A grid of bordered chips
//! is a face with no hierarchy, which is what this band was.

use crate::ui::{Model, theme};

/// The word on the control that asks every channel for its roster again.
pub const REFRESH: &str = "ask the channels again";

/// **One entry of the band**: a full-width row, one word, no state rule and
/// never chosen — the band is where you GO, and where you are is said by the
/// column that filled in, not by a highlight up here.
fn entry(ui: &mut egui::Ui, word: &str, ink: egui::Color32) -> egui::Response {
    theme::paint::row(ui, word, ink, None, false, 0.0)
}

/// **Paint the band and take the clicks on it.**
///
/// Seven ops name no workspace, so their subject is every channel this box
/// holds and none of them hangs off a row: the decision queue's `attention`,
/// this pane's own `workspaces`, the trail's `ops`, the ball pane's `balls`
/// and `board`, the engines' verb table and a search. The
/// roster is their home because it is the pane that is already the union
/// across channels, and they are offered on a seat that has aimed at nothing —
/// which is the seat most likely to be asking any of them.
///
/// They sit outside the scrolled region for the reason the heading does, and
/// stand down under a covering pane exactly as the per-wall controls do: what
/// each opens would replace what is standing there.
///
/// **The queue's entry is the one that wears a colour** (§2: *the queue's
/// entry wears the attention accent while anything is waiting, so the one
/// control most worth pressing is the one that is coloured*). It reads the
/// same union the pane itself paints, and the roster's own attention rollups
/// beside it — a section holding no row is an engine that answered and holds
/// nothing — so a window with nothing waiting has nothing green on it, which
/// is the fact read the other way.
///
/// **[`REFRESH`] is the one that opens nothing**, and it is the affordance
/// this pane owed its own read (yog's `docs/PARITY.md` §2: the interactable a
/// query owes a seat is the one that reaches the view it populates). What it
/// adds over the standing read's cadence is where a FAILURE lands: a channel
/// that cannot be reached says so under its own header
/// (`crate::ui::Model::unreachable`), and until this control existed that
/// sentence only ever appeared on a beat nobody could ask for.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let again = entry(ui, REFRESH, theme::INK_WEAK);
    crate::ui::act::tag(&again, &[crate::verbs::WORKSPACES.word]);
    if again.clicked() {
        model.refresh_roster();
    }
    // **Read off the roster's own rows as well as the queue's**: the queue's
    // read stands only while its pane is open, and a wall's `attention`
    // rollup is on the glass from the first roster answer — so the entry is
    // green the moment anything asks, not the second time the pane is opened.
    let asking = model.waiting.iter().any(|section| !section.rows.is_empty())
        || model
            .roster
            .iter()
            .any(|chunk| chunk.walls.iter().any(|wall| wall.attention > 0));
    let ink = if asking {
        theme::accent(theme::State::Attention)
    } else {
        theme::INK_WEAK
    };
    let waiting = entry(ui, crate::ui::queue::OPEN, ink);
    crate::ui::act::tag(&waiting, &[crate::verbs::ATTENTION.word]);
    if waiting.clicked() {
        model.begin_queue();
    }
    // **The opening control carries `help`'s token because it ASKS it**
    // (`crate::ui::model::window`): opening the pane composes the gesture,
    // the way the records control carries both of its reads' tokens.
    let answers = entry(ui, crate::ui::commands::OPEN, theme::INK_WEAK);
    crate::ui::act::tag(&answers, &[crate::verbs::HELP.word]);
    if answers.clicked() {
        model.begin_commands();
    }
    // **The trail's control carries `ops` for the queue's reason** — the
    // read stands on the pane, so opening it is what asks (`crate::ui::
    // model::trail`).
    let trail = entry(ui, crate::ui::trail::OPEN, theme::INK_WEAK);
    crate::ui::act::tag(&trail, &[crate::verbs::OPS]);
    if trail.clicked() {
        model.begin_trail();
    }
    // **The ball pane's control carries FOUR tokens**, which is PARITY
    // §3's rule that the ledger's unit is the op rather than the widget,
    // spent on the widest control this window has. Opening the pane stands
    // all four reads up (`crate::state::Open::Board`): two of every channel
    // this box holds, and — when the window is aimed at a wall — two of
    // that wall. The two aimed ones are asked by this same click and by no
    // other, so this is where their token belongs.
    let board = entry(ui, crate::ui::board::OPEN, theme::INK_WEAK);
    crate::ui::act::tag(
        &board,
        &[
            crate::verbs::BALLS.word,
            crate::verbs::BOARD.word,
            crate::verbs::WORKSPACE_BALLS.word,
            crate::verbs::MARKS.word,
        ],
    );
    if board.clicked() {
        model.begin_board();
    }
    // **And this one carries none**: it opens a pane and crosses no wire.
    // `search` is spent by the control inside it, which is where the
    // needle it requires is (`crate::ui::find`) — the same division
    // `enroll a box…` keeps with `mint`.
    if entry(ui, crate::ui::find::OPEN, theme::INK_WEAK).clicked() {
        model.begin_finding();
    }
}

#[cfg(test)]
mod tests;
