//! **The link**: what the frame and the off-frame threads say to each other,
//! and the crate's one lock.
//!
//! `rules/locks-outside-state.yml` names this file, and it named it before
//! there was anything in it — a confinement rule installed after the first site
//! is a rule that has to be argued with. This is the first tenant, and there
//! should not be a second: everything above the socket is a pure function of
//! what it is handed, and everything below it is one thread on one connection.
//!
//! # The frame owns the model and the threads never touch it
//!
//! What crosses the lock is small and one-directional in each half: frames that
//! **landed**, gestures to **send**, and the standing question set. The window's
//! [`Model`] is the frame's alone, so no worker can be mid-write in one when a
//! frame reads it, and there is no shared structure to keep consistent.
//!
//! [`Link::settle`] is the whole of the frame's side and it is called once at
//! the top of a frame: it files what landed, hands over what was composed, and
//! publishes what to ask next. Nothing in it can block — the lock is held
//! across a drain and three moves, and no worker holds it across a socket.
//!
//! # The standing set is a QUERY, never stored
//!
//! [`Standing::of`] derives what to ask from the model: every channel's roster,
//! the aimed wall's conversations, the selected conversation's transcript. So
//! there is nothing to invalidate and nothing that can disagree with the focus
//! — a click changes the model, and what is asked next follows from it.

use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use crate::ui::{Channel, Model, Posted};

/// What crosses the lock, and what the frame publishes for the workers to ask.
mod traffic;

pub use traffic::{Heard, Open, Said, Standing};

/// What the two sides share.
#[derive(Default)]
struct Shared {
    heard: Vec<Heard>,
    outbox: Vec<Posted>,
    standing: Standing,
    stopped: bool,
}

/// **The one handle**, cloned to every worker.
#[derive(Clone)]
pub struct Link {
    shared: Arc<Mutex<Shared>>,
    beat: Duration,
}

impl Link {
    /// A link whose workers pause `beat` between passes. The pause is the
    /// **cadence**, not a timeout: a seat asks at human cadence and the two
    /// surfaces that move faster than an operator looks are held reads.
    pub fn new(beat: Duration) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared::default())),
            beat,
        }
    }

    /// How long a worker waits between passes.
    pub fn beat(&self) -> Duration {
        self.beat
    }

    /// **The frame's whole side**: file what landed, take what was composed,
    /// and publish what to ask next.
    pub fn settle(&self, model: &mut Model) {
        let mut shared = self.hold();
        for heard in std::mem::take(&mut shared.heard) {
            match heard.said {
                Said::Frame(frame) => model.absorb(&heard.channel, crate::reply::read(&frame)),
                // A tail for a conversation the operator has left is dropped
                // here, where what is selected is known for certain. Filing it
                // would paint one conversation's words under another's name.
                Said::Live { conversation, read } => {
                    if model.conversation.as_deref() == Some(conversation.as_str()) {
                        model.absorb(&heard.channel, read);
                    }
                }
                // The sign-in lane's guard, and it is the tail's verbatim: a
                // run the operator has stopped following is dropped here,
                // where what the pane is following is known for certain.
                Said::Signin { provider, read } => {
                    if model.following().as_deref() == Some(provider.as_str()) {
                        model.absorb(&heard.channel, read);
                    }
                }
                Said::Unreachable(why) => model.unreachable(&heard.channel, why),
                // **An act that earned no reply is an exchange, not a
                // relationship** (REMOTE §3, bl-3969), so it goes to the bar
                // and never to a channel's section — see `Model::acted`.
                Said::Acted { op, reach } => model.acted(&op, &reach),
                // The same door as a frame, with the one fact a refusal
                // cannot carry for itself — which act it answered.
                Said::Receipt { op, frame } => {
                    model.receipt(&heard.channel, &op, crate::reply::read(&frame));
                }
            }
        }
        shared.outbox.append(&mut model.outbox);
        shared.standing = Standing::of(model);
    }

    /// A worker's report.
    pub fn heard(&self, channel: &Channel, said: Said) {
        self.hold().heard.push(Heard {
            channel: channel.clone(),
            said,
        });
    }

    /// The held read's accumulation so far, stamped with the conversation it is
    /// about. Already read, because a follow frame is an append and only the
    /// lane knows which read it belongs to (REMOTE §5.5).
    pub fn live(&self, channel: &Channel, conversation: &str, read: crate::reply::Read) {
        self.heard(
            channel,
            Said::Live {
                conversation: conversation.to_owned(),
                read,
            },
        );
    }

    /// One sign-in lane's fold so far, stamped with the provider row it is
    /// about. Already read, for [`Self::live`]'s reason: a lane's frame is an
    /// append and only the lane knows which read it belongs to (REMOTE §8.3).
    pub fn signing(&self, channel: &Channel, provider: &str, read: crate::reply::Read) {
        let said = Said::Signin {
            provider: provider.to_owned(),
            read,
        };
        self.heard(channel, said);
    }

    /// What to ask, as of the last frame.
    pub fn standing(&self) -> Standing {
        self.hold().standing.clone()
    }

    /// Everything the frame composed since the last drain.
    ///
    /// **It is a take and there is nothing that puts one back.** That is the
    /// whole of REMOTE §3's *sent exactly once per operator gesture*: the queue
    /// is the only copy, the poster is its only reader, and no arm anywhere
    /// re-queues an envelope a leg could not deliver.
    pub fn compose(&self) -> Vec<Posted> {
        std::mem::take(&mut self.hold().outbox)
    }

    /// Ask every worker to finish its pass and stop.
    pub fn stop(&self) {
        self.hold().stopped = true;
    }

    /// Whether they have been asked to.
    pub fn stopped(&self) -> bool {
        self.hold().stopped
    }

    /// The one lock acquisition, and the one place a poisoned lock is
    /// recovered: a worker that panicked mid-pass left the queues consistent —
    /// they are vectors of finished values — so the honest answer is to carry
    /// on rather than to poison every later frame with it.
    fn hold(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests;
