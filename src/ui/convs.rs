//! **The conversation list** for the wall the window is aimed at.
//!
//! A row is a glance, not a transcript: what it is called, what it is doing,
//! how long since it moved, why its latest model call failed if it did, and
//! the first line of what was said. Everything deeper is one click away in the
//! chat pane, and a list that tried to be the pane would be neither.

use crate::reply::convs::ConvRow;
use crate::ui::{Model, theme};

/// What the list says with no wall aimed at.
pub const NO_WALL: &str = "pick a workspace";
/// What it says for a wall that answered, and answered nothing.
pub const NO_CONVERSATIONS: &str = "no conversations here";
/// **What it says for a wall it has not been ANSWERED about** (bl-f780).
///
/// The third sentence, and it is the [`UNCERTAIN`] doctrine one level up: *no
/// conversations here* is a definite fact about a wall nobody has looked at
/// yet, and the pane already refuses to state a definite fact about a
/// conversation nobody could take a reading of. It stands from the keypress
/// that aims until the answer lands — a round trip on a wire, not the
/// millisecond it is on loopback.
pub const NOT_ANSWERED: &str = "waiting to hear about this wall";

/// **What it says for an aim this seat cannot ask about at all**, which is
/// permanent rather than transient.
///
/// `crate::place` restores a saved aim without checking it, on the ground that
/// a stale one is inert — `crate::state::Standing::aimed` finds no channel by
/// that name and asks nothing. Inert is right about the dialling and wrong
/// about the paint: nothing is ever asked, so [`NOT_ANSWERED`] would stand
/// forever over a wall that has no channel to answer it. The refusal that was
/// silent is said here instead.
pub fn no_channel(channel: &str) -> String {
    format!(
        "this seat holds no channel named {channel:?}, so nothing is asked about this wall — pick one from the channels beside it"
    )
}

/// The word this pane wears, and the subject the arrows act on when it is
/// focused.
pub const HEADING: &str = "conversations";
/// The mark a state nothing observed wears, inside the badge it qualifies.
pub const UNCERTAIN: &str = "?";

/// Paint the list and take a click on it.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    ui.heading(crate::ui::keys::heading(
        HEADING,
        model.focus == crate::ui::Pane::Conversations,
    ));
    let Some(aim) = model.aim.clone() else {
        ui.label(NO_WALL);
        return;
    };
    ui.label(aim.address.clone());
    // **An aim on a channel this seat does not hold is asked about by nobody**,
    // so it gets its own sentence rather than one that implies an answer came
    // back. The roster carries every channel this box holds from boot, off the
    // disk and before anything is dialled, so this is a question about the
    // model and not about a socket.
    if !model.holds(&aim.channel) {
        ui.label(no_channel(&aim.channel));
        return;
    }
    // **The list is the model's, not this pane's**: a conversation this window
    // has started but the engine cannot resolve yet stands in it as a row of
    // its own (`crate::ui::model::claim`), and it stands there for the pointer
    // and the keyboard alike because both walk the one list.
    let rows = model.rows();
    if rows.is_empty() {
        ui.label(if model.answered.as_ref() == Some(&aim) {
            NO_CONVERSATIONS
        } else {
            NOT_ANSWERED
        });
        return;
    }
    // The list scrolls; the heading and the address above it do not (bl-e5d2,
    // and `crate::ui::roster` for why the heading stays out).
    let reveal = model.revealing(crate::ui::keys::Pane::Conversations);
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            for row in rows {
                conversation(ui, model, &row, reveal);
            }
        });
}

/// One row, indented to its depth under the conversation root.
fn conversation(ui: &mut egui::Ui, model: &mut Model, row: &ConvRow, reveal: bool) {
    let selected = model.conversation.as_ref() == Some(&row.root_id);
    ui.horizontal(|ui| {
        ui.add_space(indent(row.depth));
        // **The headline TRUNCATES, and that is a layout invariant rather than
        // a nicety** (bl-b3b2). `Ui::selectable_label` lays its text with
        // `TextWrapMode::Extend` — hard-coded, and doubly so inside a
        // horizontal layout — so a long conversation name made the pane's inner
        // `min_rect` wider than the pane. That is not merely an overflow: a
        // side panel paints a frame sized to its own `max_width` and then
        // reserves `inner_response.response.rect.max` from the layout, so the
        // two disagree exactly when the shell's yield policy caps the pane
        // below its content — and the strip between the painted frame and the
        // reserved edge is covered by NO panel, with the central panel
        // beginning after it. Nothing paints it, so what shows there is the
        // window surface's own clear: black, or the desktop on the frames the
        // alpha path lets through.
        // **The selection is drawn UNDER the run** rather than by the widget,
        // because the widget that drew it is the one that could not truncate.
        // Same ink egui's own selectable seat uses, so a selected row is
        // unchanged to look at.
        //
        // A RESERVED SLOT IS THE ONLY WAY TO GET IT THERE, and the first cut of
        // this painted the fill after the label instead (bl-dc07). A painter
        // appends to its layer, so "after" is "on top": the selected
        // conversation became a solid bar of selection ink with its own name
        // invisible underneath, which is the one row in the pane an operator is
        // looking at. Nothing in the suite could see it — the glyphs WERE
        // painted, so the paint walk read them back intact, and the defect
        // lived entirely in what was drawn over them. `crate::snapshot` is the
        // witness that caught it and the one that keeps it caught.
        let ground = ui.painter().add(egui::Shape::Noop);
        let seat = ui.add(
            egui::Label::new(headline(row))
                .truncate()
                .selectable(false)
                .sense(egui::Sense::click()),
        );
        if selected {
            ui.painter().set(
                ground,
                egui::Shape::rect_filled(
                    seat.rect.expand2(ui.spacing().button_padding),
                    ui.visuals().widgets.active.rounding,
                    ui.visuals().selection.bg_fill,
                ),
            );
        }
        if selected && reveal {
            seat.scroll_to_me(None);
        }
        if seat.clicked() {
            model.select(&row.root_id.clone());
        }
    });
    // **The hue's own words** (REMOTE §9.10). A conversation whose latest model
    // call failed paints red and, until this line, said nothing about why — so
    // a wall whose provider row holds no credential was a list of red rows an
    // operator opened one by one to learn the one thing all of them said.
    //
    // It goes ABOVE the preview because the preview is what was last said and
    // this is why nothing more was: the row reads top-down as label, reason,
    // last words. Both lines are painted in the row's own tone and neither is
    // derived from the other — the engine states the hue and the clause
    // separately, and a seat that inferred one would be holding a second
    // opinion about a reading it did not take.
    if let Some(failure) = &row.failure {
        beneath(ui, row, failure);
    }
    if !row.preview.is_empty() {
        beneath(ui, row, &row.preview);
    }
}

/// A line hung under a row's headline, at the row's own indent and in its own
/// ink. One function rather than two blocks that must not drift: the second
/// line of a row is a shape, and a second copy of it is a second shape.
fn beneath(ui: &mut egui::Ui, row: &ConvRow, said: &str) {
    ui.horizontal(|ui| {
        ui.add_space(indent(row.depth) + 12.0);
        ui.colored_label(theme::tone_ink(&row.tone), said);
    });
}

/// How far a row hangs under its root, in points.
///
/// Added rather than multiplied, over a **bounded** count: there is no cast
/// from the wire's own width to a screen coordinate, so there is no truncation
/// to suppress a lint about. The cap is not a special case either — past it a
/// list is unreadable whatever the indent says, and the label is what the extra
/// width would have cost.
fn indent(depth: u64) -> f32 {
    const STEP: f32 = 16.0;
    const DEEPEST: u64 = 8;
    (0..depth.min(DEEPEST)).fold(0.0, |at, _| at + STEP)
}

/// A row's headline: its label, what it is doing, how long since it moved, and
/// what is waiting under it.
///
/// **The badge wears a `?` for a state nothing observed.** The engine answers a
/// reading and whether it could take one, and painting the first without the
/// second would state a definite fact about a conversation nobody looked at —
/// including this window's own, between a start's receipt and its driver's
/// first write.
pub fn headline(row: &ConvRow) -> String {
    let mut said = vec![format!(
        "{}  [{}{}]  {}",
        row.display,
        row.state.label(),
        if row.uncertain { UNCERTAIN } else { "" },
        age(row.age_secs)
    )];
    if row.attention > 0 {
        said.push(format!("{} waiting", row.attention));
    }
    if row.members > 1 {
        said.push(format!("{} members", row.members));
    }
    said.join("  ")
}

/// A compact age: `42s`, `7m`, `3h`, `2d`. **Negative clamps to zero** — two
/// machines' clocks disagreeing is a fact about a seat that dials somewhere
/// else, and an age in the future is not a thing to paint.
pub fn age(secs: i64) -> String {
    let secs = secs.max(0);
    for (bound, unit, per) in [(60, 's', 1), (3600, 'm', 60), (86_400, 'h', 3600)] {
        if secs < bound {
            return format!("{}{unit}", secs / per);
        }
    }
    format!("{}d", secs / 86_400)
}

#[cfg(test)]
mod tests;
