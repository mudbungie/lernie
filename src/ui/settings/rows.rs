//! **A row, and the knob pages made of them** (DESIGN §4.48 item 7): a
//! label, its help sentence beneath in weak ink, the control at the right end
//! — the baseline's shape, and the one every page lays its rows in where its
//! control is one thing wide.
//!
//! **The table is the page**: a knob added to `crate::settings::KNOBS` is a
//! row of the page its `page` field names, painted as the kind its row names
//! — a number is a slider, a toggle a check — with no edit here. A change
//! lands in `Model::settings` on the frame it is made and every reader reads
//! it on the next (§4.46 item 4); nothing on these pages crosses the wire.
//! The pin is the bar page's last row, painted on the same fact the bar's
//! head toggles (`Place::pinned`), because one fact with two doors is still
//! one fact.

use crate::settings::{KNOBS, Kind, Knob, Knobbed, Settings};
use crate::ui::{Model, theme};

/// The pin's row: the same fact as the bar's head.
pub const PINNED: &str = "bar pinned open";
/// What the pin's row does.
pub const PINNED_HELP: &str = "keeps the bar open whatever the pointer does — the pin at its head";
/// **How wide a slider's row stands**: the toolkit's trough and the number
/// beside it, with its unit.
pub const SLIDER: f32 = 200.0;
/// The link at the foot that puts every knob back, and what it does.
pub const DEFAULTS: &str = "defaults";
pub const DEFAULTS_HELP: &str =
    "puts every knob on the seat's three pages back to its default; the pin stays";

/// **The width under which a row stacks**: twice a slider's span, so the
/// words keep at least a slider's width beside the control. Under it — the
/// matrix size with the bar open — the control stands under the sentence
/// instead of beside it, and nothing is laid past the box (§4.32).
pub const STACK: f32 = 2.0 * SLIDER;

/// **One row**: the label over its sentence at the left, `control` at the
/// right end — or under the sentence where the row is narrower than
/// [`STACK`]. Answers the row's seat, for a search hit to scroll to.
pub fn row(
    ui: &mut egui::Ui,
    label: &str,
    help: &str,
    control: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    let words = |ui: &mut egui::Ui| {
        ui.label(egui::RichText::new(label).color(theme::INK));
        ui.add(egui::Label::new(egui::RichText::new(help).color(theme::INK_WEAK)).wrap());
    };
    let laid = if ui.available_width() >= STACK {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            control(ui);
            ui.add_space(theme::space::S);
            ui.vertical(words);
        })
    } else {
        ui.vertical(|ui| {
            words(ui);
            control(ui);
        })
    };
    ui.add_space(theme::space::M);
    laid.response
}

/// **One knob page**: its knobs' rows, the pin on the bar's, and the
/// defaults link at the foot.
pub fn page(ui: &mut egui::Ui, model: &mut Model, seat: crate::settings::Page) {
    for knob in KNOBS.iter().filter(|knob| knob.page == seat) {
        self::knob(ui, &mut model.settings, &mut model.seeking.jump, knob);
    }
    if seat == crate::settings::Page::Bar {
        row(ui, PINNED, PINNED_HELP, |ui| {
            theme::paint::toggle(ui, &mut model.bar.pinned, "", PINNED_HELP);
        });
    }
    if theme::paint::link(ui, DEFAULTS, theme::INK_WEAK, DEFAULTS_HELP).clicked() {
        model.settings = Settings::default();
    }
}

/// A knob's control as its row paints it: a check, a slider with its unit
/// and bounds, or a seat per word of a choice.
enum Control {
    On(bool),
    At(f32, &'static str, (f32, f32, f32)),
    Word(&'static str, &'static [(&'static str, &'static str)]),
}

/// **One knob's row**: its control by its kind, writing a copy back through
/// the row's own setter so the typed field and the table cannot disagree
/// about what a number means. The row a search hit named scrolls into view
/// on the frame it is painted, and the hit is spent.
///
/// **A row whose kind and value disagree is a table fault**, judged by
/// `crate::settings::tests` before any page paints it; nothing is painted for
/// one rather than a control that lies.
pub fn knob(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    jump: &mut Option<&'static str>,
    knob: &Knob,
) {
    let control = match (knob.kind, (knob.get)(settings)) {
        (Kind::Toggle, Knobbed::On(on)) => Control::On(on),
        (
            Kind::Number {
                unit,
                floor,
                ceiling,
                step,
            },
            Knobbed::At(at),
        ) => Control::At(at, unit, (floor, ceiling, step)),
        (Kind::Choice { words }, Knobbed::Word(at)) => Control::Word(at, words),
        _ => return,
    };
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
    let mut written = None;
    let seat = row(ui, knob.label, &help, |ui| match control {
        Control::On(mut on) => {
            if theme::paint::toggle(ui, &mut on, "", &help).changed() {
                written = Some(Knobbed::On(on));
            }
        }
        // **The slider is given its span**: it lays its trough and its number
        // in a row of its own, and a row laid in a right-to-left parent takes
        // the whole width and starts at the left — so it is handed the width
        // it needs, and stands at the right end like every other control.
        Control::At(mut at, unit, bounds) => {
            ui.allocate_ui_with_layout(
                egui::vec2(SLIDER, theme::ROW),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    if theme::paint::slider(ui, &mut at, bounds, unit, &help).changed() {
                        written = Some(Knobbed::At(at));
                    }
                },
            );
        }
        // **A choice is a seat per word, each over the sentence that says
        // what it is for** (§4.48 item 8: the baseline's two modes, told in
        // its words), the chosen one wearing the open fill.
        Control::Word(at, words) => {
            ui.vertical(|ui| {
                for (word, says) in words {
                    if theme::paint::seat(ui, at == *word, word, says).clicked() {
                        written = Some(Knobbed::Word(word));
                    }
                    ui.label(egui::RichText::new(*says).small().color(theme::INK_WEAK));
                }
            });
        }
    });
    if let Some(value) = written {
        (knob.set)(settings, value);
    }
    if *jump == Some(knob.key) {
        seat.scroll_to_me(Some(egui::Align::Min));
        *jump = None;
    }
}
