//! **The clients pane between frames** (bl-e53c; DESIGN §4.28) — whether it is
//! open, and what the one read it stands up filed.
//!
//! # A flag, because the pane holds no question of its own
//!
//! `super::login` is a struct of two options because a sign-in being followed
//! and a row asked what it offers are two facts the pane itself owns. This pane
//! owns none: every row on it is the engine's answer, there is nothing to drill
//! into and nothing to draft, so the state is *open* and the shape is
//! `super::records`' — a `bool` beside an answer filed on the model.
//!
//! # The rows are the engine's, and they are filed whether or not it is open
//!
//! [`Model::machines`] sits beside [`Model::roles`] and [`Model::providers`]
//! for their reason verbatim: a frame in flight when the pane closes is the
//! last one, and a pane that wrote its own row back would be painting a claim
//! the engine had not made — here about *who is connected*, which is the one
//! fact on the row that is true only at the instant it was answered.
//!
//! # It retires with the wall, exactly as the login pane does
//!
//! Its read is addressed at the aim and its rows are one workspace's
//! registrations, so a pane left standing over a new aim would paint one
//! wall's machines under another's name.

use super::{Listing, Model};

impl Model {
    /// **Open the clients pane on the wall the window is aimed at**, or do
    /// nothing where it is aimed at none — the aim being the gate for
    /// [`Model::begin_tuning`]'s reason: the read it stands up carries a
    /// workspace, and a workspace is what an aim is.
    pub fn begin_clients(&mut self) {
        if self.aim.is_some() {
            self.stand(Listing::Clients);
        }
    }

    /// **Close it.** The rows stay, for the reason the roles do: the next open
    /// on the same wall is about the same machines, and the standing read
    /// replaces them anyway.
    pub fn close_clients(&mut self) {
        self.put_down(Listing::Clients);
    }

    /// **The pane and its rows go with the wall they are about** — called by
    /// the act that moves the aim, so nothing on it outlives its subject.
    pub(super) fn retire_clients(&mut self) {
        self.put_down(Listing::Clients);
        self.machines = None;
    }
}

#[cfg(test)]
mod tests;
