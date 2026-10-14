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

impl super::Model {
    /// **A receipt goes with the conversation it was spent on** (bl-a7f5). An
    /// act's sentence — `seen`, a nudge, a retarget — is about the selection it
    /// was fired at, and it stands in the state slot, which is the selected
    /// conversation's: left standing after the selection moves, it hides the new
    /// conversation's state under a sentence about another one. A failure is not
    /// retired here — a refusal is the operator's to read, wherever they went.
    pub(super) fn moving_to(&mut self, conversation: Option<&str>) {
        if self.conversation.as_deref() != conversation {
            self.notice.take_if(|said| matches!(said, Notice::Said(_)));
        }
    }
}

/// The door [`Notice::refused`] appends to a refused role write.
pub const CONFIG_DOOR: &str = "/config opens the settings editors";

/// **What an act is called in a sentence about it** (bl-8a4b): the words the
/// operator sent and the start's two acts are not words they typed, so they
/// are said as what they were; every other act is its own word. Never as
/// `/word`: a slash word in a notice is a door (`diagnostics::named`), and a
/// door to resend an act IN DOUBT is the one invitation it must not make.
fn act_named(op: &str) -> String {
    if op == crate::verbs::MESSAGE.word {
        return "your message".to_owned();
    }
    if [crate::verbs::PREPARE, crate::verbs::PROMPT].contains(&op) {
        return "the new conversation".to_owned();
    }
    format!("the {op}")
}

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
    pub fn answered(tool: &str, verdict: &str, scope: &str, advanced: bool) -> Self {
        let said = format!("{verdict} {tool} for this {scope}");
        Self::Said(if advanced {
            format!("{said} — the conversation is running again")
        } else {
            format!("{said} — it stays parked")
        })
    }

    /// **What `seen` answered, and whether it took the row off the queue**
    /// (bl-6802) — read off the queue the receipt says remains, never
    /// predicted: new evidence or undelivered mail keeps a seen conversation
    /// asking, and the sentence must not claim otherwise. `named` is what
    /// every other surface calls it (`Model::seen_name`, bl-c887).
    pub fn seen(acked: &crate::reply::queue::Acknowledged, named: &str) -> Self {
        let stays = acked
            .rows
            .iter()
            .any(|row| row.workspace == acked.workspace && row.agent == acked.agent);
        Self::Said(if stays {
            format!("seen {named} — it is still asking: something newer keeps it on the queue")
        } else {
            format!(
                "seen {named} — off the queue; {} still waiting on that engine",
                acked.rows.len()
            )
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
                "auto-approval is off for this conversation: every tool call but a read waits \
                 for you — /restore gives it back"
            } else {
                "auto-approval is back for this conversation — its tool calls follow the \
                 workspace's usual rules again"
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
            || "no branch to deliver".to_owned(),
            |source| format!("from {source}"),
        );
        let landed = commit.map_or_else(
            || "nothing landed".to_owned(),
            |commit| format!("at {commit}"),
        );
        Self::Said(format!(
            "delivered onto {target} {landed}, {from}, based on {base} — the ball stays \
             open, and closing it still delivers what it would have"
        ))
    }

    /// **What a retirement actually did to the source ref** (§4.36), which is
    /// the engine's answer and never this seat's prediction: an undeclared
    /// retention keeps the ref, and only the reply knows whether this project
    /// declared one and whether the keep had expired.
    pub fn retired(discarded: bool) -> Self {
        Self::Said(
            if discarded {
                "the candidate's worktree is released, and its branch is deleted: this \
                 project keeps them only so long"
            } else {
                "the candidate's worktree is released — its branch, and so its changes, \
                 are kept"
            }
            .to_owned(),
        )
    }

    /// **The seat's own door after an engine's refusal**, where the refused
    /// op names one (bl-8a4b): a role write the engine will not make is
    /// mended in the workspace's settings, which `/config` opens. Decided by
    /// the op this seat sent and never by the engine's prose, which it quotes
    /// untouched; the door is a run on the rule while the menu offers it
    /// (`crate::ui::status::diagnostics::named`).
    pub fn refused(op: &str, said: String) -> Self {
        use crate::verbs::{EFFORT, MODEL, PRIORITY};
        if [MODEL.word, EFFORT, PRIORITY].contains(&op) {
            return Self::Refused(format!("{said} — {CONFIG_DOOR}"));
        }
        Self::Refused(said)
    }

    /// The line the shell paints, with the half that says whose sentence it is.
    ///
    /// **Fact first, remedy last**, which every refusal this seat paints keeps
    /// and which the notice's own wrap (bl-3d0f) exists to protect: the half
    /// that says what to do is the half a cut sentence loses.
    pub fn line(&self) -> String {
        match self {
            Self::Refused(said) => format!("the engine refused: {said}"),
            Self::Unreadable(why) => format!(
                "lernie could not read the engine's answer ({why}) — updating lernie may fix it"
            ),
            Self::Unreachable(why) => format!("lernie could not reach {why}"),
            Self::InDoubt { op, why } => format!(
                "{} reached the engine but no answer came back ({why}), so it may have \
                 run — look before doing it again; this window is already re-reading",
                act_named(op)
            ),
            Self::Said(said) => said.clone(),
            Self::Unsent { op, why } => format!(
                "{} never reached the engine ({why}), so nothing happened — it is safe \
                 to do it again",
                act_named(op)
            ),
        }
    }
}
