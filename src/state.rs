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
//! the aimed workspace's conversations, the selected conversation's transcript. So
//! there is nothing to invalidate and nothing that can disagree with the focus
//! — a click changes the model, and what is asked next follows from it.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
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
    place: crate::place::Place,
    stopped: bool,
    /// Registration URLs a click fired, for the poster; and how each claim
    /// ended, for the frame (`crate::ui::Registering`).
    claims: Vec<String>,
    claimed: Vec<Result<crate::seat::register::Registered, String>>,
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
                // A tail for a conversation the operator has left is dropped
                // here, where what is selected is known for certain. Filing it
                // would paint one conversation's words under another's name.
                Said::About {
                    conversation,
                    op,
                    read,
                } => {
                    if model.conversation.as_deref() == Some(conversation.as_str()) {
                        model.about(&heard.channel, &op, read);
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
                // Keyed by the row it was asked of, so no guard is owed: a
                // frame for a row nobody shows is an entry nothing reads.
                Said::Offering { provider, read } => model.offering(&provider, read),
                Said::Unreachable(why) => model.unreachable(&heard.channel, why),
                // **An act that earned no reply is an exchange, not a
                // relationship** (REMOTE §3, bl-3969), so it goes to the bar
                // and never to a channel's section — see `Model::acted`.
                Said::Acted { op, reach, said } => model.acted(&op, &reach, said),
                // The same door as a frame, with the one fact a refusal
                // cannot carry for itself — which act it answered.
                Said::Receipt { op, frame } => {
                    model.receipt(&heard.channel, &op, crate::reply::read(&frame));
                }
            }
        }
        for outcome in std::mem::take(&mut shared.claimed) {
            model.registered(outcome);
        }
        shared.claims.extend(model.registering.asked.take());
        shared.outbox.append(&mut model.outbox);
        shared.standing = Standing::of(model);
        // **And the seat's own place, on the standing set's own terms**
        // (DESIGN §4.13): a projection of the frame's model, published where
        // the question set already is, so the entry point can write it down
        // once the event loop has returned without a thread or a write of its
        // own. Nothing reads it until then.
        shared.place = crate::place::Place::of(model);
    }

    /// A worker's report.
    pub fn heard(&self, channel: &Channel, said: Said) {
        self.hold().heard.push(Heard {
            channel: channel.clone(),
            said,
        });
    }

    /// A read about one conversation, stamped with it ([`Said::About`]): the
    /// follow lane's accumulation, and the selection's standing reads.
    pub fn about(&self, channel: &Channel, conversation: &str, op: &str, read: crate::reply::Read) {
        let said = Said::About {
            conversation: conversation.to_owned(),
            op: op.to_owned(),
            read,
        };
        self.heard(channel, said);
    }

    /// One sign-in lane's fold so far, stamped with the provider row it is
    /// about. Already read, for [`Self::about`]'s reason: a lane's frame is an
    /// append and only the lane knows which read it belongs to (REMOTE §8.3).
    pub fn signing(&self, channel: &Channel, provider: &str, read: crate::reply::Read) {
        let said = Said::Signin {
            provider: provider.to_owned(),
            read,
        };
        self.heard(channel, said);
    }

    /// One provider row's `models` answer, stamped with the row it was asked
    /// of — [`Self::signing`]'s reason: the reply carries no name (bl-b7d9).
    pub fn offering(&self, channel: &Channel, provider: &str, read: crate::reply::Read) {
        let said = Said::Offering {
            provider: provider.to_owned(),
            read,
        };
        self.heard(channel, said);
    }

    /// What to ask, as of the last frame.
    pub fn standing(&self) -> Standing {
        self.hold().standing.clone()
    }

    /// **Where the seat was pointed and how it was arranged**, as of the last
    /// frame — what `src/main.rs` writes down after the window closes.
    pub fn place(&self) -> crate::place::Place {
        self.hold().place.clone()
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

    /// **The registration URLs fired since the last drain** — a take, for
    /// [`Self::compose`]'s reason: a URL works once, and nothing re-queues one.
    pub fn claims(&self) -> Vec<String> {
        std::mem::take(&mut self.hold().claims)
    }

    /// How one claim ended, for the next settle to file.
    pub fn claimed(&self, outcome: Result<crate::seat::register::Registered, String>) {
        self.hold().claimed.push(outcome);
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

/// **The endpoint that worked for one entry, this run** (DESIGN §4.49;
/// yog's `docs/REMOTE.md` §15.3) — the second tenant of this file. An entry's
/// `address` is a list dialled in order, and the endpoint that answered is
/// tried first for the rest of the run: RAM and never disk, so a box that
/// moved between networks starts from the list's own order on its next start.
/// Keyed by the entry's directory, because entries share nothing (§4.6).
///
/// It is here rather than in the channel because a channel is opened per
/// gesture and the off-frame threads each open their own — what worked
/// outlives every one of them and is shared across all of them, which is
/// exactly the state this chokepoint exists to inventory.
static WORKED: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

/// Read or replace the endpoint that worked for the entry `key` names. **`f`
/// runs under the lock and must not touch a socket.**
pub(crate) fn worked<T>(key: &str, f: impl FnOnce(&mut Option<String>) -> T) -> T {
    let mut table = WORKED
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let mut endpoint = table.remove(key);
    let answer = f(&mut endpoint);
    table.extend(endpoint.map(|worked| (key.to_owned(), worked)));
    answer
}

#[cfg(test)]
mod tests;
