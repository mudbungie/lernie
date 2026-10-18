//! **The settings pane** (DESIGN §4.46 item 4): the knob table painted, one
//! row per knob, and nothing decided here.
//!
//! Every row is a fact about THIS seat — yog's `docs/REMOTE.md` §7's
//! per-seat UI state — so the pane posts nothing and has no standing read:
//! a change lands in `Model::settings` on the frame it is made, every reader
//! reads it on the next, and the place file writes it at exit
//! (`crate::place`). The tuning pane (§4.17) is the workspace's settings and this
//! is the box's, and the two never meet.
//!
//! **The table is the pane**: a knob added to `crate::settings::KNOBS` is a
//! row here with no edit, painted as the kind its row names — a number is a
//! slider, a toggle a check — over its help sentence in weak ink, which is
//! also what it says under the pointer. The pin is the thirteenth row,
//! painted on the same fact the bar's head toggles (`Place::pinned`), because
//! one fact with two doors is still one fact.

use crate::settings::{KNOBS, Kind, Knob, Knobbed, Settings};
use crate::ui::{Lookup, Model, theme};

/// **The word typed after the slash** that opens it (`crate::ui::slash`),
/// and the chord Ctrl+, fires (`crate::ui::keys`).
pub const WORD: &str = "settings";
/// What the menu says beside the word.
pub const SUMMARY: &str = "this seat's own knobs — the wheel, the bar, the help, the look — Ctrl+,";
/// The pane's own heading.
pub const HEADING: &str = "this seat's settings";
/// The pin's row: the same fact as the bar's head.
pub const PINNED: &str = "bar pinned open";
/// What the pin's row does.
pub const PINNED_HELP: &str = "keeps the bar open whatever the pointer does — the pin at its head";
/// The link at the foot that puts every knob back, and what it does.
pub const DEFAULTS: &str = "defaults";
pub const DEFAULTS_HELP: &str = "puts every knob above back to its default; the pin stays";

/// Paint the pane and take what changed on it. Answers whether there was one
/// to paint, so the shell knows whether the conversation still stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    if model.lookup != Some(Lookup::Settings) {
        return false;
    }
    crate::ui::overlay::name(ui, HEADING);
    egui::ScrollArea::vertical()
        .id_salt(HEADING)
        .auto_shrink(false)
        .show(ui, |ui| {
            for knob in KNOBS {
                row(ui, &mut model.settings, knob);
            }
            theme::paint::toggle(ui, &mut model.bar.pinned, PINNED, PINNED_HELP);
            sentence(ui, PINNED_HELP);
            ui.add_space(theme::space::S);
            if theme::paint::link(ui, DEFAULTS, theme::INK_WEAK, DEFAULTS_HELP).clicked() {
                model.settings = Settings::default();
            }
        });
    true
}

/// **One knob's row**: its control by its kind, then its sentence. A slider
/// edits a copy and writes it back through the row's own setter, so the
/// typed field and the table cannot disagree about what a number means.
fn row(ui: &mut egui::Ui, settings: &mut Settings, knob: &Knob) {
    match (knob.kind, (knob.get)(settings)) {
        (Kind::Toggle, Knobbed::On(mut on)) => {
            let seat = theme::paint::toggle(ui, &mut on, knob.label, knob.help);
            written(settings, knob, seat.changed(), Knobbed::On(on));
        }
        (
            Kind::Number {
                unit,
                floor,
                ceiling,
                step,
            },
            Knobbed::At(mut at),
        ) => {
            ui.horizontal(|ui| {
                ui.label(knob.label);
                // **The scale knob cites its chords** (§4.46 item 3).
                let help = if knob.key == crate::settings::SCALE_KEY {
                    crate::ui::keys::cited(
                        knob.help,
                        &[
                            crate::ui::keys::Act::ScaleUp,
                            crate::ui::keys::Act::ScaleDown,
                            crate::ui::keys::Act::ScaleReset,
                        ],
                    )
                } else {
                    knob.help.to_owned()
                };
                let seat = theme::paint::slider(ui, &mut at, (floor, ceiling, step), unit, &help);
                written(settings, knob, seat.changed(), Knobbed::At(at));
            });
        }
        // **A row whose kind and value disagree is a table fault**, judged
        // by `crate::settings::tests` before any pane paints it; nothing is
        // painted for one rather than a control that lies.
        _ => {}
    }
    sentence(ui, knob.help);
    ui.add_space(theme::space::XS);
}

/// **Write `value` through the row's own setter** where the control said it
/// changed, so the typed field and the table cannot disagree about what a
/// number means.
fn written(settings: &mut Settings, knob: &Knob, changed: bool, value: Knobbed) {
    if changed {
        (knob.set)(settings, value);
    }
}

/// The help sentence under a row, in weak ink, wrapped.
fn sentence(ui: &mut egui::Ui, help: &str) {
    ui.add(egui::Label::new(egui::RichText::new(help).color(theme::INK_WEAK)).wrap());
}

#[cfg(test)]
mod tests;
