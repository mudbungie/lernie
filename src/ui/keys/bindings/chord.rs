//! **A chord** (DESIGN §4.46 item 3): a key on a modifier plane, how it is
//! written on the glass, and whether this frame pressed exactly it. Split
//! from [`super`] at the 300-line cap on the seam the table draws: that
//! file is what the rows ARE, and this is what a row's key is.

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

    /// The key under Ctrl and Shift.
    pub const fn ctrl_shift(key: egui::Key) -> Self {
        Self {
            key,
            ctrl: true,
            shift: true,
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
            egui::Key::Backslash => "\\",
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
