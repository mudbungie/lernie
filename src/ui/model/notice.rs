//! **What the seat last heard that was not content** — the sentence the shell
//! paints where an answer's content would have been.
//!
//! Split from [`super`] at the design-time budget on a seam of its own: the
//! model is what the window holds and how it changes, and this is one closed
//! vocabulary of what the seat heard that it cannot paint as content, with the
//! one line that says whose sentence it is. It changes when a new *kind* of
//! not-content appears, which is not when the model changes.
//!
//! **Five of the six are things that went wrong and the sixth is not**
//! (bl-bce2). A receipt that carries a FACT — which call was released, whether
//! a floor stands — is not content either: there is no row it belongs under
//! and no pane it fills, and the window has exactly one place for a one-line
//! statement about an act just performed. So [`Notice::Said`] is here, and
//! being here is what makes it dismissible: an act is an event and not a
//! state, so it does not re-post on a beat.

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
    /// **An act's receipt, in the engine's own terms** — the sixth arm, and
    /// the first that is not a failure at all.
    ///
    /// It exists because two acts answer with a fact rather than with content:
    /// `answer` says which call it landed on and whether the conversation is
    /// running again, and `revoke`/`restore` say whether a floor stands over
    /// the conversation now — re-derived by the engine, so it is the one
    /// statement about a floor this seat can make (§4.34).
    Said(String),
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

    /// **What an answer landed on, and what it did to the conversation.** The
    /// verdict rides verbatim (`crate::reply`'s rung 3), and `advanced` is the
    /// half worth saying: `pass` and `refuse` both drive the branch on, where
    /// `hold` is the operator saying *stay parked* and launches nothing.
    pub fn answered(tool: &str, tool_use: &str, verdict: &str, advanced: bool) -> Self {
        let said = format!("answered {tool} ({tool_use}): {verdict}");
        Self::Said(if advanced {
            format!("{said} — the conversation is running again")
        } else {
            format!("{said} — it stays parked")
        })
    }

    /// **Whether a floor stands over the conversation now**, which is the
    /// engine's re-derivation and never an echo of the direction that was
    /// asked: restoring one whose ancestor is still revoked leaves it floored,
    /// and this is the sentence that says so instead of claiming a restore
    /// that did not happen.
    pub fn floored(standing: bool) -> Self {
        Self::Said(
            if standing {
                "a floor stands over this conversation: every tool call but a read waits for you"
            } else {
                "the floor is lifted — its tool calls are adjudicated by the ordinary policy again"
            }
            .to_owned(),
        )
    }

    /// **What a delivery acted on** (§4.36). The identities and not a verdict:
    /// the standing fact is the tagged squash the target's history now
    /// carries, so what this seat can honestly say is which refs moved and to
    /// what — never *this candidate won*, which is a thing about a cohort
    /// nobody records.
    /// **Two of the four identities are optional and each absence is a fact**
    /// rather than a blank: no source ref means there was none to deliver, and
    /// no commit means the delivery landed nothing. Saying either as an empty
    /// string would report a delivery that did not happen.
    pub fn delivered(
        base: &str,
        target: &str,
        source: Option<String>,
        commit: Option<String>,
    ) -> Self {
        let from = source.map_or_else(
            || "no source ref".to_owned(),
            |source| format!("from {source}"),
        );
        let landed = commit.map_or_else(
            || "nothing landed".to_owned(),
            |commit| format!("at {commit}"),
        );
        Self::Said(format!(
            "delivered onto {target} {landed}, {from}, off the pinned {base} — the \
             ball is not closed and what its close delivers is unchanged"
        ))
    }

    /// **What a retirement actually did to the source ref** (§4.36), which is
    /// the engine's answer and never this seat's prediction: an undeclared
    /// retention keeps the ref, and only the reply knows whether this project
    /// declared one and whether the keep had expired.
    pub fn retired(discarded: bool) -> Self {
        Self::Said(
            if discarded {
                "the candidate's worktree is released, and its source ref went with it: \
                 this project's retention says the keep had expired"
            } else {
                "the candidate's worktree is released — its source ref, and so its whole \
                 diff, is still addressable"
            }
            .to_owned(),
        )
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
            Self::Said(said) => said.clone(),
            Self::Unsent { op, why } => format!(
                "`{op}` was not sent: it never left this seat ({why}), so nothing \
                 happened — it is safe to do it again."
            ),
        }
    }
}
