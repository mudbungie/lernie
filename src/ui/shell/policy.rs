//! **What a window's width buys it**, and the three columns a window is made
//! of (bl-e5d2, bl-dfda).
//!
//! This is the layout's own policy and the one place it is stated. It is a
//! **pure function of one number**, so what the window does as it narrows is a
//! value a test reads back rather than a layout somebody has to look at — and
//! the harness that photographs the window judges every width it renders,
//! because there is no width this answers nothing for.
//!
//! # Two shapes, and the second is the answer the policy used to lack
//!
//! [`widths`] is the yield: the conversation keeps a floor and the two list
//! panes give way to it, together and in proportion to what each is worth,
//! until they reach their own floor. Past that **nothing yields**, and until
//! bl-dfda that was where the policy stopped — at a phone-shaped viewport the
//! three columns were still laid side by side, each about 120 points wide, with
//! every line in every one of them wrapped to two or three words.
//!
//! The answer is not more yielding, because there is none left to do: it is a
//! second **shape**. Below the width at which the yield still leaves the
//! conversation its floor, the window shows [`Column`] — one column at a time,
//! with a bar naming the three. That is the covering-pane idiom this seat
//! already has ([`crate::ui::enroll`], [`crate::ui::tuning`],
//! [`crate::ui::records`]) read across the whole layout rather than only the
//! central panel: a surface you navigate to, act in, and come back from.
//!
//! # There is no floor under the narrow shape, and that is not an omission
//!
//! A width policy needs a floor exactly where two things compete for one
//! window, and in the narrow shape nothing competes: the shown column has the
//! whole width. So there is no width at which this runs out of an answer, which
//! is what lets the picture-taking harness drop its *is this width promised*
//! question entirely rather than keep a gate that now says yes to everything.
//! What a very small window costs is elision inside the content, which is the
//! content's own business and every pane's own rule.

use crate::ui::keys::Pane;

/// **What the two list panes are worth when the window is wide enough**, in
/// points: the roster holds a handful of short words, the conversation list
/// holds a headline and a preview under it.
pub(crate) const ROSTER: f32 = 280.0;
/// The conversation list's own worth, on the same reading.
pub(crate) const CONVS: f32 = 320.0;

/// **The floor the conversation and its composer keep** in the broad shape.
/// Below this a chat pane is a strip: a message elides inside its own width,
/// the composer's box shows the first few words of a draft, and `send` sits
/// against the frame. It is what the two list panes yield to, and the width at
/// which it can no longer be kept is where the narrow shape begins.
pub const CHAT_FLOOR: f32 = 420.0;

/// **The width a list pane never goes under** while it is on the glass beside
/// another. A pane below it shows nothing at all, which is worse than a chat
/// pane under its floor — so this is the one thing the floor yields to.
pub const SIDE_FLOOR: f32 = 140.0;

/// **One of the three columns the window is made of.**
///
/// In the broad shape all three are on the glass at once and this says nothing;
/// in the narrow shape it is the one that IS on the glass, and the bar that
/// names all three is how an operator moves between them
/// (`crate::ui::shell`). It is held on the model because it is a navigation an
/// operator performed, which is not a thing any other fact can be asked for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Column {
    /// The roster: every workspace this seat can reach. **The window opens
    /// here**, for `crate::ui::keys::Pane`'s own reason — a seat with nothing
    /// aimed at has exactly one thing to do next.
    #[default]
    Channels,
    /// The aimed wall's conversations.
    Conversations,
    /// The selected conversation, and the composer under it.
    Conversation,
}

impl Column {
    /// **The three, left to right** — the order the broad shape lays them in,
    /// so the bar reads the same way the wide window does and a step sideways
    /// means the same thing in both shapes.
    pub(crate) fn all() -> [Column; 3] {
        [Self::Channels, Self::Conversations, Self::Conversation]
    }

    /// **The column's one name, which is the pane's own heading.** The bar does
    /// not get a vocabulary of its own: a second word for a column is a second
    /// thing to keep in step, and an operator reading *channels* on a bar and
    /// something else over the pane would be reading about two places.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Channels => crate::ui::roster::HEADING,
            Self::Conversations => crate::ui::convs::HEADING,
            Self::Conversation => crate::ui::chat::HEADING,
        }
    }

    /// **Which list the arrows walk while this column is the one on the
    /// glass.**
    ///
    /// In the narrow shape the arrows have no choice to make: there is one
    /// column on the window, so it is the one they belong to, and
    /// `crate::ui::keys` spends this at the top of every frame rather than
    /// letting a focus set at another width walk a list nobody can see. The
    /// conversation column answers the conversation LIST, because the row a
    /// walk lands on is exactly what the chat pane is showing.
    pub(crate) fn arrows(self) -> Pane {
        match self {
            Self::Channels => Pane::Roster,
            Self::Conversations | Self::Conversation => Pane::Conversations,
        }
    }

    /// **One column left or right, saturating at the ends.**
    ///
    /// The ends do not wrap, for `crate::ui::keys::moved`'s reason one level
    /// up: a wrap makes the same keypress mean *the next one* twice and *back
    /// to the start* once, with nothing on the glass to say which it will be.
    pub(crate) fn stepped(self, step: isize) -> Column {
        let all = Self::all();
        let at = all.iter().position(|column| *column == self).unwrap_or(0);
        all.get(at.saturating_add_signed(step).min(all.len() - 1))
            .copied()
            .unwrap_or(self)
    }
}

/// **What a window of this width gets**: every column at once, or one at a
/// time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// **All three columns on the glass together**, with the two list panes
    /// capped at these widths.
    Broad {
        /// What the roster pane may take.
        roster: f32,
        /// What the conversation list may take.
        convs: f32,
    },
    /// **One column at a time**, and a bar naming the three.
    Narrow,
}

/// **The two list panes' widths at a given window width** — the yield, and the
/// policy the window had none of (bl-e5d2).
///
/// The side panels used to keep their widths as the window narrowed and the
/// central panel absorbed the whole loss, so at 900 points the pane the window
/// exists for was a ~140-point strip while the roster kept 280. The rule is the
/// other way round: **the conversation has a floor and the list panes yield to
/// it**, together and in proportion to what each is worth, until they reach
/// their own floor.
pub fn widths(window: f32) -> (f32, f32) {
    let share = ((window - CHAT_FLOOR) / (ROSTER + CONVS)).clamp(0.0, 1.0);
    (
        (ROSTER * share).max(SIDE_FLOOR),
        (CONVS * share).max(SIDE_FLOOR),
    )
}

/// **The shape a window of this width takes**, and the whole of the decision.
///
/// The line between the two is [`widths`] itself rather than a second constant:
/// the broad shape holds exactly as long as the yield still leaves the
/// conversation [`CHAT_FLOOR`], and the first width where it cannot is the
/// first width one column at a time is the better answer. A number written here
/// would be a copy of a policy that already lives in one function, and the two
/// would part company on the first tuning of either.
pub fn shape(window: f32) -> Shape {
    let (roster, convs) = widths(window);
    if window - roster - convs >= CHAT_FLOOR {
        Shape::Broad { roster, convs }
    } else {
        Shape::Narrow
    }
}

#[cfg(test)]
mod tests;
