//! **Why an exchange produced no answer** — and the one thing this end can say
//! about what the far end did with the request.
//!
//! yog's `docs/REMOTE.md` §3 rules the recovery: *"A lost reply leaves an act IN
//! DOUBT, and the recovery is a read — never a resend … a client whose act
//! earned a transport error instead of a reply paints the failure and consults
//! the world, which is the durable record … Asks are the opposite case and
//! re-ask freely."*
//!
//! Acting on that needs one fact a bare `Err(String)` cannot carry: **did the
//! request cross?** The two answers have opposite remedies — do it again, or do
//! nothing and look — so a seat that collapsed them would give one of them
//! wrong. That is the same division [`crate::ui::Notice`] already draws three
//! ways for three remedies, taken one layer down where the fact is known.
//!
//! **The seam is [`crate::channel::Channel::dial`]**, and it is the seam the
//! transport already had: everything `dial` refuses happened before the far end
//! could adjudicate anything, and what it hands back is — in its own words — *"a
//! socket with a request on it and no answer yet read"*. So the classification
//! is structural rather than a reading of a message, exactly as
//! `Channel::wrote` reads a typed `rustls::Error` rather than its wording.
//!
//! **A read does not care and must not be made to.** Both arms are one sentence
//! to the asker and the follow lane, which is what [`Reach::said`] is for: a
//! standing question is answered in place and asking twice is asking once, so
//! the classification is spent by the poster and by nobody else.

/// What a transport failure says about the request that earned it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reach {
    /// **The request never crossed.** The channel would not open, this box holds
    /// no material for it, the handshake did not verify, the write failed, or
    /// the peer stated a protocol this build does not speak — which REMOTE §3
    /// refuses *"before any gesture is decoded"*, so nothing was adjudicated
    /// there either.
    ///
    /// Nothing happened, so doing it again is safe.
    Unsent(String),
    /// **The request crossed and the answer did not come back.** The far end had
    /// the gesture and this end cannot say what it did with it: an act here is
    /// REMOTE §3's IN DOUBT and is never resent, because *"an act is not
    /// idempotent — two clicks of Nudge are two nudges"*.
    ///
    /// The recovery is a read. For a *read* this arm means nothing at all —
    /// ask again.
    Unanswered(String),
}

impl Reach {
    /// The sentence, without the classification. **What every caller that does
    /// not act on the difference gets**, so a read path carries no arm for a
    /// fact it has no use for.
    pub fn said(&self) -> String {
        match self {
            Self::Unsent(why) | Self::Unanswered(why) => why.clone(),
        }
    }

    /// Whether the request reached the far end. The whole of what the poster
    /// reads, named so the arm it picks is legible where it is picked.
    pub fn crossed(&self) -> bool {
        matches!(self, Self::Unanswered(_))
    }
}

#[cfg(test)]
mod tests;
