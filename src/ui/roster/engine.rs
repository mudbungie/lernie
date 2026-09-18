//! **One engine's row**, the control that begins a conversation on it, and the
//! cursor track the whole list makes (DESIGN §4.39).
//!
//! Split from [`super`] at the design-time budget on the seam the pane now
//! has: [`super`] is the list and what a section says, `wall` is one
//! workspace, and this is the engine itself — the row an operator clicks to
//! open one, the `+` at its right edge, and the order the rows stand in.
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

/// **The word on the control that begins a conversation on this engine**
/// (§4.39) — one glyph, at the right edge of every engine's row.
///
/// A glyph with no word beside it, which is the one compact control in this
/// window that has none (`docs/STYLE.md` §2). What a word would cost is the
/// row: the engine's own name, its host and its dial state are what the row is
/// for, and a control captioned *begin* on every one of them would take a
/// third of the list's width away from the thing being listed. `+` is the one
/// glyph a person reads as *another one of these* without being taught.
pub const BEGIN: &str = "+";

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
    for chunk in model.engine_rows() {
        let name = chunk.channel.name.clone();
        rows.push(Step::Engine(name.clone()));
        if open.as_deref() != Some(name.as_str()) {
            continue;
        }
        for wall in super::ordered(&chunk.walls) {
            let Some(address) = chunk.channel.address(&wall) else {
                continue;
            };
            let aim = Aim {
                channel: name.clone(),
                address,
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

/// **Paint one engine's row with its `+`, and take a click on either.**
///
/// The open one is the chosen row, which is what an accordion has instead of a
/// disclosure triangle: the raise says which one you are inside. The standing
/// one carries the brand rule, and takes the keyboard so the ring is on it and
/// Enter fires it.
///
/// **The row is handed a width with the `+` already taken out of it**, rather
/// than painted full width with the control on top: two widgets over one
/// rectangle is a click that fires both, and `theme::paint::row` fills
/// whatever width it is given. The control is a square one row tall, so what
/// it costs the name is a constant and the Tab order runs left to right the
/// way the row reads.
///
/// **The `+` stands down while a pane covers the conversation**, which is the
/// per-wall controls' own rule one row kind over (`super::wall`): what it does
/// is put the caret in the composer's box, the composer stands down under
/// every covering pane, so a `+` offered there would drop the selection and
/// ask for a caret in a box that is not on the glass.
pub fn render(ui: &mut egui::Ui, model: &mut Model, chunk: &Chunk, open: bool, reveal: bool) {
    let name = chunk.channel.name.clone();
    let standing = model.standing.as_deref() == Some(name.as_str());
    let words = super::header(&chunk.channel);
    let (seat, begun) = ui
        .horizontal(|ui| {
            let begins = !model.covered();
            let taken = if begins {
                theme::ROW + ui.spacing().item_spacing.x
            } else {
                0.0
            };
            let width = (ui.available_width() - taken).max(0.0);
            let seat = ui
                .allocate_ui(egui::vec2(width, theme::ROW), |ui| {
                    theme::paint::row(
                        ui,
                        &words,
                        theme::INK,
                        standing.then_some(theme::BRAND),
                        open,
                        0.0,
                    )
                })
                .inner;
            let begun = begins
                && ui
                    .add_sized(
                        [theme::ROW, theme::ROW],
                        egui::Button::new(BEGIN).min_size(egui::Vec2::ZERO),
                    )
                    .clicked();
            (seat, begun)
        })
        .inner;
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
    // **The `+` is read last**, so a frame in which both fired — which no
    // pointer can produce and a Tab-and-Enter can — ends where the operator
    // aimed: beginning a conversation is the more specific of the two acts and
    // opens the engine on its way (`crate::ui::model::engines`).
    if begun {
        model.begin_on(&name);
    }
}
