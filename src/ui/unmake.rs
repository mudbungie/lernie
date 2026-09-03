//! **The unmaking pane**: the wall this would unmake, the name that arms it,
//! and the two ways out (DESIGN §4.20; yog's `docs/PARITY.md` §7).
//!
//! # This is the seat's destructive-act idiom, and the pane IS half of it
//!
//! Every other control in this window adds or changes something and is undone
//! by doing the other thing: aiming, selecting, depositing, nudging, starting,
//! enrolling, and the four tuning gestures. This is the first that destroys,
//! and it is a **place** rather than a control among them. Three things follow
//! and DESIGN §4.20 states all three; what is here is why they land in this
//! file.
//!
//! - **A covering pane is the only placement that is the same in both layout
//!   shapes** (`crate::ui::shell::policy`). The composer stands down off the
//!   conversation's own column in the narrow shape, and a list row is aimed by
//!   a click with the next row one pixel away — neither is where an unmaking
//!   belongs.
//! - **It states its subject before it offers the act**, because what is being
//!   unmade is the one fact a confirmation is about, and the pane is the only
//!   surface with room to say it.
//! - **The way out comes first**, in the layout and therefore in the tab order.
//!   The control an operator reaches for by reflex must be the one that changes
//!   nothing.
//!
//! # The arming is an enablement here, and a parameter next door
//!
//! `delete-workspace` is refused unless the typed name matches the workspace
//! exactly, so [`CONFIRM`] is disabled until it does —  **disabled and not
//! absent**, which is the tuning pane's `set`: the parameter is missing, not
//! the subject, so the control stays on the glass saying what would fill it.
//! `delete-agent`'s box next door is the other case and stays the other case
//! (`crate::ui::composer::acts`): there an empty box is a gesture somebody
//! meant, so its control is never disabled. The seat reads which of the two a
//! `typed` is off the wire's own grammar and invents no policy of its own.
//!
//! And the arming is **not spent on firing**, for the reason the composer's is
//! not: a refusal is the common case for this act — the engine declines while
//! anything in the workspace is live — and clearing the box would charge a
//! retype for the engine's *no*.

use crate::ui::{Model, theme};

/// The word that opens it, on the wall the window is aimed at.
pub const OPEN: &str = "unmake this workspace…";
/// The word that closes it, having unmade nothing. It says what it leaves
/// rather than what it abandons: `cancel` on a destructive pane names the
/// destruction as the thing in progress, and nothing is in progress.
pub const CLOSE: &str = "keep it";
/// The word that spends it.
pub const CONFIRM: &str = "unmake it";
/// The pane's own heading.
pub const HEADING: &str = "unmake this workspace";
/// What the arming box asks for.
pub const ARM: &str = "the workspace's own name";
/// What it says while the box does not hold that name. The refusal is stated
/// here rather than left to a greyed control: a disabled button says a control
/// is not live and nothing about what would make it live.
pub const NOT_ARMED: &str = "not armed — type the name above, exactly";
/// What it says once it does.
pub const ARMED: &str = "armed";
/// What it says once the act has been asked for.
pub const ASKED: &str = "asked — waiting for the engine";
/// What the engine will refuse it for, said before it is asked rather than
/// after: both conditions are facts an operator can check and neither is
/// visible on this pane.
pub const REFUSED_IF: &str =
    "refused unless this engine owns the workspace and nothing in it is live";

/// **What the arming box is worth**, in points. Fixed rather than infinite for
/// the reason the composer's is: this pane stands in the central panel, which
/// is what the two side panels leave, and a box that took the rest of the line
/// would push the control off the row at the widths the window actually opens
/// at.
const ARM_WIDTH: f32 = 200.0;

/// Paint the pane and take the clicks on it. Answers whether there was one to
/// paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    // **The subject, in the roster's own two facts**: the address a gesture
    // carries and the channel it goes down. Read off the PANE rather than off
    // `Model::aim`, so what is named here is what would be unmade even after
    // the roster underneath has been re-aimed.
    let Some(aim) = model.unmaking.as_ref().map(|held| held.aim.clone()) else {
        return false;
    };
    ui.heading(HEADING);
    ui.label(format!("{} on {}", aim.address, aim.channel));
    ui.colored_label(theme::NOTICE, REFUSED_IF);
    ui.separator();
    // **Every row here WRAPS**, because this pane is what the two side panels
    // leave (bl-dc07): at a 400-point window that is about 120 points, and an
    // unwrapped row lays its controls on one line however long the line has to
    // be — which on this pane would put the way out off the right edge.
    //
    // The box is typed into IN PLACE and the arming read back after, in this
    // frame: a copy taken before the edit would leave the control one frame
    // behind the keystroke that armed it.
    let mut armed = false;
    if let Some(held) = model.unmaking.as_mut() {
        ui.horizontal_wrapped(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut held.typed)
                    .desired_width(ARM_WIDTH)
                    .hint_text(ARM),
            );
        });
        armed = held.armed();
    }
    ui.horizontal_wrapped(|ui| {
        // **The way out is first**, and therefore first in the tab order too.
        if ui.button(CLOSE).clicked() {
            model.close_unmaking();
        }
        let spend = ui.add_enabled(armed, egui::Button::new(CONFIRM));
        crate::ui::act::tag(&spend, &[crate::verbs::DELETE_WORKSPACE.word]);
        if spend.clicked() {
            model.post_unmaking();
        }
    });
    if let Some(held) = model.unmaking.as_ref() {
        ui.colored_label(theme::NOTICE, said(held));
    }
    true
}

/// **What the pane says about its own arming**, as a pure function of the state
/// — so the three sentences are read back as a value rather than looked for on
/// a screen.
pub fn said(unmaking: &crate::ui::Unmaking) -> String {
    if unmaking.posted {
        ASKED.to_owned()
    } else if unmaking.armed() {
        ARMED.to_owned()
    } else {
        NOT_ARMED.to_owned()
    }
}

#[cfg(test)]
mod tests;
