//! **The login pane between frames** (bl-e3c5; DESIGN §4.24) — what it is
//! asking about, and the three acts its controls spend.
//!
//! # Two subjects, and neither excludes the other
//!
//! `super::tuning` is an enum because its two states are one pane's two modes:
//! showing rows, or rewriting one assignment, and *editing while closed* is a
//! state that means nothing. This pane is not that shape. **A sign-in being
//! followed and a row that was asked what it offers are two independent facts
//! about two possibly different rows**, and every combination of them is a
//! window an operator really reaches — so [`Login`] is a struct of two options
//! rather than an enum, and the state that would mean nothing (a subject with
//! no pane) is unrepresentable because both live *inside* the pane's own
//! option.
//!
//! # What is NOT held here is any answer
//!
//! [`Model::providers`], [`Model::offered`] and [`Model::signin`] are the
//! engine's, filed on the model beside the roles for `super::tuning`'s reason
//! verbatim: a pane that wrote its own row back would be painting a claim the
//! engine had not made, and a sign-in is exactly the act whose outcome this end
//! cannot predict.
//!
//! # One read stands, one is posted, and one is held
//!
//! All three of DESIGN's cadences meet on this pane, which is why each is
//! stated. **`providers` STANDS while the pane is open** (§4.17's rule): a
//! credential lands on the engine while the operator is looking at the table,
//! so the row that said *no credential* has to say otherwise on the next beat
//! without anybody asking again. **`models` is POSTED** (§4.21's rule): what a
//! row offers is fixed for the life of the engine's own answer, and a standing
//! read would spend a round trip a beat forever on something that cannot change
//! under the operator. **`login-tail` is HELD** on its own thread
//! (`crate::offframe::signin`), because it is answered at the provider's pace.

use super::{Aim, Model};

/// The login pane, while it is open.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Login {
    /// **Which row a sign-in was started on**, and therefore whose run the
    /// lane is following — or `None` where the pane stands on the table alone.
    ///
    /// It is the lane's whole subject (`crate::state::Standing::signin`), so
    /// closing the pane ends the lane. It terminates nothing: the run is
    /// engine-owned and bounded there (REMOTE §8.3), which is a property this
    /// side gets for free and must not re-implement.
    pub following: Option<String>,
    /// **Which row was asked what it offers.** The question rather than the
    /// answer — the answer is [`Model::offered`] — so the pane can say it is
    /// waiting under the row that asked instead of under all of them.
    pub asking: Option<String>,
}

impl Model {
    /// **Open the login pane on the wall the window is aimed at**, or do
    /// nothing where it is aimed at none.
    ///
    /// The aim is the gate rather than a separate check, exactly as it is for
    /// the tuning pane: every gesture this pane composes carries a workspace,
    /// and a workspace is what an aim is.
    pub fn begin_login(&mut self) {
        if self.aim.is_some() {
            self.login = Some(Login::default());
        }
    }

    /// **Close it.** The answers stay, for the reason the roles do: the next
    /// open on the same wall is about the same table, and the standing read
    /// replaces it anyway. What does end is the lane — its subject is this
    /// pane's `following`, and there is none once the pane is down.
    pub fn close_login(&mut self) {
        self.login = None;
    }

    /// **The row whose sign-in this seat is following**, if any. The lane's
    /// subject and the pane's own question, asked once so the two cannot
    /// disagree.
    pub fn following(&self) -> Option<String> {
        self.login.as_ref()?.following.clone()
    }

    /// **Start a sign-in on one row**, and follow it.
    ///
    /// The fold is dropped in the same act: a second sign-in is a different
    /// run — upstream terminates and replaces the first — so keeping the old
    /// lines under the new row's name would paint one run's output as
    /// another's. What replaces it is the act's own receipt, which upstream
    /// answers at once with the run's standing.
    pub fn post_signin(&mut self, provider: &str) {
        let Some(pane) = self.login.as_mut() else {
            return;
        };
        pane.following = Some(provider.to_owned());
        self.signin = None;
        self.aimed_act(&crate::verbs::login, provider);
    }

    /// **Ask one row what it offers**, dropping whatever the last row answered.
    ///
    /// A read rather than an act ([`super::Posted::read`]): asking twice is
    /// asking once, so a lost reply is re-asked by clicking again rather than
    /// leaving anything in doubt.
    pub fn post_offering(&mut self, provider: &str) {
        let Some(pane) = self.login.as_mut() else {
            return;
        };
        pane.asking = Some(provider.to_owned());
        self.offered = None;
        let Some(Aim { address, .. }) = self.aim.clone() else {
            return;
        };
        self.outbox.push(super::Posted::read(crate::verbs::models(
            address,
            provider.to_owned(),
        )));
    }

    /// **Compose one act against the aimed wall and one provider row**, or none
    /// at all — the aim being the same gate [`begin_login`](Self::begin_login)
    /// enforces, made unreachable here rather than merely unlikely.
    ///
    /// `&dyn Fn` rather than a generic, for `super::tuning::Model::tune`'s
    /// reason: a bound would stamp one copy of this body out per call site.
    fn aimed_act(&mut self, gesture: &dyn Fn(String, String) -> serde_json::Value, provider: &str) {
        let Some(Aim { address, .. }) = self.aim.clone() else {
            return;
        };
        self.outbox
            .push(super::Posted::act(gesture(address, provider.to_owned())));
    }

    /// **Whether the aimed wall is held on another box** — read off the
    /// roster's own client-side channel stamp and never off an address, which
    /// is what keeps a host out of the pane's sentences.
    ///
    /// A channel with a name on its host is an entry (§8.2); this box's own
    /// engine has none. A channel this seat no longer holds answers `false`,
    /// which is the honest reading: there is nothing to say *elsewhere* about.
    pub fn elsewhere(&self) -> bool {
        let Some(aim) = self.aim.as_ref() else {
            return false;
        };
        self.roster
            .iter()
            .find(|chunk| chunk.channel.name == aim.channel)
            .is_some_and(|chunk| chunk.channel.named_there.is_some())
    }

    /// **The pane and its three answers go with the wall they are about** —
    /// called by the act that moves the aim, so nothing on it outlives its
    /// subject. It is `super::records::retire_records`'s rule one noun over.
    pub(super) fn retire_login(&mut self) {
        self.login = None;
        self.providers = None;
        self.offered = None;
        self.signin = None;
    }
}

#[cfg(test)]
mod tests;
