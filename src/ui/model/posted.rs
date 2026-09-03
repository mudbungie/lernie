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

use serde_json::Value;

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
}

impl Posted {
    /// A gesture that changes the world.
    pub fn act(envelope: Value) -> Self {
        Self {
            envelope,
            act: true,
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
        }
    }
}

#[cfg(test)]
mod tests;
