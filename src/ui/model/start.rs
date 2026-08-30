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
            Phase::Started(name) => format!("started «{name}» in {}", self.address),
            Phase::Staging | Phase::Firing => {
                format!("starting in {}: {}…", self.address, self.goal)
            }
        }
    }

    /// Whether a start is still in flight — the gate on composing a second.
    pub fn outstanding(&self) -> bool {
        !matches!(self.phase, Phase::Started(_))
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
        self.outbox.push(crate::verbs::prepare(address.to_owned()));
        self.start = Some(Start {
            address: address.to_owned(),
            goal,
            phase: Phase::Staging,
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
        self.outbox.push(crate::verbs::prompt(
            prepared,
            address.clone(),
            goal.clone(),
        ));
        self.start = Some(Start {
            address,
            goal,
            phase: Phase::Firing,
        });
    }

    /// **The minted name**, which is what the whole flow was for.
    ///
    /// A receipt for a start this window did not fire still paints its name,
    /// against the wall the window is aimed at: it is the one fact the engine
    /// added, and a seat that dropped it would have started a conversation and
    /// said nothing about it.
    pub(super) fn started(&mut self, conversation: String) {
        let mut start = self.start.clone().unwrap_or_else(|| Start {
            address: self
                .aim
                .as_ref()
                .map_or_else(String::new, |aim| aim.address.clone()),
            goal: String::new(),
            phase: Phase::Firing,
        });
        start.phase = Phase::Started(conversation);
        self.start = Some(start);
    }
}

#[cfg(test)]
mod tests;
