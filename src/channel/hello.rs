//! **The version preface** (yog's `docs/REMOTE.md` §3): *"each end writes one
//! frame, `{"protocol": <integer>}`, before it reads the peer's."* Both write
//! before either reads, so neither waits on the other and there is no ordering
//! rule to remember.
//!
//! **This is why the seat exists as a separate program and it is not
//! decoration.** Until the four-component split one crate shipped both ends of
//! every connection and the wire could not skew. A seat is installed on the
//! operator's own laptop or phone and upgraded on that device's schedule,
//! while the engine it dials is upgraded on the server's — so the day the two
//! disagree about what a frame means is a day that will arrive, and it must
//! arrive as a sentence rather than as a gesture answered wrongly.
//!
//! **The seat writes and confirms; it never admits.** A seat dials and is
//! never dialled, so there is exactly one half of the exchange here. The
//! engine's half — refusing a peer in band on the connection it opened — is
//! the server's, and a seat that carried it would be a seat that listens.
//!
//! **A mismatch is fail-closed and the refusal names both versions**, which is
//! REMOTE §3's requirement rather than a nicety: the sentence *is* the upgrade
//! prompt, so it must name a number an operator can act on. There is no
//! negotiation, no version list and no compat shim — negotiation is the
//! mechanism that makes every later version carry every earlier one's shape
//! forever, and the operator who installed both ends can upgrade the older one.

use std::io::{self, Read, Write};

use serde_json::json;

use super::{Reach, frame};

/// **The per-bump ledger and the constant it re-exports** (bl-c515). Split out
/// because the two grow for different reasons: this file is what the preface
/// DOES and has not changed in five versions, and that one is the record of
/// every version it has carried — one entry longer after every bump, forever.
mod ledger;

pub use ledger::PROTOCOL;

/// The preface's one key, and the whole of its shape.
const KEY: &str = "protocol";

/// What a peer that stated no version is called in the sentence. An unversioned
/// build, a frame that is not an object, a frame without the key and a
/// terminator where a preface belongs are one case on purpose: none of them can
/// be served, and four sentences for one outcome is four sentences.
///
/// **A peer this end could not read at all is no longer among them** (bl-3969).
/// It was, and the collapse was right about the sentence and wrong about the
/// recovery: the other four are a peer speaking, so nothing crossed, where an
/// unreadable preface is a broken connection with this end's request already on
/// it. See [`confirm`] — the sentence is still one per outcome; there are now
/// two outcomes.
const UNSTATED: &str = "no version";

/// Write this build's preface. Called before this end reads, which is what
/// makes the exchange deadlock-free without an ordering rule.
pub fn state(w: &mut dyn Write) -> io::Result<()> {
    frame::write_value(w, &json!({ KEY: PROTOCOL }))
}

/// Read the engine's preface and refuse a mismatch — as the one [`Reach`] every
/// other thing that can go wrong with this transport already arrives as, so
/// nothing above here carries a case for it.
///
/// **The classification splits where the four collapsed cases do not.** A peer
/// that STATED something — another version, a frame that is not an object, a
/// frame without the key, a terminator where a preface belongs — refused this
/// end before adjudicating anything (REMOTE §3: a request of a version this
/// build does not speak *"is never adjudicated"*), so nothing crossed and one
/// sentence still serves all four. A preface this end could not READ is the
/// fifth case and is not about a version at all: the request went out in the
/// same breath as this end's preface, so the connection broke with the gesture
/// already at the far end and an act on it is IN DOUBT (REMOTE §3, bl-3969). It
/// says so in the transport's own words rather than borrowing a sentence about
/// a number nobody stated.
pub fn confirm(r: &mut dyn Read) -> Result<(), Reach> {
    let stated = frame::read_value(r)
        .map_err(|e| Reach::Unanswered(format!("the engine stated no version: {e}")))?;
    let peer = stated.and_then(|frame| frame.get(KEY)?.as_u64());
    if peer == Some(u64::from(PROTOCOL)) {
        return Ok(());
    }
    Err(Reach::Unsent(mismatch(peer)))
}

/// The refusal: both versions, and what to do about it.
fn mismatch(peer: Option<u64>) -> String {
    let peer = peer.map_or_else(|| UNSTATED.to_owned(), |v| v.to_string());
    format!(
        "wire protocol mismatch: this seat speaks version {PROTOCOL}, \
         the engine speaks {peer}. There is no negotiation — \
         upgrade the older component until both speak one version."
    )
}

#[cfg(test)]
mod tests;
