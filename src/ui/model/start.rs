//! **A start, between its two acts** — the one thing this window holds across a
//! round trip, and the two receipts that move it along.
//!
//! Every other gesture the composer fires is answered by content: a deposit
//! earns a captured run, an advance earns a receipt that says only *it
//! launched*. A start is not one gesture. `prepare` stages it and answers the
//! body the fire must carry; `prompt` fires that body with the goal and answers
//! the name the engine minted. So the second act is composed by the frame that
//! **absorbs the first's receipt**, which is the only shape that keeps the
//! window's own rule: a frame composes into the outbox and never waits on a
//! socket.
//!
//! **A start in flight is what refuses a second one**, and the gate is the held
//! value itself rather than a flag beside it. Two starts chained through one
//! composer would spend one goal on two conversations and leave the first
//! unfinished, which is upstream's own finding (yog's `docs/DESIGN.md` §3.4).
//! Here it costs nothing to prevent: while a start is outstanding the composer
//! paints the sentence instead of the box, so there is no control to press.

/// The n-candidate spread: the obligation a fan is about, and the two acts a
/// held one composes on the way through.
mod spread;
pub use spread::Spread;

use crate::reply::start::Prepared;
use crate::ui::Model;

/// What the window holds while a start is happening, and after it happened.
#[derive(Debug, Clone, PartialEq)]
pub struct Start {
    /// The workspace it is being started in, **in this box's spelling** — held
    /// rather than read off the aim at fire time, because an operator may aim
    /// somewhere else while the first act is in flight and a fire that followed
    /// the aim would prompt a workspace nothing staged.
    pub address: String,
    /// What the operator typed. It is held here rather than left in the draft
    /// so that the text has a painted representation for the whole of the round
    /// trip: a start that shows nothing between Enter and the engine's answer
    /// reads as a window that did nothing.
    pub goal: String,
    /// How far along it is.
    pub phase: Phase,
    /// **The obligation this start is a SPREAD over**, where it is one
    /// (§4.36). A fan is not a second start path: it is this start with n in
    /// the middle — `prepare`, then `fan` instead of `prompt`, then one
    /// ordinary `prompt` per candidate that comes back. So it is held here
    /// rather than beside, and every rule this value already carries applies
    /// to it unchanged: one is outstanding at a time (two `prepare` receipts
    /// cannot be told apart — the reply carries no correlation), a refusal
    /// retires it with the goal back in the box, and an unread answer takes it
    /// back.
    pub spread: Option<Spread>,
}

/// The three states of a start, which are two acts and a receipt.
#[derive(Debug, Clone, PartialEq)]
pub enum Phase {
    /// The staging act is out; nothing has answered it yet.
    Staging,
    /// The fire is out, carrying the staged body and the goal.
    Firing,
    /// The engine minted this name for the conversation it started.
    Started(String),
    /// The engine refused one of the two acts, in its own words — and the goal
    /// went back to the box, because a refused fire spends nothing (yog's
    /// `docs/DESIGN.md` §8.1: *sign in first* is the one this ball exists for,
    /// and a blank goal and the spend ceiling ride the same door).
    Refused(String),
}

impl Start {
    /// The line the composer paints.
    ///
    /// **The two acts read the same**, deliberately: which of the two envelopes
    /// is currently out is this seat's business, not a fact about the
    /// operator's conversation. What changes the sentence is the *receipt*,
    /// because the minted name is a thing the operator can use.
    pub fn line(&self) -> String {
        match &self.phase {
            Phase::Started(name) if self.spread.is_some() => {
                format!("one candidate of «{name}» in {}", self.address)
            }
            Phase::Started(name) => format!("started «{name}» in {}", self.address),
            Phase::Refused(why) => format!("not started in {}: {why}", self.address),
            Phase::Staging | Phase::Firing => {
                format!("starting in {}: {}…", self.address, self.goal)
            }
        }
    }

    /// Whether a start is still in flight — the gate on composing a second.
    pub fn outstanding(&self) -> bool {
        !matches!(self.phase, Phase::Started(_) | Phase::Refused(_))
    }
}

impl Model {
    /// **Stage a start**: compose the first act and hold the draft as the goal
    /// the second will carry.
    ///
    /// An empty draft composes nothing — a conversation nobody said anything to
    /// is a driver launched for nothing, which is upstream's own *a blank goal
    /// never sends* — and the draft is taken only where something was actually
    /// composed, so a mis-click never costs what was typed. Both are the
    /// deposit's own rules, spelled once each because the two acts share a box.
    pub fn stage(&mut self, address: &str) {
        if self.draft.trim().is_empty() {
            return;
        }
        let goal = std::mem::take(&mut self.draft);
        self.outbox.push(super::Posted::act(crate::verbs::prepare(
            address.to_owned(),
        )));
        self.start = Some(Start {
            address: address.to_owned(),
            goal,
            phase: Phase::Staging,
            spread: None,
        });
    }

    /// **The second act**, composed by the frame that took the first's receipt.
    ///
    /// A staged body this window did not stage is not dropped — nothing that
    /// arrives is — it is fired on its own terms: addressed by the name it came
    /// back under, which is right wherever no entry renames, and carrying the
    /// goal its own rung composed. The bare rung prefills nothing, so that goal
    /// is empty and the fire below never happens, which is the same predicate
    /// [`stage`](Self::stage) spends.
    pub(super) fn fire(&mut self, prepared: &Prepared) {
        let held = self
            .start
            .clone()
            .filter(|start| matches!(start.phase, Phase::Staging));
        let (address, goal) = held.map_or_else(
            || (prepared.workspace.clone(), prepared.goal.clone()),
            |start| (start.address, start.goal),
        );
        if goal.trim().is_empty() {
            self.start = None;
            return;
        }
        self.outbox.push(super::Posted::act(crate::verbs::prompt(
            prepared,
            address.clone(),
            goal.clone(),
        )));
        self.start = Some(Start {
            address,
            goal,
            phase: Phase::Firing,
            spread: None,
        });
    }

    /// **Whether `op` is one of the start's two acts and a start is out on
    /// it** — the one question a receipt that is not an answer has to ask
    /// before it can be filed (`super::absorb`). Any other op's refusal is
    /// somebody else's sentence and leaves the start where it is.
    pub(super) fn starting(&self, op: &str) -> bool {
        (op == crate::verbs::PREPARE || op == crate::verbs::PROMPT || op == crate::verbs::FAN)
            && self.start.as_ref().is_some_and(Start::outstanding)
    }

    /// **The engine refused one of the two acts.** The sentence stands where
    /// the start's own did, and the box comes back under it with the goal in
    /// it: a refused fire spends nothing, so what was typed is the operator's
    /// again. The refund goes to the draft only where the draft is empty — it
    /// is, whenever the composer has been standing down for this start, and a
    /// draft typed elsewhere in the meantime is not this start's to overwrite.
    pub(super) fn refuse_start(&mut self, why: String) {
        let Some(mut start) = self.start.take() else {
            return;
        };
        self.refund(&start.goal);
        start.phase = Phase::Refused(why);
        self.start = Some(start);
    }

    /// **A start whose act earned no answer this seat can read** — a frame it
    /// cannot paint, or none at all (REMOTE §3's IN DOUBT). Nothing is held:
    /// the sentence about *that* is the bar's, and the goal is the operator's
    /// again, because a start that stays outstanding forever is a composer
    /// with no box.
    pub(super) fn take_back_start(&mut self) {
        if let Some(start) = self.start.take() {
            self.refund(&start.goal);
        }
    }

    fn refund(&mut self, goal: &str) {
        if self.draft.trim().is_empty() {
            goal.clone_into(&mut self.draft);
        }
    }

    /// **The minted name**, which is what the whole flow was for.
    ///
    /// A receipt for a start this window did not fire still paints its name,
    /// against the wall the window is aimed at: it is the one fact the engine
    /// added, and a seat that dropped it would have started a conversation and
    /// said nothing about it.
    ///
    /// **And the name is selected** — *a start focuses what it started* — but
    /// only where the window is still aimed at the wall it was started on. The
    /// selection is painted in the aimed wall's list and nowhere else, so a
    /// selection on another wall is one the operator can neither see nor leave;
    /// what the claim then amounts to, and what spends it, is
    /// [`super::claim`].
    pub(super) fn started(&mut self, conversation: String) {
        let mut start = self.start.clone().unwrap_or_else(|| Start {
            address: self
                .aim
                .as_ref()
                .map_or_else(String::new, |aim| aim.address.clone()),
            goal: String::new(),
            phase: Phase::Firing,
            spread: None,
        });
        start.phase = Phase::Started(conversation.clone());
        // **A spread selects nothing** (§4.36). *A start focuses what it
        // started* is a sentence about ONE conversation; n receipts land one
        // after another and the focus would be whichever arrived last, which
        // is a fact about the network. The fleet pane is what a fan is read
        // on, and it covers the conversation anyway.
        let here = start.spread.is_none()
            && self
                .aim
                .as_ref()
                .is_some_and(|aim| aim.address == start.address);
        self.start = Some(start);
        if here {
            self.select(&conversation);
        }
    }
}

#[cfg(test)]
mod tests;
