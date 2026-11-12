//! **One pane, two homes** (DESIGN §4.48 item 8): the three panes the
//! advanced mode docks beside the column, the two rails they dock on, and
//! the one query that decides where each is laid.
//!
//! # The home is a query, and the pane's state does not know it
//!
//! The trail ([`super::trail`]), the config pane ([`super::config`]) and the
//! records pane ([`super::records`]) each stand or not as they always did;
//! what changed is where a standing one is painted. [`Model::docked`] is the
//! whole of that decision — the mode knob, and whether the window has room
//! for two rails beside the column — and `crate::ui::overlay` and
//! `crate::ui::rail` each ask it before painting. Nothing here is a second
//! field saying *this pane is on a rail*: a pane docked is a standing pane
//! read through one query, and the mode flipping moves it with no act.
//!
//! # Why these three left the shared fields
//!
//! [`super::Lookup`] and [`super::Listing`] each hold one pane because no
//! two of theirs ever stand together. A rail's pane stands BESIDE an overlay
//! — the trail docked while the settings float — so it cannot share a field
//! with one: opening `/help` would have put the rail down. So the trail is
//! its own field, and the context rail's two occupants are one field
//! ([`Context`]), because they are exclusive on the rail and were exclusive
//! as overlays: a rail shows one pane, and a field per occupant would make
//! *both on one rail* a state the paint order resolves.
//!
//! # The room is a reading of the window
//!
//! A rail that squeezed the column under its floor would be a column nobody
//! can read a transcript in, so the home is decided once per frame from
//! the window's width ([`Model::housed`]; `crate::ui::rail::room`): with
//! room for two rails at their floor beside the bar and the column's own
//! floor, the mode is honoured; without it the panes float as the simple
//! mode lays them, and nothing is lost — the pane an operator asked for is
//! on the glass either way. The reading is written down the way the bar's
//! last pointer time is (`crate::ui::bar::Bar::left_at`): an input fact the
//! model's questions need, taken at the top of the frame.

use super::{Configuring, Model};

/// **The two rails** (DESIGN §4.48 item 8), docked at the window's right
/// while the mode is advanced: the branch rail nearer the column, the
/// context rail at the edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rail {
    /// The trail: the conversation's branch, and every act that crossed a
    /// server's boundary (§4.29, §4.35; `/trail`).
    Branch,
    /// The workspace's files as the wire lists them: the config pane's
    /// (§4.30; `/config`) and the records' (§4.32; `/records`).
    Context,
}

impl Rail {
    /// The two, in the order they are laid: the context rail first, so it
    /// takes the edge and the branch stands beside the column.
    pub const ALL: [Self; 2] = [Self::Context, Self::Branch];

    /// The rail's name, on its panel and in a complaint.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Branch => "the branch rail",
            Self::Context => "the context rail",
        }
    }
}

/// **What stands on the context rail** — or floats, in the simple mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Context {
    /// The config pane, with the file it is pointed at (`super::config`).
    Config(Configuring),
    /// The records pane, which holds nothing of its own (`super::records`).
    Records,
}

impl Model {
    /// **Whether the rails are the three panes' home** — the one query
    /// (§4.48 item 8): the mode is advanced, and the window has room.
    pub fn docked(&self) -> bool {
        self.settings.mode == crate::settings::Mode::Advanced && self.room
    }

    /// **Whether `rail`'s pane stands**, wherever it is laid.
    pub fn standing_on(&self, rail: Rail) -> bool {
        match rail {
            Rail::Branch => self.trail.is_some(),
            Rail::Context => self.context.is_some(),
        }
    }

    /// **Whether `rail` is laid**: its pane stands, and the panes are docked.
    pub fn railed(&self, rail: Rail) -> bool {
        self.docked() && self.standing_on(rail)
    }

    /// **Whether a pane floats over the conversation** — the
    /// switcher, the tuning pane, a listing, the window's own, the login
    /// pane, the fleet pane, an unmaking, or one of the three re-homed panes
    /// while they are not docked. The question the shell and every
    /// pane-opening control share, asked once so no two float on one glass:
    /// a control that opened a second would replace what is standing without
    /// saying so. A docked pane is not a cover: the column stands beside it.
    pub fn covered(&self) -> bool {
        self.switching
            || self.tuning.is_some()
            || self.listing.is_some()
            || self.lookup.is_some()
            || self.login.is_some()
            || self.fleet.is_some()
            || self.unmaking.is_some()
            || (!self.docked() && Rail::ALL.into_iter().any(|rail| self.standing_on(rail)))
    }

    /// **Write down whether the window has room for the rails**, taken at
    /// the top of the frame — and keep the one-overlay rule where the home
    /// just changed: two panes that stood docked may not both float, so the
    /// branch rail's yields, the context being about what is selected.
    pub fn housed(&mut self, room: bool) {
        self.room = room;
        if !self.docked() && self.trail.is_some() && self.context.is_some() {
            self.trail = None;
        }
    }
}

#[cfg(test)]
mod tests;
