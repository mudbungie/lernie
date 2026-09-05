//! **What a frame composed, and what a lost reply would mean for it.**
//!
//! yog's `docs/REMOTE.md` §3 gives an act and an ask opposite recoveries — *"A
//! lost reply leaves an act IN DOUBT, and the recovery is a read — never a
//! resend … Asks are the opposite case and re-ask freely"* — so a seat that
//! paints a lost reply has to know which it sent. This is where that is
//! recorded, and it is recorded **at the control**, exactly as the `act:<op>`
//! parity tag is (DESIGN §4.16: *"the decision about which ops a control fires
//! is recorded at the control and nowhere else"*).
//!
//! # It cannot be computed, and that was checked rather than assumed
//!
//! The tempting derivation is the poster's own branch: a gesture naming no
//! workspace is fanned, and the three window-level reads (§4.21) are exactly the
//! nameless ones this window composes. It does not hold. The vendored request
//! vocabulary carries ops with no workspace slot that plainly change the world —
//! `create`, `close`, `complete`, `deliver`, `update`, `retire` among them — so
//! the predicate is true of what this build happens to compose and false of the
//! vocabulary. A rule that holds by coincidence is one the next control breaks
//! silently, so the tests below assert it against the corpus rather than
//! against a list written here.
//!
//! The other derivation — a table of which ops are acts — is the second
//! implementation `crate::envelope` exists to refuse: *"Reimplementing that side
//! to route a gesture would be a second table over thirty-odd variants"*. The
//! composing control already knows, for free, and knowing it there costs one
//! word per site.

//! # The channel is the second thing a frame knows and an envelope cannot say
//!
//! The same argument, one field over (bl-4855). The poster reads *no
//! workspace* as *every channel this box holds*, which is true of a window-level
//! READ — a roster refresh, a verb table, a search — and false of a `config`
//! write aimed at one engine's own `cadence.yaml`, which names no workspace
//! either and would be written to every engine this box is a client of. The
//! envelope cannot tell them apart, because on the wire they are the same
//! shape; the composing control can, for free, because it fired on a pane
//! open on an aim.
//!
//! So a gesture may name the channel it is addressed to, and naming none is
//! not a default — it is the assertion *every channel is my subject*. It is
//! recorded at the control for [`Self::act`]'s reason and for
//! `crate::offframe::poster`'s: **the aim is not read on the send pass at
//! all**, because an operator may compose one gesture aimed at a wall on one
//! channel and fire another at a row on a second before either leaves.

use serde_json::Value;

use crate::ui::Channel;

/// A gesture on its way out, and whether a lost reply leaves it in doubt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    /// The envelope, exactly as [`crate::verbs`] built it. Nothing here reads
    /// it; the poster routes it and the transport carries it.
    pub envelope: Value,
    /// **Whether it changes the world.** `true` is an act: sent once per
    /// operator gesture, never resent, and IN DOUBT if no reply comes back.
    /// `false` is a read, which may be asked again freely.
    pub act: bool,
    /// **The channel this gesture is addressed to**, where its envelope names
    /// no workspace and cannot address one itself.
    ///
    /// `None` is not *unset*: it is the window-level read's own assertion that
    /// the subject is every channel this box holds. A gesture whose envelope
    /// DOES name a workspace never carries one — `crate::seat::route` chooses
    /// the channel by resolving that name over this box's entries, and
    /// rewrites it to the host's spelling on the way, which naming a channel
    /// here would bypass.
    pub channel: Option<Channel>,
}

impl Posted {
    /// A gesture that changes the world.
    pub fn act(envelope: Value) -> Self {
        Self {
            envelope,
            act: true,
            channel: None,
        }
    }

    /// A gesture that only asks. **The window's three posted reads** (§4.21's
    /// roster refresh, verb table and search) plus anything later that only
    /// looks; the standing set is not posted at all and so never comes through
    /// here.
    pub fn read(envelope: Value) -> Self {
        Self {
            envelope,
            act: false,
            channel: None,
        }
    }

    /// **Address it down one channel**, which is what a gesture naming no
    /// workspace must do when its subject is one engine rather than all of
    /// them.
    #[must_use]
    pub fn down(self, channel: Channel) -> Self {
        Self {
            channel: Some(channel),
            ..self
        }
    }
}

#[cfg(test)]
mod tests;
