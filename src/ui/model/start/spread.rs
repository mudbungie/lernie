//! **A start spread over n candidates** (§4.36; yog's VISION §4.10) — the
//! obligation a fan is about, and the two acts a held one composes.
//!
//! Split from [`super`] at the design-time budget on the seam the subject has:
//! [`super`] is *a start between its two acts*, and this is *what changes when
//! there are n of them*. Nothing here is a second start path — the held
//! [`super::Start`] is the same value, the same gate and the same refusal
//! ladder, and what a spread changes is exactly two things: the fire composes
//! `fan` rather than `prompt`, and the answer to it composes n `prompt`s.
//!
//! # Why it is the frame that fires them and not a control
//!
//! §4.26's argument, read over n. The second act of a start belongs to the
//! frame that absorbs the first's receipt, because a frame composes into the
//! outbox and never waits on a socket. A fan sharpens it: `fan` MATERIALIZES n
//! attempt worktrees, so a candidate prepared and never fired is a worktree
//! balls made for nothing. Firing them is the completion of the act rather
//! than a convenience.
//!
//! **What that forfeits is stated rather than hidden**: upstream's terminal
//! fires each candidate itself, *"with whatever variation you want between
//! them"*. This window fires n with one goal. Per-candidate variation is a
//! surface — n boxes, or one edited n times — and it arrives with the ball
//! that builds it; what it is not is a reason to leave n worktrees empty.
//!
//! # The address is the pane's, and both hops need it for different reasons
//!
//! `fan` carries its workspace inside the prepared body (`crate::envelope`),
//! so it routes like any addressed gesture. The `prompt`s do too. What neither
//! can do is name the ENGINE without a workspace — which is why `deliver` and
//! `retire` are addressed down a channel instead (DESIGN §4.30's ruling, one
//! pane over).

use super::{Phase, Start};
use crate::reply::start::Prepared;
use crate::ui::Model;

/// **What a fan is about**: the delivery obligation, and how many attempts.
///
/// The obligation is both halves or neither — upstream's own shape, because a
/// target is a ball's `work/<id>` ref *in* a project — and both come off the
/// work-diff row the control fired from, never off a box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spread {
    pub ball: String,
    pub project: String,
    /// How many isolated attempts. One or zero materializes nothing and hands
    /// back the ordinary claim binding, which is upstream's own reading and
    /// the reason the pane's stepper never goes below two.
    pub n: u64,
}

impl Model {
    /// **Stage a spread**: the start's own first act, held with the obligation
    /// the fire will carry.
    ///
    /// It refuses while a start is outstanding, which is [`Start`]'s existing
    /// gate spent on a second subject: two `prepare` acts in flight cannot be
    /// told apart, because the receipt carries no correlation with the gesture
    /// that earned it. An empty goal composes nothing, for `Model::stage`'s
    /// reason verbatim — n conversations nobody said anything to is n drivers
    /// launched for nothing.
    pub fn stage_spread(&mut self, address: &str, spread: Spread, goal: &str) {
        if goal.trim().is_empty() || self.start.as_ref().is_some_and(Start::outstanding) {
            return;
        }
        self.outbox
            .push(super::super::Posted::act(crate::verbs::prepare(
                address.to_owned(),
            )));
        self.start = Some(Start {
            address: address.to_owned(),
            goal: goal.to_owned(),
            phase: Phase::Staging,
            spread: Some(spread),
        });
    }

    /// **The staging receipt**, which is a fan's second act or a start's,
    /// decided by the held value rather than by the reply — the reply is the
    /// same `prepared` either way, which is the whole reason the intent is
    /// held at all.
    pub(in crate::ui::model) fn staged(&mut self, prepared: &Prepared) {
        let held = self
            .start
            .clone()
            .filter(|start| matches!(start.phase, Phase::Staging));
        let Some((start, spread)) = held.and_then(|start| {
            let spread = start.spread.clone()?;
            Some((start, spread))
        }) else {
            self.fire(prepared);
            return;
        };
        self.outbox
            .push(super::super::Posted::act(crate::verbs::fan(
                prepared,
                start.address.clone(),
                spread.ball.clone(),
                spread.project.clone(),
                spread.n,
            )));
        self.start = Some(Start {
            phase: Phase::Firing,
            ..start
        });
    }

    /// **The spread's own receipt**: one staged body per candidate, each fired
    /// with the one goal the operator typed.
    ///
    /// A `fanned` answer this window did not ask for still fires — nothing
    /// that arrives is dropped ([`Model::fire`]'s own rule) — against the wall
    /// each candidate came back naming, with the candidate's own goal. That is
    /// empty on the bare rung, so the ordinary empty-goal guard makes it a
    /// no-op rather than n conversations about nothing.
    pub(in crate::ui::model) fn fanned(&mut self, rows: Vec<Prepared>) {
        let held = self.start.clone().filter(|start| start.spread.is_some());
        let (address, goal) = held.map_or_else(
            || (String::new(), String::new()),
            |start| (start.address, start.goal),
        );
        for row in &rows {
            let to = if address.is_empty() {
                row.workspace.clone()
            } else {
                address.clone()
            };
            let said = if goal.trim().is_empty() {
                row.goal.clone()
            } else {
                goal.clone()
            };
            if said.trim().is_empty() {
                continue;
            }
            self.outbox
                .push(super::super::Posted::act(crate::verbs::prompt(
                    row, to, said,
                )));
        }
    }
}

#[cfg(test)]
mod tests;
