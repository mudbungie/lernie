//! **The doctor pane between frames** (bl-9bbb; DESIGN §4.21): whether it is
//! open, and what each channel said about its own wiring.
//!
//! # It is a window-level read whose subject the aim can narrow
//!
//! Bare, `doctor` names no workspace, so it fans like the verb table and the
//! pane is the union — one section per channel, each answer replacing its own.
//! Naming a workspace adds that wall's checks and goes down that wall's
//! channel alone. The pane takes the aim as that name when there is one, which
//! is the one place this read differs from `help`: it is offered on an unaimed
//! seat exactly as `help` is, and an aimed seat asks the deeper question
//! rather than a second control asking it.
//!
//! # Posted, not standing, and cleared when asked
//!
//! A diagnosis is an answer to a moment, asked by the operator and asked again
//! by opening the pane again — `help`'s cadence (`super::window`). Unlike the
//! verb table it is **cleared on asking**, because two asks may have two
//! subjects: sections left over from a bare ask, painted under an aimed one,
//! would say every channel had answered the narrower question.

use super::{Lookup, Model};
use crate::reply::doctor::Check;
use crate::ui::Channel;

/// **One channel's answer to *is this box wired***, stamped with the channel
/// it came down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnosis {
    pub channel: Channel,
    pub rows: Vec<Check>,
}

impl Model {
    /// **Open the doctor pane and ask** — about the aimed wall as well where
    /// the window is aimed at one, and about every channel's box where it is
    /// not. Nothing gates it: the seat most likely to be asking is the one
    /// with nothing to aim at.
    pub fn begin_doctor(&mut self) {
        self.lookup = Some(Lookup::Doctor);
        self.diagnoses.clear();
        let named = self.aim.as_ref().map(|aim| aim.address.clone());
        self.outbox
            .push(super::Posted::read(crate::verbs::doctor(named)));
    }

    /// **Whether that pane is the one standing.**
    pub fn doctoring(&self) -> bool {
        self.lookup == Some(Lookup::Doctor)
    }

    /// File one channel's checks, replacing what that channel last said and
    /// leaving every other channel standing ([`Model::paged`]'s terms).
    pub(super) fn diagnosed(&mut self, channel: &Channel, rows: Vec<Check>) {
        let answered = Diagnosis {
            channel: channel.clone(),
            rows,
        };
        match self
            .diagnoses
            .iter_mut()
            .find(|held| held.channel.name == channel.name)
        {
            Some(held) => *held = answered,
            None => self.diagnoses.push(answered),
        }
    }
}

#[cfg(test)]
mod tests;
