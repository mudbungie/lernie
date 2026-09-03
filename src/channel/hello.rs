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
///
/// **5 is the first clause of the rule and the one this seat cannot check**
/// (REMOTE §9.12, upstream bl-e654; bl-e6ee here). `reply/governing` lost
/// `branch`, gained `follows` and `diverged_lineages`, and — the half no
/// signature can see — its `oid` **changed meaning under the same key**: it
/// named the `config/*` ancestor an agent's branch forked off, a commit that
/// never moved, and now names the commit control reads at each step boundary,
/// the followed lineage's head. The doctrine inverted with it, from
/// fork-is-the-freeze to follow-the-tip.
///
/// **Nothing here decodes `governing`**, so this seat paid the integer and no
/// field, and that is the whole of what it owed: the shape falls to
/// [`crate::reply::read`]'s unknown-kind arm and its fixture asserts exactly
/// that from `corpus/unreadable/`. The trap is recorded here rather than
/// nowhere, because it is aimed at whoever lands the pane: a reader that took
/// `oid` for the fork commit would paint a plausible number that has been
/// wrong since this bump, and it is the one kind of drift a corpus replay
/// cannot catch — the bytes are well-formed and the field is spelled the same.
///
/// **6 is a bump this seat paid for one row and not for the two ops beside
/// it** (REMOTE §9.13, upstream bl-23bd; bl-675e here). `reply/providers`'
/// rows gained `effort` and `priority`, two required booleans saying which
/// tuning knobs that provider row actually takes — a capability stated as a
/// column of the row it is about, rather than as a second answer a seat would
/// have to join back. The `/effort` and `/priority` **ops** that landed with
/// them moved nothing: a new op is a new spelling in an existing vocabulary,
/// and a peer that has not heard of one refuses it in band by name, which is
/// the first paragraph of this comment rather than an exception to it.
///
/// Nothing here paints providers either, so this is the second consecutive
/// integer bought without a field — see [`crate::reply`] for why that is the
/// arrangement working. The request half is not free of obligation, though:
/// the two new ops carry a top-level `workspace` and `src/verbs/tests/corpus`
/// asserts this seat routes every vocabulary frame by the slot upstream's own
/// signature says it carries, which is where a miss would be silent.
///
/// **7 is the first bump this end reads a field out of** (REMOTE §9.14,
/// upstream bl-8758; bl-38d4 here). Every `reply/help` row gained `surface`,
/// classing the op `control` — every seat owes it a discoverable interactable
/// — or `machine`, spoken by programs and owed nothing. It is wire-visible
/// because it rides a reply this seat vendors, and it is load-bearing here:
/// the field IS the roster the interface-parity gate judges this window
/// against (`crate::snapshot::parity`, yog's `docs/PARITY.md` §2). The bump
/// also carried the `roles` shape, which nothing here paints yet and which
/// therefore landed in the corpus ledger — `corpus/README.md` on why that
/// directory is the record rather than an oversight.
pub const PROTOCOL: u32 = 7;

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
