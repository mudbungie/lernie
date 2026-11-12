//! **The offers page, between frames** (DESIGN §4.51): the form an offer is
//! minted from, the offer last minted, and the acts the page's rows spend.
//!
//! The page is a [`Listing`] for the clients pane's reason: its rows are one
//! workspace's pending offers and their state moves on the server while the
//! operator watches — open, claimed, proved — so the read stands while the
//! page does (`crate::state::traffic::Open::Offers`) and is put down with the
//! aim. Nothing here is a secret: the URL is a one-time code and the word is
//! a check, so the page is an ordinary page and not the whole-window cover
//! the enrollment it replaces had to be.

use crate::qr::Symbol;
use crate::reply::offers::Offered;

use super::{Listing, Model, Posted};

/// **The two grades** an offer may state (REMOTE §4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Grade {
    /// A face: it asks and acts.
    #[default]
    Operator,
    /// A tool host: it executes and may ask nothing.
    Foot,
}

impl Grade {
    /// The word the wire carries.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Self::Operator => "operator",
            Self::Foot => "foot",
        }
    }

    /// Both, in the order the page seats them.
    pub const BOTH: [Self; 2] = [Self::Operator, Self::Foot];
}

/// An offer the server minted, as the page shows it: the URL, when its code
/// expires, and the same URL as a symbol where it fits one.
#[derive(Debug, Clone, PartialEq)]
pub struct Minted {
    pub offered: Offered,
    pub symbol: Option<Symbol>,
}

/// The form, and the offer last minted from it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Offering {
    /// The name typed for the new device.
    pub name: String,
    pub grade: Grade,
    pub minted: Option<Minted>,
}

impl Offering {
    /// Whether the mint fires: a name is typed.
    pub fn ready(&self) -> bool {
        !self.name.trim().is_empty()
    }
}

impl Model {
    /// Open the page on the aimed workspace. With no aim there is nothing to
    /// offer a place in.
    pub fn begin_offers(&mut self) {
        if self.aim.is_some() {
            self.stand(Listing::Offers);
        }
    }

    /// Put the page down. What was minted stays until the aim moves: the
    /// offer exists on the server whether or not the page is open.
    pub fn close_offers(&mut self) {
        self.put_down(Listing::Offers);
    }

    /// The page goes with the workspace it was about, and its rows and its
    /// minted URL with it — both are that workspace's.
    pub(super) fn retire_offers(&mut self) {
        self.put_down(Listing::Offers);
        self.pending = None;
        self.offering = Offering::default();
    }

    /// **Mint the offer** the form states, for the aimed workspace.
    pub fn post_offer(&mut self) {
        let Some(aim) = self.aim.clone().filter(|_| self.offering.ready()) else {
            return;
        };
        self.outbox.push(Posted::act(crate::verbs::offer(
            aim.address,
            self.offering.name.trim().to_owned(),
            self.offering.grade.word().to_owned(),
        )));
    }

    /// **The offer came back.** Its URL is shown until another is minted or
    /// the aim moves, and the name is cleared: it is spent on that offer.
    pub(crate) fn offered(&mut self, offered: Offered) {
        self.offering.name.clear();
        self.offering.minted = Some(Minted {
            symbol: Symbol::encode(offered.url.as_bytes()).ok(),
            offered,
        });
    }

    /// **Approve or decline the offer named `name`** in the aimed workspace.
    /// The row leaves the list when the next standing read says it has.
    pub fn post_verdict(&mut self, name: &str, approve: bool) {
        let Some(aim) = self.aim.clone() else {
            return;
        };
        let door = if approve {
            crate::verbs::approve
        } else {
            crate::verbs::decline
        };
        self.outbox
            .push(Posted::act(door(aim.address, name.to_owned())));
    }
}

#[cfg(test)]
mod tests;
