//! **One engine's row**, and the cursor track the accordion makes of the pane
//! (DESIGN §4.39).
//!
//! Split from [`super`] at the design-time budget on the seam the pane now
//! has: [`super`] is the list and what a section says, `wall` is one
//! workspace, and this is the engine itself — the row an operator clicks to
//! open one, and the order the rows stand in.
//!
//! **The cursor here is not the selection, and it is the one place in this
//! window where that is true** (§4.39). Landing on an engine's row only stands
//! there: opening is an act, and a walk that opened every engine it moved
//! through would be a walk nobody could use to reach the one below. So the row
//! carries the brand rule while the cursor is on it, and
//! [`crate::ui::keys`]'s Enter calls [`Model::open_engine`] — the same door
//! this file's click calls, which is what keeps it from being a second
//! surface.

use crate::ui::{Aim, Chunk, Model, theme};

/// **One stop on the roster's cursor track**: an engine's own row, or a wall
/// under the open one.
///
/// Two kinds because landing on them means two different things (§4.39):
/// landing on a wall aims it, exactly as a click does, and landing on an
/// engine row only stands there — the walk must not open every engine it
/// passes through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// An engine's row, by the name this box calls it.
    Engine(String),
    /// A wall under the open engine, by what a gesture must carry.
    Wall(Aim),
}

impl Step {
    /// **Whether the roster's cursor is on this stop right now.**
    pub fn holding(&self, model: &Model) -> bool {
        match self {
            Self::Engine(name) => model.standing.as_deref() == Some(name.as_str()),
            Self::Wall(aim) => model.standing.is_none() && model.aim.as_ref() == Some(aim),
        }
    }
}

/// **Every stop the roster's walk makes, in the order the pane paints them.**
///
/// It is the keyboard's cursor track and it is a **query**, derived from the
/// same rows and the same order [`super::render`] draws — so a key cannot walk
/// onto a row a click cannot reach, and cannot walk in an order the glass does
/// not show. A closed engine offers its own row and nothing under it, and a
/// wall this seat holds no name for is on neither surface: no envelope can
/// address it.
pub fn track(model: &Model) -> Vec<Step> {
    let open = model.engine_open();
    let mut rows = Vec::new();
    for chunk in model.engine_rows() {
        let name = chunk.channel.name.clone();
        rows.push(Step::Engine(name.clone()));
        if open.as_deref() != Some(name.as_str()) {
            continue;
        }
        for wall in super::ordered(&chunk.walls) {
            if let Some(address) = chunk.channel.address(&wall) {
                rows.push(Step::Wall(Aim {
                    channel: name.clone(),
                    address,
                }));
            }
        }
    }
    rows
}

/// **Paint one engine's row and take a click on it.**
///
/// The open one is the chosen row, which is what an accordion has instead of a
/// disclosure triangle: the raise says which one you are inside. The standing
/// one carries the brand rule, and takes the keyboard so the ring is on it and
/// Enter fires it.
pub fn render(ui: &mut egui::Ui, model: &mut Model, chunk: &Chunk, open: bool, reveal: bool) {
    let name = chunk.channel.name.clone();
    let standing = model.standing.as_deref() == Some(name.as_str());
    let seat = theme::paint::row(
        ui,
        &super::header(&chunk.channel),
        theme::INK,
        standing.then_some(theme::BRAND),
        open,
        0.0,
    );
    if standing && reveal {
        seat.scroll_to_me(None);
    }
    // **Tab and the arrows agree** (bl-2d6b), one row kind over: a Tab that
    // lands on an engine's row hands the arrows to the roster AND stands the
    // cursor there, so the next arrow steps off the row the ring is on.
    if seat.gained_focus() {
        model.focus = crate::ui::keys::Pane::Roster;
        model.standing = Some(name.clone());
    }
    if seat.clicked() {
        model.open_engine(&name);
    }
}
