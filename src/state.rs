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

use serde_json::Value;

use crate::ui::{Channel, Model};

/// What crosses the lock, and what the frame publishes for the workers to ask.
mod traffic;

pub use traffic::{Heard, Said, Standing};

/// What the two sides share.
#[derive(Default)]
struct Shared {
    heard: Vec<Heard>,
    outbox: Vec<Value>,
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
                Said::Live {
                    conversation,
                    frame,
                } => {
                    if model.conversation.as_deref() == Some(conversation.as_str()) {
                        model.absorb(&heard.channel, crate::reply::read(&frame));
                    }
                }
                Said::Unreachable(why) => model.unreachable(why),
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

    /// One frame of the held read, stamped with the conversation it is about.
    pub fn live(&self, channel: &Channel, conversation: &str, frame: Value) {
        self.heard(
            channel,
            Said::Live {
                conversation: conversation.to_owned(),
                frame,
            },
        );
    }

    /// What to ask, as of the last frame.
    pub fn standing(&self) -> Standing {
        self.hold().standing.clone()
    }

    /// Everything the frame composed since the last drain.
    pub fn compose(&self) -> Vec<Value> {
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
