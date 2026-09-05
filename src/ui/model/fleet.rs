//! **The fleet pane between frames** (bl-a43a): the three words its five acts
//! are composed from, the last receipt in the op's own name, and the two reads
//! that stand while it is open.
//!
//! # It is the tuning pane's shape, and every one of its seven ops names a wall
//!
//! `fleet`, `disband`, `arm`, `disarm`, `scan`, `science` and `work-diff` all
//! carry a workspace, so the pane's subject is the **aimed wall**: it opens on
//! a row, its reads are asked of that wall while it stands, and it is retired
//! when the aim moves (`super::tuning`, and `Model::aim_at`). Nothing about it
//! is the queue's shape, which is what tells it from the ball pane next door —
//! that one is two widths and this one is one.
//!
//! # It holds three words because three of its acts carry one
//!
//! A cap is a number, a project is a name and a model is a name; none of the
//! three is derivable from anything on the glass, so each is a box, and the
//! control that spends one stands down until it has something to spend
//! (§4.20's enablement rule: the parameter is missing, not the subject).
//!
//! **The boxes are bound to these fields and never to a copy** (`crate::ui::
//! fleet`, and the tuning editor before it): a draft is what the pane holds,
//! so an edit is a write to it and there is no second value for the two to
//! disagree about.
//!
//! # The receipt is read by the OP and never by the reply
//!
//! `fleet`, `disband`, `arm` and `disarm` answer with the same `armed` kind,
//! and the two families it spans are the fleet loop and the alignment monitor.
//! So [`Armed`] carries the op the poster stamped the frame with
//! (`crate::state::Said::Receipt`) and the flag the engine sent, and the pane
//! says both. A seat that read *which family* off the reply would be guessing
//! between two settings on two carriers.
//!
//! **Nothing here holds the loop's STATE**, and that is deliberate: whether a
//! loop is running is on the `board` answer, in the ball pane (DESIGN §4.31).
//! A second copy of it here would be a second authority, and the receipt below
//! is an event rather than a state — what happened when the operator pressed
//! the control, said once.

use super::{Aim, Model};
use crate::reply::diff::Diff;
use crate::reply::science::Attempt;

/// **The fleet pane, while it is open**: the words its acts are composed from,
/// and the last thing an act answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fleet {
    /// **The wall it was opened on**, held rather than followed — the
    /// unmaking's arrangement (§4.20), and for a sharper form of its reason:
    /// these acts start drones and spend money in one workspace, so a gesture
    /// composed against whatever the aim happens to be at the moment of the
    /// click would be a control that moved under the operator. The pane is
    /// retired when the aim moves, so the two can never disagree — and holding
    /// it is what makes that a fact rather than a promise.
    pub at: Aim,
    /// Whose ready balls the loop should run. `fleet` requires it and nothing
    /// on this pane can derive it, so it is typed.
    pub project: String,
    /// How many at once. A number, which is why `fleet` is a door and not a
    /// row (`crate::verbs::fleet`).
    pub cap: u64,
    /// The cheap model the monitor is pinned to.
    pub model: String,
    /// **How many candidates a spread asks for** (§4.36). A stepper for the
    /// cap's reason and floored at two for a sharper one: upstream reads 1 and
    /// 0 as *materialize nothing and hand back the ordinary claim binding*,
    /// which is a start and not a fan.
    pub spread: u64,
    /// What each of those candidates is for. A start with no goal is a driver
    /// launched for nothing, and n of them is n.
    pub goal: String,
    /// The delivery subject one acceptance carries, verbatim — balls tags it
    /// with the handle, and that tag is the only acceptance mark there is.
    pub summary: String,
    /// What the last of the four standing acts answered.
    pub said: Option<Armed>,
}

/// **One receipt, in the op's own name.** The op is the poster's stamp and the
/// flag is the engine's; this seat joins them and classifies neither.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Armed {
    pub op: String,
    pub armed: bool,
}

impl Model {
    /// **Open the fleet pane on the aimed wall**, or do nothing with nothing
    /// aimed at: every one of its seven ops carries a workspace, so a pane
    /// opened unaimed would be a screen of controls with nowhere to fire.
    ///
    /// The cap opens at one rather than at zero. Zero is a loop that spawns
    /// nothing and still reaps, which upstream refuses to spell as a cap at
    /// all — `disband` is that — so it is not a value this box should be able
    /// to send.
    pub fn begin_fleet(&mut self) {
        if let Some(at) = self.aim.clone() {
            self.fleet = Some(Fleet {
                at,
                project: String::new(),
                cap: 1,
                model: String::new(),
                spread: 2,
                goal: String::new(),
                summary: String::new(),
                said: None,
            });
        }
    }

    /// **Close it.** The two reads' answers stay, for the reason the roles do:
    /// the next open is about the same wall, and the standing read replaces
    /// them anyway.
    pub fn close_fleet(&mut self) {
        self.fleet = None;
    }

    /// **The pane and its two answers go with the wall they are about** —
    /// called by the act that moves the aim.
    pub(super) fn retire_fleet(&mut self) {
        self.fleet = None;
        self.attempts = None;
        self.work = None;
    }

    /// **Spend one of the five acts**, composed against the aimed wall. It is
    /// one door because all five are the same gesture with a different verb:
    /// the caller names the envelope, and the aim is read here so no control
    /// holds a second opinion about which wall it is firing at.
    pub fn post_fleet(&mut self, envelope: serde_json::Value) {
        self.outbox.push(super::Posted::act(envelope));
    }

    /// **Spend one of the candidate acts**, ADDRESSED down the channel the
    /// pane stands on (§4.30's ruling, §4.36's first customer).
    ///
    /// `deliver` and `retire` name no workspace anywhere in their envelopes —
    /// their subject is a ball in a project on one engine — so the poster
    /// would otherwise fan them over every channel this box holds, accepting
    /// one candidate on every engine the operator is a client of. A seat that
    /// no longer holds the channel composes nothing, which is the honest
    /// reading: there is nothing left to address.
    pub fn post_candidate(&mut self, envelope: serde_json::Value) {
        let Some(down) = self.channel() else {
            return;
        };
        self.outbox.push(super::Posted::act(envelope).down(down));
    }

    /// File the receipt four ops share, under the op that earned it.
    pub(super) fn armed(&mut self, op: &str, armed: bool) {
        if let Some(fleet) = self.fleet.as_mut() {
            fleet.said = Some(Armed {
                op: op.to_owned(),
                armed,
            });
        }
    }

    /// File the attempts.
    pub(super) fn scienced(&mut self, rows: Vec<Attempt>) {
        self.attempts = Some(rows);
    }

    /// File what changed.
    pub(super) fn worked(&mut self, rows: Vec<Diff>) {
        self.work = Some(rows);
    }
}

#[cfg(test)]
mod tests;
