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

/// **The id the flag's reason box wears** (`crate::ui::composer::acts::WHY`).
pub const REASON_ID: &str = "the flag's reason box";

/// **The id the deletion's arming box wears**
/// (`crate::ui::composer::acts::ARM`).
pub const ARM_ID: &str = "the deletion's arming box";

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
pub const BOXES: [&str; 12] = [
    BOX_ID,
    REASON_ID,
    ARM_ID,
    CONFIG_ID,
    WORKFLOW_ID,
    FAN_GOAL_ID,
    SUMMARY_ID,
    PROJECT_ID,
    TITLE_ID,
    BODY_ID,
    NOTE_ID,
    DELIVER_ID,
];

/// Whether a box that takes text holds the keyboard right now.
pub fn typing(ctx: &egui::Context) -> bool {
    let focused = ctx.memory(egui::Memory::focused);
    BOXES
        .iter()
        .any(|name| focused == Some(egui::Id::new(*name)))
}
