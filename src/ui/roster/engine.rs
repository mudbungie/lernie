//! **One engine's row**, the control that begins a conversation on it, and the
//! cursor track the whole list makes (DESIGN §4.39).
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

use crate::ui::model::Engine;
use crate::ui::{Aim, Model, theme};

/// **One stop on the window's cursor track**: an engine's own row, a wall
/// under the open one, or a conversation under the aimed wall.
///
/// Three kinds because landing on them means three different things (§4.39):
/// landing on a wall aims it and landing on a conversation selects it, exactly
/// as a click does, and landing on an engine row only stands there — the walk
/// must not open every engine it passes through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    /// An engine's row, by the name this box calls it.
    Engine(String),
    /// A wall under the open engine, by what a gesture must carry.
    Wall(Aim),
    /// A conversation under the aimed wall, by its root id.
    Conversation(String),
}

impl Step {
    /// **Whether the window's cursor is on this stop right now.**
    ///
    /// The three are mutually exclusive by construction rather than by a rule
    /// anybody keeps: standing on an engine clears the aim's claim to the
    /// cursor, and aiming clears the selection (`crate::ui::model::acts`), so
    /// at most one row answers yes.
    pub fn holding(&self, model: &Model) -> bool {
        match self {
            Self::Engine(name) => model.standing.as_deref() == Some(name.as_str()),
            Self::Wall(aim) => {
                model.standing.is_none()
                    && model.conversation.is_none()
                    && model.aim.as_ref() == Some(aim)
            }
            Self::Conversation(root_id) => {
                model.standing.is_none() && model.conversation.as_deref() == Some(root_id.as_str())
            }
        }
    }
}

/// **Every stop the walk makes, in the order the pane paints them.**
///
/// It is the keyboard's cursor track and it is a **query**, derived from the
/// same rows and the same order [`super::render`] draws — so a key cannot walk
/// onto a row a click cannot reach, and cannot walk in an order the glass does
/// not show. A closed engine offers its own row and nothing under it; a wall
/// this seat holds no name for is on neither surface, because no envelope can
/// address it; and a wall that is not the aimed one offers no conversations,
/// because this seat has asked about none (§4.12) and the glass paints none.
pub fn track(model: &Model) -> Vec<Step> {
    let open = model.engine_open();
    let mut rows = Vec::new();
    for engine in model.engine_rows() {
        let name = engine.name().to_owned();
        rows.push(Step::Engine(name.clone()));
        if open.as_deref() != Some(name.as_str()) {
            continue;
        }
        for wall in super::ordered(&engine.walls()) {
            let Some(aim) = engine.aim(&wall) else {
                continue;
            };
            let aimed = model.aim.as_ref() == Some(&aim);
            rows.push(Step::Wall(aim));
            if aimed {
                rows.extend(
                    model
                        .rows()
                        .into_iter()
                        .map(|row| Step::Conversation(row.root_id)),
                );
            }
        }
    }
    rows
}

/// **Paint one engine's row, and take a click on it.**
///
/// The open one is the chosen row, which is what an accordion has instead of a
/// disclosure triangle: the raise says which one you are inside. The standing
/// one carries the brand rule, and takes the keyboard so the ring is on it and
/// Enter fires it.
///
/// **It carries no `+` any more** (§4.42 item 7): the start is `+ new` under
/// each wall (`super::wall::NEW`), the row the conversation would hang from,
/// rather than a glyph at the far edge of a different row. An engine with no
/// wall this seat can address has nothing to begin on, which the composer
/// already said — a start is *a wall and no conversation*.
pub fn render(ui: &mut egui::Ui, model: &mut Model, engine: &Engine, open: bool, reveal: bool) {
    let name = engine.name().to_owned();
    let standing = model.standing.as_deref() == Some(name.as_str());
    let seat = theme::paint::row(
        ui,
        &engine.header(),
        theme::INK,
        standing.then_some(theme::BRAND),
        open,
        0.0,
    );
    if standing && reveal {
        seat.scroll_to_me(None);
    }
    // **Tab and the arrows agree** (bl-2d6b), one row kind over: a Tab that
    // lands on an engine's row stands the cursor there, so the next arrow
    // steps off the row the ring is on.
    if seat.gained_focus() {
        model.standing = Some(name.clone());
    }
    if seat.clicked() {
        model.open_engine(&name);
    }
}

/// **The row that founds a workspace** (§4.42 item 7; bl-673e), standing last
/// under this box's own engine beside the walls' `+ new` rows.
pub const FOUND: &str = "+ new workspace";

/// **`+ new workspace`**: the box asks for `<name> <goal>`
/// ([`Model::begin_founding`]), and the switcher closes on the way.
///
/// **Under this box's own engine and no other**: a gesture naming a
/// workspace no entry holds goes down the flat root (REMOTE §8.2), so a name
/// typed under an entry's heading would found it on a different engine than
/// the one it was offered under. It stands down where that channel will not
/// dial and under a covering pane, `+ new`'s own rules.
pub fn found(ui: &mut egui::Ui, model: &mut Model, engine: &Engine) {
    // The box's own engine leads any fold it is in (`crate::ui::model::fold`).
    let chunk = &engine.lead;
    if chunk.channel.name != crate::seat::OWN
        || matches!(chunk.held, crate::ui::Held::Unheld(_))
        || (model.covered() && !model.switching)
    {
        return;
    }
    let seat = theme::paint::row(ui, FOUND, theme::INK_WEAK, None, false, 0.0);
    if seat.clicked() {
        model.begin_founding(&chunk.channel.name);
        model.chose();
    }
}
