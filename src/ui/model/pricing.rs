//! **The prices pane between frames** (REMOTE §9.23; DESIGN §4.41; bl-9111):
//! what each channel last said its table is, and the draft of the two acts.
//!
//! # The window-level family's shape, one noun over
//!
//! `prices` names no workspace, so its subject is every channel this box holds
//! (§4.21's mechanical definition). It opens from an unaimed seat, it survives
//! an aim and a selection, and one channel's answer replaces its own section
//! and leaves every other standing. It shares [`Lookup`] with the rest of that
//! family because *which channel-wide pane is standing* is one fact.
//!
//! # The read is POSTED, and the acts are what re-read it
//!
//! A table is written by an operator, and both acts that write it answer with
//! it re-derived — so the section an act fired from is replaced by its own
//! receipt, and a standing read would spend a round trip a beat on an answer
//! that only moves when somebody here moves it. The spend line is the one
//! fact that drifts under the operator; opening the pane again re-asks.
//!
//! # The acts are addressed down ONE channel, never fanned
//!
//! Neither act names a workspace, so the poster would fan either over every
//! engine this box is a client of (§4.30's hazard). Each is fired from one
//! channel's section and addressed down that channel, which is what the
//! operator was looking at.
//!
//! # The provider is checked against the roster this seat already holds
//!
//! Where the window is aimed at a wall on the section's channel and that wall
//! has answered `providers`, a provider it does not name disables the control
//! with the reason beside it — the engine would refuse it in band, and the
//! seat already holds the roster that says so. Anywhere else the engine's own
//! refusal names it. Opening the pane asks the aimed wall's roster for this.

use super::{Lookup, Model, Posted};
use crate::reply::prices::Prices;
use crate::ui::Channel;
use crate::verbs::prices::{bound, rates};

/// **One channel's table**, stamped with the channel it came down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub channel: Channel,
    pub prices: Prices,
}

/// **The pane's whole holding**: each channel's table, and the four boxes the
/// two acts are composed from. One field on the model rather than five.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pricing {
    pub tables: Vec<Table>,
    /// The row's provider, which the roster check reads.
    pub provider: String,
    /// The row's model, or `*`.
    pub model: String,
    /// Two to four rates, space-separated, as `lernie price` takes them.
    pub rates: String,
    /// The bound in USD.
    pub bound: String,
}

/// What the price control says while no row is named.
pub const NEEDS_ROW: &str = "name a provider and a model to price";
/// What the ceiling control says while its box is empty.
pub const NEEDS_BOUND: &str = "type a ceiling in USD, or off, to set one";

impl Model {
    /// **Open the pane and ask for every channel's table** — and the aimed
    /// wall's provider roster, which is what the price control checks against.
    pub fn begin_prices(&mut self) {
        self.lookup = Some(Lookup::Pricing);
        self.outbox
            .push(Posted::read(crate::verbs::prices::prices()));
        if let Some(aim) = &self.aim {
            self.outbox
                .push(Posted::read(crate::verbs::providers(aim.address.clone())));
        }
    }

    /// **Whether the prices pane is the one standing.**
    pub fn pricing(&self) -> bool {
        self.lookup == Some(Lookup::Pricing)
    }

    /// File one channel's table, replacing that channel's section only.
    pub(super) fn priced(&mut self, channel: &Channel, prices: Prices) {
        let answered = Table {
            channel: channel.clone(),
            prices,
        };
        let tables = &mut self.pricing.tables;
        match tables
            .iter_mut()
            .find(|held| held.channel.name == channel.name)
        {
            Some(held) => *held = answered,
            None => tables.push(answered),
        }
    }

    /// **Why the price control is not live on that channel**, or `None` where
    /// it is — the enablement rule (DESIGN §4.20): the parameter is missing,
    /// so the control stays on the glass saying what would fill it.
    pub fn unpriceable(&self, channel: &Channel) -> Option<String> {
        let draft = &self.pricing;
        if draft.provider.trim().is_empty() || draft.model.trim().is_empty() {
            return Some(NEEDS_ROW.to_owned());
        }
        let words: Vec<&str> = draft.rates.split_whitespace().collect();
        if let Err(why) = rates(&words) {
            return Some(why);
        }
        let aim = self
            .aim
            .as_ref()
            .filter(|aim| aim.channel == channel.name)?;
        let roster = self.providers.as_ref()?;
        let provider = draft.provider.trim();
        (!roster.iter().any(|row| row.name == provider)).then(|| {
            format!(
                "{} has no provider row {provider:?} — the engine would refuse it",
                aim.address
            )
        })
    }

    /// **Price the drafted row down that channel**, or do nothing where the
    /// control is not live. The boxes are kept: a second row is usually the
    /// same provider.
    pub fn post_price(&mut self, channel: &Channel) {
        if self.unpriceable(channel).is_some() {
            return;
        }
        let draft = &self.pricing;
        let words: Vec<&str> = draft.rates.split_whitespace().collect();
        let envelope = crate::verbs::price(
            draft.provider.trim().to_owned(),
            draft.model.trim().to_owned(),
            rates(&words).unwrap_or_default(),
        );
        self.outbox
            .push(Posted::act(envelope).down(channel.clone()));
    }

    /// **Delete one row** — the `off` its own control carries.
    pub fn post_unprice(&mut self, channel: &Channel, provider: &str, model: &str) {
        let envelope = crate::verbs::price(provider.to_owned(), model.to_owned(), None);
        self.outbox
            .push(Posted::act(envelope).down(channel.clone()));
    }

    /// **Why the ceiling control is not live**, or `None` where it is.
    pub fn unbounded(&self) -> Option<String> {
        let typed = self.pricing.bound.trim();
        if typed.is_empty() {
            return Some(NEEDS_BOUND.to_owned());
        }
        bound(typed).err()
    }

    /// **Set the bound down that channel**, or lift it where the box says
    /// `off` — one control, because the box's own word is the act.
    pub fn post_ceiling(&mut self, channel: &Channel) {
        let Ok(usd) = bound(self.pricing.bound.trim()) else {
            return;
        };
        self.outbox
            .push(Posted::act(crate::verbs::ceiling(usd)).down(channel.clone()));
    }

    /// **Lift the bound** — the control beside a section that has one.
    pub fn post_lift(&mut self, channel: &Channel) {
        self.outbox
            .push(Posted::act(crate::verbs::ceiling(None)).down(channel.clone()));
    }
}

#[cfg(test)]
mod tests;
