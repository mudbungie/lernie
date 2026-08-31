//! **What the seat last heard that was not content** — the sentence the shell
//! paints where an answer's content would have been.
//!
//! Split from [`super`] at the design-time budget on a seam of its own: the
//! model is what the window holds and how it changes, and this is one closed
//! vocabulary of what went wrong, with the one line that says whose sentence it
//! is. It changes when a new *kind* of not-content appears, which is not when
//! the model changes.

/// What the seat last heard that it could not turn into content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    /// The engine refused, in its own words.
    Refused(String),
    /// This seat could not read what arrived — a malformed frame, or a kind
    /// this build does not paint. A statement about **this seat**, which is why
    /// it reads differently from a refusal.
    Unreadable(String),
    /// This seat could not **reach** the far end: a channel that will not open,
    /// an engine that is not there, a preface that did not agree.
    ///
    /// Three arms and not two, because the remedies are three different acts. A
    /// refusal is answered by typing something else; an unreadable frame by
    /// upgrading the seat; an unreachable channel by looking at this box's own
    /// files or at whether the engine is up. A seat that collapsed them would
    /// send an operator to check a certificate over a workspace they mistyped.
    Unreachable(String),
}

impl Notice {
    /// The line the shell paints, with the half that says whose sentence it is.
    pub fn line(&self) -> String {
        match self {
            Self::Refused(said) => format!("the engine refused: {said}"),
            Self::Unreadable(why) => format!("this seat could not read the answer: {why}"),
            Self::Unreachable(why) => format!("this seat could not reach it: {why}"),
        }
    }
}
