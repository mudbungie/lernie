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
    /// **An act that crossed and was never answered** — yog's `docs/REMOTE.md`
    /// §3's IN DOUBT. The engine had the gesture and this end cannot say what it
    /// did with it, so the effect may have landed.
    ///
    /// The fifth arm, and the first whose remedy is to do NOTHING. The four
    /// above are answered by an act — type something else, upgrade the seat,
    /// look at this box's files, do it again — and this one is answered by
    /// looking, because *"an act is not idempotent"* and a resend of one that
    /// already ran is a second effect nobody asked for. Which is why it must
    /// not wear any of the other four sentences: every one of them invites the
    /// act this one forbids.
    InDoubt { op: String, why: String },
    /// **An act that never left this box.** The counterpart, and it exists
    /// because the honest answers are opposite: nothing happened, so doing it
    /// again is safe and is the remedy.
    ///
    /// It is in the bar rather than on the channel's section — where the same
    /// transport failure on a READ goes — because an act is an exchange
    /// (`super::absorb`). A channel's section is also the slot the roster read
    /// owns and overwrites on every beat, so a click that went nowhere would
    /// have left a red sentence for less than a second and then nothing at all.
    Unsent { op: String, why: String },
}

impl Notice {
    /// **What an act with no reply is**, off the transport's own reading of
    /// whether the request crossed ([`crate::channel::Reach`]).
    pub fn act(op: &str, reach: &crate::channel::Reach) -> Self {
        let (op, why) = (op.to_owned(), reach.said());
        if reach.crossed() {
            return Self::InDoubt { op, why };
        }
        Self::Unsent { op, why }
    }

    /// The line the shell paints, with the half that says whose sentence it is.
    ///
    /// **Fact first, remedy last**, which every refusal this seat paints keeps
    /// and which the notice's own wrap (bl-3d0f) exists to protect: the half
    /// that says what to do is the half a cut sentence loses.
    pub fn line(&self) -> String {
        match self {
            Self::Refused(said) => format!("the engine refused: {said}"),
            Self::Unreadable(why) => format!("this seat could not read the answer: {why}"),
            Self::Unreachable(why) => format!("this seat could not reach it: {why}"),
            Self::InDoubt { op, why } => format!(
                "`{op}` is IN DOUBT: it reached the engine and no answer came back \
                 ({why}), so it may have run. This seat never resends an act — the \
                 world is the record, and the reads on this window are already \
                 asking it again."
            ),
            Self::Unsent { op, why } => format!(
                "`{op}` was not sent: it never left this seat ({why}), so nothing \
                 happened — it is safe to do it again."
            ),
        }
    }
}
