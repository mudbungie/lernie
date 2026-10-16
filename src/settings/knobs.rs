//! **The rows** (DESIGN §4.46 item 4's table), in the order the pane paints
//! them. Each is three strings and two accessors; the help sentence says
//! what the knob does in the operator's terms and never the toolkit's.

use super::{BAR_SHUT, DESK, Kind, Knob, Knobbed, Toggle, points, seconds};

/// **Every knob.** Keys are the file's words; the pin is not here because it
/// is the place file's own key (`crate::place::Place::pinned`), painted on
/// the pane as a row beside these.
pub const KNOBS: &[Knob] = &[
    Knob {
        key: "wheel",
        label: "wheel step",
        help: "how far one notch of the wheel scrolls",
        kind: points(20.0, 400.0, 10.0),
        get: |s| Knobbed::At(s.wheel),
        set: |s, v| {
            if let Knobbed::At(at) = v {
                s.wheel = at;
            }
        },
    },
    Knob {
        key: "help",
        label: "hover help",
        help: "whether a control says what it does when the pointer rests on it",
        kind: Kind::Toggle,
        get: |s| Knobbed::On(s.on(Toggle::Help)),
        set: |s, v| {
            if let Knobbed::On(on) = v {
                s.set(Toggle::Help, on);
            }
        },
    },
    Knob {
        key: "help_delay",
        label: "hover help delay",
        help: "how long the pointer rests on a control before it says what it does",
        kind: seconds(3.0, 0.1),
        get: |s| Knobbed::At(s.help_delay),
        set: |s, v| {
            if let Knobbed::At(at) = v {
                s.help_delay = at;
            }
        },
    },
    Knob {
        key: super::SCALE_KEY,
        label: "scale",
        help: "how large everything is drawn: text, air and controls alike",
        kind: Kind::Number {
            unit: "×",
            floor: 0.5,
            ceiling: 2.0,
            step: 0.1,
        },
        get: |s| Knobbed::At(s.scale),
        set: |s, v| {
            if let Knobbed::At(at) = v {
                s.scale = at;
            }
        },
    },
    Knob {
        key: "linger",
        label: "bar linger",
        help: "how long the bar stays open after the pointer leaves it",
        kind: seconds(5.0, 0.05),
        get: |s| Knobbed::At(s.linger),
        set: |s, v| {
            if let Knobbed::At(at) = v {
                s.linger = at;
            }
        },
    },
    Knob {
        key: "ease",
        label: "bar ease",
        help: "how long the bar's edge takes to slide between its two widths; 0 snaps",
        kind: seconds(1.0, 0.01),
        get: |s| Knobbed::At(s.ease),
        set: |s, v| {
            if let Knobbed::At(at) = v {
                s.ease = at;
            }
        },
    },
    Knob {
        key: "bar_shut",
        label: "bar width, shut",
        help: "how wide the bar stands while it is shut — what dragging its edge shut sets",
        kind: points(40.0, 400.0, 1.0),
        get: |s| Knobbed::At(s.bar_shut.unwrap_or(BAR_SHUT)),
        set: |s, v| {
            if let Knobbed::At(at) = v {
                s.bar_shut = Some(at);
            }
        },
    },
    Knob {
        key: "bar_open",
        label: "bar width, open",
        help: "how wide the bar stands while it is open — what dragging its edge open sets",
        kind: points(100.0, 1000.0, 10.0),
        get: |s| Knobbed::At(s.bar_open.unwrap_or(DESK / 4.0)),
        set: |s, v| {
            if let Knobbed::At(at) = v {
                s.bar_open = Some(at);
            }
        },
    },
    Knob {
        key: "bubbles",
        label: "bubbles",
        help: "whether each block of the transcript stands on a faint fill saying who said it",
        kind: Kind::Toggle,
        get: |s| Knobbed::On(s.on(Toggle::Bubbles)),
        set: |s, v| {
            if let Knobbed::On(on) = v {
                s.set(Toggle::Bubbles, on);
            }
        },
    },
    Knob {
        key: "fold",
        label: "fold the working",
        help: "whether a turn's working folds to one line once its answer stands",
        kind: Kind::Toggle,
        get: |s| Knobbed::On(s.on(Toggle::Fold)),
        set: |s, v| {
            if let Knobbed::On(on) = v {
                s.set(Toggle::Fold, on);
            }
        },
    },
    Knob {
        key: "confirm",
        label: "confirm a delete",
        help: "whether the trashcan asks before deleting; off, it deletes at once",
        kind: Kind::Toggle,
        get: |s| Knobbed::On(s.on(Toggle::Confirm)),
        set: |s, v| {
            if let Knobbed::On(on) = v {
                s.set(Toggle::Confirm, on);
            }
        },
    },
    Knob {
        key: "dock",
        label: "dock unfolded",
        help: "whether the dock of running conversations stands unfolded when the window opens",
        kind: Kind::Toggle,
        get: |s| Knobbed::On(s.on(Toggle::Dock)),
        set: |s, v| {
            if let Knobbed::On(on) = v {
                s.set(Toggle::Dock, on);
            }
        },
    },
];
