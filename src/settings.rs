//! **The knob table** (DESIGN §4.46 item 4): every setting that is a fact
//! about this seat, as one row each — its key, its label, its help sentence,
//! its kind with the bounds a control paints, and the way to read and write
//! it on the typed [`Settings`] the readers read.
//!
//! **One table, three readers.** The pane paints it (`crate::ui::settings`),
//! the place file reads and writes it ([`from_json`], [`to_json`]) and the
//! tests judge it, so a knob added here is painted, remembered and judged
//! with no second edit. The readers — the bar, the chat, the theme's install,
//! the delete — read the typed fields and never the table.
//!
//! **Nothing here crosses the boundary.** yog's `docs/REMOTE.md` §7 rules
//! that per-seat UI state is the seat's own, which is what every row is: the
//! workspace's facts are the tuning pane's (§4.17) and the config pane's (§4.30).
//!
//! **Written only where set.** [`to_json`] writes a knob only when it stands
//! off its default, so a seat that touched nothing writes nothing and a
//! default moved in a later build reaches it — the rule the bar's dragged
//! widths kept before they were knobs (§4.45 item 2).

use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

/// **The defaults, each named once** — cited by the readers that used to
/// hold them (`crate::ui::bar`) so the number has one home.
/// Points scrolled per wheel notch: egui's own is 40 on native, under two
/// rows a notch (§4.46 item 1).
pub const WHEEL: f32 = 100.0;
/// Seconds the pointer rests on a control before it tells (§4.46 item 2).
pub const HELP_DELAY: f32 = 0.5;
/// The window's scale: egui's zoom factor.
pub const SCALE: f32 = 1.0;
/// **The scale knob's key**, named because the chords move it (§4.46 item 3).
pub const SCALE_KEY: &str = "scale";
/// Seconds the bar stays open after the pointer leaves (§4.45 item 1).
pub const LINGER: f32 = 0.75;
/// Seconds the bar's edge takes to travel between its widths (§4.44 item 1).
pub const EASE: f32 = 0.12;
/// The bar's collapsed width in points (§4.44 item 1, §4.45 item 2).
pub const BAR_SHUT: f32 = 80.0;
/// **The desk size's width** (`crate::snapshot::SIZES`), whose quarter is
/// the bar's open width by default.
pub const DESK: f32 = 1400.0;

/// **Everything this seat can be set to**, typed. `Default` is the table's
/// defaults, and the two widths are `None` until dragged or set: absent is
/// the policy's answer, clamped to the window on every frame
/// (`crate::ui::bar::drag::shown`), and never a number this file has to
/// refuse.
///
/// **The switches are a set**, not a field apiece: which of them are on is
/// the one fact, [`Settings::on`] is the query, and the default is the set
/// of the four that stand on. A struct of five independent bools is five
/// fields that cannot be iterated, which the table has to.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub wheel: f32,
    pub help_delay: f32,
    pub scale: f32,
    pub linger: f32,
    pub ease: f32,
    pub bar_shut: Option<f32>,
    pub bar_open: Option<f32>,
    /// The switches that are on.
    pub on: BTreeSet<Toggle>,
}

/// **The five switches.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Toggle {
    /// Whether a control tells under the pointer (§4.46 item 2).
    Help,
    /// The fills under the transcript's blocks (§4.45 item 7).
    Bubbles,
    /// Whether a working folds itself once the answer stands (§4.45 item 5).
    Fold,
    /// Whether a delete asks first (§4.44 item 1).
    Confirm,
    /// Whether the dock stands unfolded when the window opens (§4.42 item 9).
    Dock,
}

impl Settings {
    /// **Whether `switch` is on.**
    pub fn on(&self, switch: Toggle) -> bool {
        self.on.contains(&switch)
    }

    /// **Turn `switch` on or off.**
    pub fn set(&mut self, switch: Toggle, on: bool) {
        if on {
            self.on.insert(switch);
        } else {
            self.on.remove(&switch);
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            wheel: WHEEL,
            help_delay: HELP_DELAY,
            scale: SCALE,
            linger: LINGER,
            ease: EASE,
            bar_shut: None,
            bar_open: None,
            on: [Toggle::Help, Toggle::Bubbles, Toggle::Fold, Toggle::Confirm].into(),
        }
    }
}

/// What kind of control a knob is painted as.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// On or off: a check box.
    Toggle,
    /// A number between two bounds, moved by `step`, said with `unit`: a
    /// slider.
    Number {
        unit: &'static str,
        floor: f32,
        ceiling: f32,
        step: f32,
    },
}

/// A knob's value, read or written: the one shape both kinds share.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Knobbed {
    On(bool),
    At(f32),
}

/// **One row of the table.**
#[derive(Debug, Clone, Copy)]
pub struct Knob {
    /// The key it is written under, and the word a reader of the file sees.
    pub key: &'static str,
    /// The label on the pane.
    pub label: &'static str,
    /// **What it does**, in the operator's terms (§4.46 item 2).
    pub help: &'static str,
    pub kind: Kind,
    /// Read it off the settings.
    pub get: fn(&Settings) -> Knobbed,
    /// Write it; a value of the other kind writes nothing.
    pub set: fn(&mut Settings, Knobbed),
}

/// Seconds, between nothing and a few.
const fn seconds(ceiling: f32, step: f32) -> Kind {
    Kind::Number {
        unit: " s",
        floor: 0.0,
        ceiling,
        step,
    }
}

/// Points, between two bounds.
const fn points(floor: f32, ceiling: f32, step: f32) -> Kind {
    Kind::Number {
        unit: " pt",
        floor,
        ceiling,
        step,
    }
}

mod knobs;
pub use knobs::KNOBS;

/// **The knob `key` names**, if the table has it.
pub fn knob(key: &str) -> Option<&'static Knob> {
    KNOBS.iter().find(|knob| knob.key == key)
}

/// **The settings a JSON object holds**: every knob the object names and
/// this build can read, over the defaults. An absent key, a null, a value of
/// the wrong shape and a key no knob answers to are each the default — the
/// place file's own rungs (`crate::place`).
pub fn from_json(held: &Value) -> Settings {
    let mut out = Settings::default();
    for knob in KNOBS {
        let Some(value) = held.get(knob.key) else {
            continue;
        };
        let read = match knob.kind {
            Kind::Toggle => value.as_bool().map(Knobbed::On),
            // **Through serde, never a cast** — `f64 as f32` is the one
            // narrowing the lint set denies (`crate::place::width`).
            Kind::Number { .. } => serde_json::from_value(value.clone()).ok().map(Knobbed::At),
        };
        if let Some(read) = read {
            (knob.set)(&mut out, read);
        }
    }
    out
}

/// **The JSON object the settings are written as**: only the knobs standing
/// off their default, each under its key. Empty for a seat set to nothing.
pub fn to_json(settings: &Settings) -> Value {
    let rest = Settings::default();
    let mut out = Map::new();
    for knob in KNOBS {
        let held = (knob.get)(settings);
        if held == (knob.get)(&rest) {
            continue;
        }
        let value = match held {
            Knobbed::On(on) => json!(on),
            Knobbed::At(at) => json!(at),
        };
        out.insert(knob.key.to_owned(), value);
    }
    Value::Object(out)
}

#[cfg(test)]
mod tests;
