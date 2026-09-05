//! **The ball pane's five acts between frames** (bl-f7ae; DESIGN §4.35): what
//! the model does with each — open the block, close it, send what it composed
//! — and the name every one of them stamps.
//!
//! Split from [`super`] at the design-time budget on the seam the pane itself
//! draws: that module is *what the four reads last said*, and this is *what
//! the operator would do about it*. The first changes when a kind lands, the
//! second when a control does. [`super::block`] is the value these doors are
//! about, split from here on the config pane's own seam.
//!
//! # The `--as` name is the wall's, so this seat needs no identity
//!
//! All five acts carry a `name`, and the ball that filed this asked where a
//! seat-side identity would come from. It does not come from anywhere, because
//! it is not one: yog spells the field *"the ball's bound workspace name,
//! never the operator `$USER`"*, and its own join binds a ball to a workspace
//! on exactly that equality (`projects::join::owner_name`). So the stamp is
//! the aimed wall's name **as its engine spells it** ([`Model::stamp`]), and a
//! seat that invented an operator name would break the binding it was making.
//!
//! It is read off the channel rather than off the aim because the two differ
//! exactly where an entry renames: `crate::seat::route` rewrites a
//! `workspace` FIELD at the channel boundary and there is no such field on any
//! of these five, so the leaf this box knows a wall by would cross as a
//! claimant nobody on that engine is.
//!
//! # None of the five is fanned, and none of them routes
//!
//! Each names a project and no workspace, so `crate::seat::route` has nothing
//! to resolve and the poster's own rule would fan it — one click filing the
//! same ball on every engine this box dials. Every control here hangs on a row
//! that came down ONE channel, so each says which
//! (`crate::ui::Posted::down`), which is the address bl-4855 already gave a
//! `config` write for the same reason.
//!
//! # The block holds its subject rather than following the aim
//!
//! §4.20's rule, and the fleet pane's before it: the roster stays live under
//! a covering pane, so a block that re-read the aim when it fired would amend
//! a ball on a wall the operator opened it for a different one. It is retired
//! with the aim ([`Model::retire_wall_balls`](super::super::Model)), so the
//! two can never disagree — and holding it is what makes that a fact rather
//! than a promise.

use serde_json::Value;

use super::super::Model;
use super::block::Authoring;
use crate::reply::balls::BoundBall;
use crate::reply::board::BoardRow;
use crate::ui::Channel;

impl Model {
    /// **The name the aimed wall's engine knows it by** — the `--as` stamp
    /// every act here carries, and `None` where nothing is aimed at.
    ///
    /// An entry renames, and the rename is spent at the channel boundary on a
    /// `workspace` field none of these five has; so the host's own spelling is
    /// read off the channel, and a channel that does not rename is one whose
    /// rows already arrive in it.
    pub fn stamp(&self) -> Option<String> {
        let aim = self.aim.clone()?;
        Some(self.channel()?.named_there.unwrap_or(aim.address))
    }

    /// **Open the block on a ball that does not exist yet.** Nothing but an
    /// aim is needed: the wall is what supplies the name, and the project is
    /// typed.
    pub fn begin_filing(&mut self) {
        if let (Some(at), Some(name)) = (self.aim.clone(), self.stamp()) {
            self.authoring = Some(Authoring::of(at, name, None));
        }
    }

    /// **Open it on a ball this wall holds.**
    pub fn begin_amending(&mut self, ball: &BoundBall) {
        if let (Some(at), Some(name)) = (self.aim.clone(), self.stamp()) {
            self.authoring = Some(Authoring::of(at, name, Some(ball.clone())));
        }
    }

    /// **Close it.** The way out, which changes nothing.
    pub fn close_authoring(&mut self) {
        self.authoring = None;
    }

    /// **Spend one of the acts**, down the channel the pane was opened on.
    ///
    /// One door because all five are the same gesture with a different verb:
    /// the caller names the envelope and the channel, so no control holds a
    /// second opinion about where its row came from. It composes and does not
    /// close the block — what says the act landed is the pane's own standing
    /// reads answering again, and a block that vanished on firing would take
    /// the operator's other half-typed field with it.
    pub fn post_ball(&mut self, down: &Channel, envelope: Value) {
        self.outbox
            .push(super::super::Posted::act(envelope).down(down.clone()));
    }

    /// **What claiming `row` for the aimed wall would be**, or `None` where
    /// this seat cannot compose it.
    ///
    /// Two refusals and each is a fact rather than a policy. A ball somebody
    /// already holds is not one to claim, which is read off the row's own
    /// claimant and never off the column's word (`crate::reply`'s rung 3: a
    /// column this build has not heard of is still painted). And a ball on one
    /// engine cannot be claimed by a wall on another, so the section's channel
    /// must be the aim's — the ball and the workspace are one engine's two
    /// facts, joined there and nowhere else.
    pub fn claiming(&self, down: &Channel, row: &BoardRow) -> Option<Value> {
        let aim = self.aim.as_ref()?;
        if row.claimant.is_some() || aim.channel != down.name {
            return None;
        }
        Some(crate::verbs::assign(
            row.project.clone(),
            row.id.clone(),
            self.stamp()?,
        ))
    }
}

#[cfg(test)]
mod tests;
