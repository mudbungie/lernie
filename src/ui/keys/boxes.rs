//! **Every box on the glass that takes text**, by the id it wears — and the
//! gate that asks whether one of them holds the keyboard right now.
//!
//! Split from [`super`] at the 300-line cap on the seam the module doc already
//! draws: [`super`] is what a key DOES, and this is the registry of what
//! stands the keys down. The two grow for different reasons — a binding moves
//! that file, and a pane that lands a text box moves this one.
//!
//! **The gate asks for these boxes BY NAME** rather than for egui's
//! `wants_keyboard_input`, which answers *is anything focused at all* — every
//! button included. Tabbing to Send would otherwise turn the arrows off, and a
//! click focuses a control too, so the honest question is the narrow one: every
//! box that takes text wears an id, [`BOXES`] is the whole list of them, and
//! the gate is a comparison against it.

/// **The id the composer's box wears**, and the whole of what the keyboard has
/// to know about it. The deposit's box and the start's are one control with two
/// subjects and are never painted together, so they wear one id — and the gate
/// below is a comparison rather than a guess about what "focused" means.
pub const BOX_ID: &str = "the composer's box";

/// **The id the config editor's box wears** (`crate::ui::config::edit`).
pub const CONFIG_ID: &str = "the config editor's box";

/// **The id the workflow name box wears** (`crate::ui::config`).
pub const WORKFLOW_ID: &str = "the workflow name box";

/// **The id the fan's goal box wears** (`crate::ui::fleet::candidates`).
pub const FAN_GOAL_ID: &str = "the fan's goal box";

/// **The id the delivery subject's box wears** (`crate::ui::fleet::candidates`).
pub const SUMMARY_ID: &str = "the delivery subject's box";

/// **The five the ball pane's authoring block wears** (bl-f7ae,
/// `crate::ui::board::acts`) — the project a new ball is filed in, its title
/// and body, the journal note an amendment appends, and the id typed back that
/// arms a delivery.
///
/// Five ids for one block because the gate is a comparison against a FOCUSED
/// id: two boxes sharing one would be one box as far as the keyboard is
/// concerned, and an arrow taken from inside the second would walk the roster
/// under a half-typed title.
pub const PROJECT_ID: &str = "the new ball's project box";
pub const TITLE_ID: &str = "the ball's title box";
pub const BODY_ID: &str = "the ball's body box";
pub const NOTE_ID: &str = "the ball's journal box";
pub const DELIVER_ID: &str = "the delivery's arming box";

/// **The id a staged proposal's arming box wears** (`crate::ui::config::proposals`).
pub const VERDICT_ID: &str = "the proposal's arming box";

/// **Every box on the glass that takes text, so the gate can name them all**
/// (bl-dbc9).
///
/// It was one id, and one was enough while the only way into the other two was
/// Tab — a hazard, but one an operator walked into deliberately. A conversation
/// row's menu now LANDS the cursor in the reason box and in the arming box
/// (`crate::ui::model::fill`), and an arrow taken from inside either would have
/// walked the conversation list under a half-typed reason and flagged the row
/// it landed on. The gate is still a comparison rather than
/// `wants_keyboard_input` — which answers *is anything focused*, buttons
/// included — and this is the whole list of what it compares against. A fourth
/// box belongs here in the commit that paints it.
pub const BOXES: [&str; 11] = [
    BOX_ID,
    CONFIG_ID,
    WORKFLOW_ID,
    FAN_GOAL_ID,
    SUMMARY_ID,
    PROJECT_ID,
    TITLE_ID,
    BODY_ID,
    NOTE_ID,
    DELIVER_ID,
    VERDICT_ID,
];

/// Whether a box that takes text holds the keyboard right now.
pub fn typing(ctx: &egui::Context) -> bool {
    let focused = ctx.memory(egui::Memory::focused);
    BOXES
        .iter()
        .any(|name| focused == Some(egui::Id::new(*name)))
}

/// **The caret lives in the prompt line** (bl-b1a1; DESIGN §4.42, keyboard
/// whole): whenever nothing else holds the keyboard and the window is at
/// rest, the box takes it back — at launch, after the switcher picks, after
/// Esc leaves the box, after an overlay closes. Typing is the way in, so a
/// keystroke must land somewhere without a click first.
///
/// **At rest** is the prompt standing with nothing over it and nothing
/// asking: an overlay or the switcher unpaints the box, and while the ask
/// stands its digits and arrows are the keys' (`super::asking` — *while
/// nothing holds the keyboard*), so the box waits to be clicked into there.
/// A control the operator tabbed to holds the keyboard, so nothing is taken
/// from it.
pub fn rest(ctx: &egui::Context, model: &crate::ui::Model) {
    let resting =
        !model.covered() && model.pending().is_none() && crate::ui::ask::row(model).is_none();
    if resting && ctx.memory(egui::Memory::focused).is_none() {
        ctx.memory_mut(|memory| memory.request_focus(egui::Id::new(BOX_ID)));
    }
}
