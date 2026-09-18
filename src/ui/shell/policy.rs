//! **What a window's width buys it**, and the two columns a window is made of
//! (bl-e5d2, bl-dfda, bl-b9a3).
//!
//! This is the layout's own policy and the one place it is stated. It is a
//! **pure function of one number**, so what the window does as it narrows is a
//! value a test reads back rather than a layout somebody has to look at — and
//! the harness that photographs the window judges every width it renders,
//! because there is no width this answers nothing for.
//!
//! # Two shapes, and the second is the answer the policy used to lack
//!
//! [`widths`] is the yield: the conversation keeps a floor and the one list
//! pane gives way to it until it reaches its own floor. Past that **nothing
//! yields**, and until bl-dfda that was where the policy stopped — at a
//! phone-shaped viewport the columns were still laid side by side, each about
//! 120 points wide, with every line in every one of them wrapped to two or
//! three words.
//!
//! The answer is not more yielding, because there is none left to do: it is a
//! second **shape**. Below the width at which the yield still leaves the
//! conversation its floor, the window shows [`Column`] — one column at a time,
//! with a bar naming the two. That is the covering-pane idiom this seat
//! already has ([`crate::ui::enroll`], [`crate::ui::tuning`],
//! [`crate::ui::records`]) read across the whole layout rather than only the
//! central panel: a surface you navigate to, act in, and come back from.
//!
//! # The policy LOST a term when the middle column went (DESIGN §4.39)
//!
//! It used to yield two list panes together and in proportion to what each was
//! worth. The conversations stand under their wall in the roster now
//! ([`crate::ui::roster`]), so there is one list, one width to answer and one
//! edge to drag — and every expression below is the old one with the second
//! term struck out rather than a new regime. What the list is worth is still
//! [`CONVS`], because its widest row is still a conversation's headline with a
//! preview under it; the engine and wall rows above them are shorter.
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

/// **What an operator's drag is worth**, and what this policy still owns of it
/// — the default and the floors (DESIGN §4.39). Split from this file on the
/// seam §4.39 itself draws: the yield below is what a WINDOW's width buys, and
/// that is what a SEAT's drag buys once the window has answered.
pub mod edges;

pub use edges::{TAIL_FLOOR, rows, shown, span};

/// **What the one list pane is worth when the window is wide enough**, in
/// points: its widest row is a conversation's headline with a preview under
/// it, which is what it was worth as a column of its own and is what it is
/// worth now that the engines stand above the same rows.
pub(crate) const CONVS: f32 = 320.0;

/// **The floor the conversation and its composer keep** in the broad shape.
/// Below this a chat pane is a strip: a message elides inside its own width,
/// the composer's box shows the first few words of a draft, and `send` sits
/// against the frame. It is what the list pane yields to, and the width at
/// which it can no longer be kept is where the narrow shape begins.
pub const CHAT_FLOOR: f32 = 420.0;

/// **The width the list pane never goes under** while it is on the glass
/// beside the conversation. A pane below it shows nothing at all, which is
/// worse than a chat pane under its floor — so this is the one thing the floor
/// yields to.
pub const SIDE_FLOOR: f32 = 140.0;

/// **One of the two columns the window is made of** (DESIGN §4.39).
///
/// In the broad shape both are on the glass at once and this says nothing;
/// in the narrow shape it is the one that IS on the glass, and the bar that
/// names both is how an operator moves between them
/// (`crate::ui::shell`). It is held on the model because it is a navigation an
/// operator performed, which is not a thing any other fact can be asked for.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Column {
    /// The accordion of engines, their walls and the aimed wall's
    /// conversations — the window's one list. **The window opens here**: a
    /// seat with nothing aimed at has exactly one thing to do next.
    #[default]
    Engines,
    /// The selected conversation, and the composer under it.
    Conversation,
}

impl Column {
    /// **The two, left to right** — the order the broad shape lays them in,
    /// so the bar reads the same way the wide window does and a step sideways
    /// means the same thing in both shapes.
    pub(crate) fn all() -> [Column; 2] {
        [Self::Engines, Self::Conversation]
    }

    /// **The column's one name, which is the pane's own heading.** The bar does
    /// not get a vocabulary of its own: a second word for a column is a second
    /// thing to keep in step, and an operator reading *engines* on a bar and
    /// something else over the pane would be reading about two places.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Engines => crate::ui::roster::HEADING,
            Self::Conversation => crate::ui::chat::HEADING,
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

/// **What a window of this width gets**: both columns at once, or one at a
/// time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    /// **Both columns on the glass together**, with the list pane capped at
    /// this width.
    Broad {
        /// What the list pane may take.
        list: f32,
    },
    /// **One column at a time**, and a bar naming both.
    Narrow,
}

/// **The width every column is worth together** — the one window width at
/// which each of the two gets exactly what it is worth and nothing is left
/// over. It is derived and never written down twice: it is the crossing point
/// of [`widths`]' two clauses, so tuning either of the two moves it.
fn worth() -> f32 {
    CONVS + CHAT_FLOOR
}

/// **The list pane's width at a given window width** — the yield, and the
/// policy the window had none of (bl-e5d2, bl-fef8).
///
/// The side panels used to keep their widths as the window narrowed and the
/// central panel absorbed the whole loss, so at 900 points the pane the window
/// exists for was a ~140-point strip while the roster kept 280. The rule is the
/// other way round: **the conversation has a floor and the list pane yields to
/// it**, until it reaches its own floor.
///
/// **And a window WIDER than every column's worth is the same rule read the
/// other way** (bl-fef8): the list's share stopped growing at 1.0, so every
/// pixel of a large display landed in the one pane whose content is already
/// prose, and the navigation column stayed the width it is worth at a
/// 740-point window forever. Both are one expression — the share is the
/// SMALLER of *this column's proportion of the window* and *what is left after
/// the conversation keeps its floor* — and the two clauses cross at exactly
/// [`worth`], which is what makes the growth a continuation of the yield
/// rather than a second regime with a constant of its own.
pub fn widths(window: f32) -> f32 {
    let share = (window / worth()).min((window - CHAT_FLOOR) / CONVS);
    (CONVS * share).max(SIDE_FLOOR)
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
    let list = widths(window);
    if window - list >= CHAT_FLOOR {
        Shape::Broad { list }
    } else {
        Shape::Narrow
    }
}

#[cfg(test)]
mod tests;
