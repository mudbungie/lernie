//! **The interface mode** (DESIGN §4.48 item 8): which home the three
//! re-homed panes have — floating over the column, or docked beside it as
//! the branch and the context rails. The baseline's two modes, its two
//! sentences for them, and its one sentence about both: *changes what is
//! shown, not what the seat can do*.
//!
//! A mode is a seat setting and nothing more: it moves no fact across the
//! wire and opens no pane. The query that reads it is `Model::docked`
//! (`crate::ui::model::rail`), which is the one place the home is decided.

/// The two modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// For talking: the bar and the chat, every pane floating.
    #[default]
    Simple,
    /// Rails beside the chat: the branch and the context.
    Advanced,
}

impl Mode {
    /// The two, in the choice's order.
    pub const ALL: [Self; 2] = [Self::Simple, Self::Advanced];

    /// The word on the knob, on the head's door, and in the place file.
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Advanced => "advanced",
        }
    }

    /// **What the mode is for**, in the baseline's words.
    pub(crate) const fn says(self) -> &'static str {
        match self {
            Self::Simple => "for talking; the bar and the chat",
            Self::Advanced => "rails beside the chat — the branch and the context",
        }
    }

    /// The other one — what the chord and the door switch to.
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Simple => Self::Advanced,
            Self::Advanced => Self::Simple,
        }
    }

    /// The mode `word` names, if either does.
    pub fn of(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.word() == word)
    }

    /// **The choice as the knob table spells it**: each word with its
    /// sentence, in order.
    pub const CHOICE: &'static [(&'static str, &'static str)] = &[
        (Self::Simple.word(), Self::Simple.says()),
        (Self::Advanced.word(), Self::Advanced.says()),
    ];
}
