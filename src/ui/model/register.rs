//! **Registering this box, between frames** (DESIGN §4.50): the URL in the
//! page's field, and where the one act stands.
//!
//! The act dials — an HTTPS request, then the wire — so it is never the
//! frame's. A click moves the URL to [`Registering::asked`]; the next settle
//! hands it to the poster (`crate::state::Link::settle`), and the outcome
//! comes back the same way and lands in [`Registering::phase`]. One at a
//! time: the button is dark while a claim is out, because a registration URL
//! works once and a second click would spend nothing but could race the
//! first's filing.

use super::{Channel, Chunk, Model};
use crate::seat::register::Registered;

/// Where the act stands.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Phase {
    /// Nothing asked yet.
    #[default]
    Idle,
    /// A claim is out and has not come back.
    Claiming,
    /// How the last one ended.
    Done(Result<Registered, String>),
}

/// The page's state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Registering {
    /// What is typed in the field.
    pub url: String,
    /// A URL the operator fired and the poster has not taken yet.
    pub asked: Option<String>,
    pub phase: Phase,
}

impl Registering {
    /// Whether the button fires: something typed, and no claim out.
    pub fn ready(&self) -> bool {
        !self.url.trim().is_empty() && self.phase != Phase::Claiming
    }
}

impl Model {
    /// **Fire the claim** for what is typed. Answers whether it fired.
    pub fn register(&mut self) -> bool {
        if !self.registering.ready() {
            return false;
        }
        self.registering.asked = Some(self.registering.url.trim().to_owned());
        self.registering.phase = Phase::Claiming;
        true
    }

    /// **The claim came back.** A box that registered holds one more channel,
    /// so the roster gains its section at once — as the boot would seat it
    /// from the disk — and the standing reads begin to ask it; what it
    /// answers while the offer is unapproved is the server's own refusal, on
    /// that section. The field is cleared on success: the URL is spent.
    pub fn registered(&mut self, outcome: Result<Registered, String>) {
        if let Ok(registered) = &outcome {
            self.registering.url.clear();
            if !self
                .roster
                .iter()
                .any(|chunk| chunk.channel.name == registered.workspace)
            {
                self.roster.push(Chunk {
                    channel: Channel {
                        name: registered.workspace.clone(),
                        named_there: Some(registered.workspace.clone()),
                        dials: Some(registered.addresses.join(", ")),
                    },
                    ..Chunk::default()
                });
            }
        }
        self.registering.phase = Phase::Done(outcome);
    }
}

#[cfg(test)]
mod tests;
