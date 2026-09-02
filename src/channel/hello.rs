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

use super::frame;

/// The protocol this build speaks.
///
/// **A new verb is not a bump.** A `Query`, an `Action` or a reply kind the
/// peer has not heard of already refuses in band, naming it (REMOTE §3's strict
/// decode) — the boundary correcting itself, not two protocols meeting. The
/// integer moves when the *existing* shape changes meaning: the framing, the
/// envelope, or what a spelling already in use is taken to say.
///
/// **2 was yog bl-77be's bump**, and it was the second clause of that rule
/// rather than the first: four shapes grew an optional field
/// (`request/advertise` and `reply/clients` gained a tool's `subject_cwd`
/// consent, `request/invoke` and `reply/invocations` gained the subject's
/// `cwd`), and — the part no ledger can see — REMOTE §5.5 changed what a
/// follow frame's `text`/`thinking` are taken to say, from the whole
/// accumulated answer to what landed since the previous frame. The spelling
/// did not move; the meaning did, which is exactly what this integer is for.
///
/// **3 and 4 are two bumps of one unreleased cycle** (REMOTE §9.10, §9.11),
/// and the pair is why this integer is not a count of releases. 3 gave
/// `reply/conversations`' row, the §6 queue row and the `agent` answer a
/// `failure` clause — why the conversation's latest model call failed — and 4
/// gave the queue row a `flag` object beside a new `flagged` signal token.
/// Each is a gained field, which §3's rule bumps whether or not a reader needs
/// it; the ledger's granularity is per bump and not per release, so a shape
/// touched twice in one cycle costs two integers. Neither number was ever
/// spoken by a peer.
///
/// **This seat consumes exactly one of the four fields** (bl-d774), which is
/// DESIGN §4.9's rule holding rather than a shortfall: `failure` reaches
/// [`crate::reply::convs::ConvRow`] because the conversation list is the pane
/// that paints the row it hangs on, and the `agent`, `attention` and queue
/// shapes stay in the corpus ledger under `unreadable/` because no pane here
/// reads them. A field is carried by the release that paints it.
pub const PROTOCOL: u32 = 4;

/// The preface's one key, and the whole of its shape.
const KEY: &str = "protocol";

/// What a peer that stated no version is called in the sentence. An unversioned
/// build, a frame that is not an object, a frame without the key and a peer
/// that hung up mid-preface are one case on purpose: none of them can be
/// served, and four sentences for one outcome is four sentences.
const UNSTATED: &str = "no version";

/// Write this build's preface. Called before this end reads, which is what
/// makes the exchange deadlock-free without an ordering rule.
pub fn state(w: &mut dyn Write) -> io::Result<()> {
    frame::write_value(w, &json!({ KEY: PROTOCOL }))
}

/// Read the engine's preface and refuse a mismatch — as the one `Err(String)`
/// every other thing that can go wrong with this transport already arrives as,
/// so nothing above here carries a case for it.
pub fn confirm(r: &mut dyn Read) -> Result<(), String> {
    let peer = stated(r);
    if peer == Some(u64::from(PROTOCOL)) {
        return Ok(());
    }
    Err(mismatch(peer))
}

/// The version the peer stated, or `None` when it stated none.
fn stated(r: &mut dyn Read) -> Option<u64> {
    frame::read_value(r).ok().flatten()?.get(KEY)?.as_u64()
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
