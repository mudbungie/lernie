//! **The bindings table** (DESIGN §4.46 item 3): every key this window
//! binds, in one home — what [`super::handle`] runs, what `/help` prints
//! (`crate::ui::commands`), and what a control bound by a row cites after
//! its help sentence ([`cited`]).
//!
//! **Two kinds of row, one table.** A *fired* row is a chord and the act it
//! fires, run here from anywhere on the glass — the box included, because a
//! chord is not a key a box takes. A *told* row is a key read where it is
//! read — the box's Enter, the menu's Tab, the ask's digits, the walk's
//! arrows, Esc's order — and stands in the table so the printed list is
//! whole without a second list beside it. One table rather than two because
//! the reader's question is *what does the keyboard do*, and the answer has
//! one order.
//!
//! **Every word is a keyboard path already**: a pane, a verb or a door is
//! `/` and its word, so a word gains no chord for being a word. What the
//! fired rows add is the acts that are neither a word nor a Tab stop worth
//! tabbing to; the set is small and conventional, and §4.46 item 3 is the
//! argument for each.

use crate::ui::Model;

/// **A key on a modifier plane.** `ctrl` is egui's `command`, which on this
/// seat's platforms is the Ctrl key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chord {
    pub key: egui::Key,
    pub ctrl: bool,
    pub shift: bool,
}

impl Chord {
    /// The key alone.
    pub const fn plain(key: egui::Key) -> Self {
        Self {
            key,
            ctrl: false,
            shift: false,
        }
    }

    /// The key under Ctrl.
    pub const fn ctrl(key: egui::Key) -> Self {
        Self {
            key,
            ctrl: true,
            shift: false,
        }
    }

    /// **How the chord is written** on the glass: `Ctrl+B`, `PageDown`,
    /// `Ctrl+,`. egui's own names, except where the name is a word for a
    /// mark an operator reads as the mark.
    pub fn spell(self) -> String {
        let key = match self.key {
            egui::Key::Comma => ",",
            egui::Key::Equals => "=",
            egui::Key::Minus => "-",
            egui::Key::Escape => "Esc",
            key => key.name(),
        };
        let ctrl = if self.ctrl { "Ctrl+" } else { "" };
        let shift = if self.shift { "Shift+" } else { "" };
        format!("{ctrl}{shift}{key}")
    }

    /// **Whether this frame pressed exactly this chord** — the key on this
    /// plane and no other, so `PageDown` is not also `Ctrl+PageDown`.
    pub fn pressed(self, ctx: &egui::Context) -> bool {
        ctx.input(|i| {
            i.events.iter().any(|event| {
                matches!(
                    event,
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } if *key == self.key
                        && modifiers.command == self.ctrl
                        && modifiers.shift == self.shift
                )
            })
        })
    }
}

/// **What a fired row does**, named so a control can cite its chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Act {
    /// The switcher (§4.42 item 7).
    Switch,
    /// The model's dropup (`/model`).
    Model,
    /// The dock, unfolded or folded (§4.42 item 9).
    Dock,
    /// The settings pane (§4.46 item 4).
    Settings,
    /// `/help`.
    Help,
    /// The bar pinned open, or unpinned (§4.44 item 1).
    Pin,
    /// A conversation begun on the aimed workspace — `+ new`.
    New,
    /// The selected tab closed (§4.44 item 2).
    CloseTab,
    /// The next tab, the previous.
    NextTab,
    PrevTab,
    /// The transcript scrolled a page, or to an end.
    PageDown,
    PageUp,
    End,
    Head,
    /// The `scale` knob moved, or put back.
    ScaleUp,
    ScaleDown,
    ScaleReset,
}

/// A row's key: fired here, or told and read where it is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyed {
    Fired(Chord, Act),
    Told(&'static str),
}

/// **One row**: its key and what it does, in the operator's terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Binding {
    pub keyed: Keyed,
    pub does: &'static str,
}

impl Binding {
    pub(super) const fn fired(chord: Chord, act: Act, does: &'static str) -> Self {
        Self {
            keyed: Keyed::Fired(chord, act),
            does,
        }
    }

    pub(super) const fn told(says: &'static str, does: &'static str) -> Self {
        Self {
            keyed: Keyed::Told(says),
            does,
        }
    }

    /// How the row's key is written.
    pub fn says(&self) -> String {
        match self.keyed {
            Keyed::Fired(chord, _) => chord.spell(),
            Keyed::Told(says) => says.to_owned(),
        }
    }
}

use egui::Key;

/// **The table**, in the order `/help` prints it.
pub const BINDINGS: &[Binding] = &[
    Binding::told("Enter", "sends the box, or fires the chosen word"),
    Binding::told("Shift+Enter", "breaks a line"),
    Binding::told("/", "opens the words"),
    Binding::told("Tab", "completes the chosen word into the box"),
    Binding::told("Up, Down", "walk whatever is open"),
    Binding::told(
        "Esc",
        "closes a dropup, then the overlay, then the menu, then the notice",
    ),
    Binding::told("1 to 6", "answers the question it is asking, by its number"),
    Binding::fired(
        Chord::ctrl(Key::X),
        Act::Switch,
        "every engine, its workspaces and their conversations",
    ),
    Binding::fired(
        Chord::ctrl(Key::O),
        Act::Model,
        "changes the model this workspace runs on",
    ),
    Binding::fired(
        Chord::ctrl(Key::T),
        Act::Dock,
        "unfolds what is running, or folds it",
    ),
    Binding::fired(
        Chord::ctrl(Key::Comma),
        Act::Settings,
        "opens this seat's settings",
    ),
    Binding::fired(
        Chord::ctrl(Key::B),
        Act::Pin,
        "pins the bar open, or unpins it",
    ),
    Binding::fired(
        Chord::ctrl(Key::N),
        Act::New,
        "begins a conversation on the aimed workspace",
    ),
    Binding::fired(
        Chord::ctrl(Key::W),
        Act::CloseTab,
        "closes the selected tab",
    ),
    Binding::fired(Chord::ctrl(Key::PageDown), Act::NextTab, "the next tab"),
    Binding::fired(Chord::ctrl(Key::PageUp), Act::PrevTab, "the previous tab"),
    Binding::fired(
        Chord::plain(Key::PageDown),
        Act::PageDown,
        "scrolls the transcript a page on",
    ),
    Binding::fired(Chord::plain(Key::PageUp), Act::PageUp, "a page back"),
    Binding::fired(Chord::ctrl(Key::End), Act::End, "the transcript's end"),
    Binding::fired(Chord::ctrl(Key::Home), Act::Head, "its head"),
    Binding::fired(
        Chord::ctrl(Key::Equals),
        Act::ScaleUp,
        "scales the window up",
    ),
    Binding::fired(Chord::ctrl(Key::Minus), Act::ScaleDown, "scales it down"),
    Binding::fired(
        Chord::ctrl(Key::Num0),
        Act::ScaleReset,
        "puts the scale back to 1",
    ),
    Binding::fired(
        Chord::plain(Key::F1),
        Act::Help,
        "what this window takes — /help",
    ),
];

/// **The one chord the table does not print**: a keyboard whose `=` key is
/// `+` under Shift delivers Ctrl+Plus for the scale-up chord, and the
/// operator pressed what the row says.
pub const ALIAS: (Chord, Act) = (Chord::ctrl(Key::Plus), Act::ScaleUp);

/// **The chord that fires `act`**, spelled — empty for an act no row fires,
/// which the table's test says is none.
pub fn chord_for(act: Act) -> String {
    BINDINGS
        .iter()
        .find_map(|row| match row.keyed {
            Keyed::Fired(chord, fires) if fires == act => Some(chord.spell()),
            _ => None,
        })
        .unwrap_or_default()
}

/// **A help sentence with its chords after it** — `help · Ctrl+B`, the
/// chords parted by ` / ` — or the sentence alone where `acts` is empty.
pub fn cited(help: &str, acts: &[Act]) -> String {
    if acts.is_empty() {
        return help.to_owned();
    }
    let chords: Vec<String> = acts.iter().map(|act| chord_for(*act)).collect();
    format!("{help} · {}", chords.join(" / "))
}

/// **The slash word a fired row is also reachable by**, where one is: the
/// chord and the word open the same thing, so the menu's row cites it.
fn word_of(act: Act) -> Option<&'static str> {
    match act {
        Act::Switch => Some(crate::ui::switcher::WORD),
        Act::Model => Some(crate::ui::tuning::MODEL),
        Act::Settings => Some(crate::ui::settings::WORD),
        Act::Help => Some(crate::ui::commands::WORD),
        _ => None,
    }
}

/// **A slash row's help with its chord**, where a chord opens the same word.
pub fn cited_word(help: &str, word: &str) -> String {
    let act = BINDINGS.iter().find_map(|row| match row.keyed {
        Keyed::Fired(_, act) if word_of(act) == Some(word) => Some(act),
        _ => None,
    });
    act.map_or_else(|| help.to_owned(), |act| cited(help, &[act]))
}

/// **Run the table over this frame's keys.**
pub fn run(ctx: &egui::Context, model: &mut Model) {
    for row in BINDINGS {
        if let Keyed::Fired(chord, act) = row.keyed
            && chord.pressed(ctx)
        {
            super::acts::fire(act, ctx, model);
        }
    }
    if ALIAS.0.pressed(ctx) {
        super::acts::fire(ALIAS.1, ctx, model);
    }
}
