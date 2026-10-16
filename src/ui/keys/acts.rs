//! **What each fired row does** (DESIGN §4.46 item 3), split from
//! [`super::bindings`] at the 300-line cap on the seam the table draws:
//! that file is what the keyboard SAYS — the rows, their spellings and the
//! citations — and this is what a row DOES. Each arm names a door a control
//! already fires, so a chord is never a second way of doing a thing.

use super::bindings::Act;
use crate::settings::{Kind, Knobbed, SCALE, SCALE_KEY, Toggle};
use crate::ui::chat::scroll::{self, Scroll};
use crate::ui::{Model, dock, slash};

/// **Fire one act.** Each arm names a door a control already fires, so a
/// chord is never a second way of doing a thing.
pub fn fire(act: Act, ctx: &egui::Context, model: &mut Model) {
    match act {
        Act::Switch => model.switch(),
        Act::Model => {
            slash::fire(model, crate::ui::tuning::MODEL, "");
        }
        Act::Dock => dock::toggle(ctx, model.settings.on(Toggle::Dock)),
        Act::Settings => {
            slash::fire(model, crate::ui::settings::WORD, "");
        }
        Act::Help => {
            slash::fire(model, crate::ui::commands::WORD, "");
        }
        Act::Pin => model.bar.pinned = !model.bar.pinned,
        Act::New => new(model),
        Act::CloseTab => {
            if let Some(selected) = model.conversation.clone() {
                model.close_tab(&selected);
            }
        }
        Act::NextTab => tab(model, 1),
        Act::PrevTab => tab(model, -1),
        Act::PageDown => scrolled(ctx, model, Scroll::Page(1.0)),
        Act::PageUp => scrolled(ctx, model, Scroll::Page(-1.0)),
        Act::End => scrolled(ctx, model, Scroll::End),
        Act::Head => scrolled(ctx, model, Scroll::Head),
        Act::ScaleUp => scale(model, 1.0),
        Act::ScaleDown => scale(model, -1.0),
        Act::ScaleReset => scale(model, 0.0),
    }
}

/// **What `+ new` does** (`crate::ui::bar`): a conversation begun on the
/// aimed wall, under the same guard the door stands behind — nothing is
/// aimed, or a pane covers the window and it is not the switcher.
fn new(model: &mut Model) {
    if model.covered() && !model.switching {
        return;
    }
    if let Some(aim) = model.aim.clone() {
        model.begin_at(&aim);
        model.chose();
    }
}

/// **One tab along the strip**, saturating at its ends as the walk does
/// (`super::moved`); with none selected, the first.
fn tab(model: &mut Model, step: isize) {
    let strip = model.strip();
    let at = model
        .conversation
        .as_ref()
        .and_then(|selected| strip.iter().position(|tab| tab == selected));
    if let Some(next) = super::moved(strip.len(), at, step).and_then(|i| strip.get(i)) {
        model.select(next);
    }
}

/// **A scroll asked of the transcript**, while the transcript is what is on
/// the glass: a pane standing over it is what the hand is on.
fn scrolled(ctx: &egui::Context, model: &Model, scroll: Scroll) {
    if !model.covered() {
        scroll::request(ctx, scroll);
    }
}

/// **The `scale` knob moved `steps` of its own step**, within its bounds —
/// or put back to its default on no step at all. Through the knob's own
/// row, so the chord and the slider cannot disagree about what a step is.
fn scale(model: &mut Model, steps: f32) {
    let knobbed = crate::settings::knob(SCALE_KEY).map(|knob| (knob, (knob.get)(&model.settings)));
    if let Some((knob, Knobbed::At(at))) = knobbed
        && let Kind::Number {
            floor,
            ceiling,
            step,
            ..
        } = knob.kind
    {
        let to = if steps == 0.0 {
            SCALE
        } else {
            (at + steps * step).clamp(floor, ceiling)
        };
        (knob.set)(&mut model.settings, Knobbed::At(to));
    }
}
