//! **The reply vocabulary**: the typed half the window paints from (yog's
//! `docs/REMOTE.md` §3, §8, §8.1, §9.7; DESIGN §4.9).
//!
//! [`envelope`](crate::envelope) reads three things out of a gesture — that it
//! is one, which workspace it names, whether the last frame said ok — and that
//! is enough to route and to exit. It is not enough to **paint**. A window
//! draws a roster row, a conversation, a transcript step; those are typed
//! answers, and this module is the seat's reading of them.
//!
//! **It is reimplemented, not shared, and that is a ruling** (REMOTE §8:
//! *"what the seat reimplements is this document … no shared protocol crate
//! was created and none should be"*). REMOTE is the versioned authority all
//! four components implement against; a crate holding the wire would make it
//! the authority for three of them and a dependency for the fourth. So the
//! spellings below are read off REMOTE and off the encoders it governs, and
//! where this module and that document disagree, one of them is a bug.
//!
//! **It decodes only what it paints.** The engine's reply surface is forty-odd
//! kinds and most of them belong to panes that do not exist here.
//! Thirty-three do
//! not: the roster, the conversation list, one workspace's role tuning, the
//! transcript, the live tail, the conversation's records pair — the steps its
//! loop took and what its worktree holds — its spine pair, the operable
//! commits and the config commit governing them — the decision queue and the receipt
//! that raises a row onto it, the window's own three reads — the engine's verb
//! table, what a needle found and the trail — the ball pane's four — the
//! world's binding table, the fleet board, one wall's own balls and the branch
//! it tracks them on — the login pane's three — the provider
//! table, what one row offers and a sign-in run — a captured run, the detached
//! advance's receipt, the start family's two — the staged body and the minted
//! name — and a new box's material. A kind nothing renders is a kind
//! nobody has to carry, and the compiler of the window is what pulls in the
//! next one — see [`Reply`] for the roster of what is here and DESIGN §4.9
//! for what is not.
//!
//! # The decode policy, stated once
//!
//! Every reader below obeys these four rungs, and each type's own doc says
//! where it spends them. The posture is deliberate per rung rather than
//! "strict" or "tolerant" wholesale, because the two failures are not
//! symmetric: guessing at a malformed answer paints a claim nobody made, and
//! refusing a whole listing over one unrecognised word drops a hundred rows to
//! avoid painting one.
//!
//! 1. **Shape refuses.** A frame that is not an object, a required field that
//!    is missing, and a field of the wrong JSON type each answer
//!    [`Read::Unreadable`], naming the field. A seat cannot paint what it
//!    cannot read.
//! 2. **An unknown reply `kind` refuses, naming it.** That is REMOTE §3's own
//!    rule — *"the strict decode already refuses an unknown one in band,
//!    naming it, which is the boundary correcting itself rather than two
//!    protocols meeting"* — and it is why a new kind is **not** a protocol
//!    bump. The refusal is a value, never a panic, and the window paints it
//!    where the pane's content would have been: a visible row, never a silent
//!    drop.
//! 3. **An unknown *token* inside a row does not refuse.** A state, a tone, a
//!    classification, a block kind: each decodes to that field's own `Unknown`
//!    arm carrying the word **verbatim**, and the row paints with the word
//!    where the badge would be. Refusing here would spend rung 2's remedy on a
//!    listing that is otherwise entirely readable. Defaulting to a known
//!    neighbour is what is actually forbidden: a token painted as a word it is
//!    not is a lie, where a token painted as itself is merely unstyled.
//! 4. **An unknown *field* is ignored, structurally.** Every reader indexes by
//!    key, so a field the engine added rides through untouched. That is the
//!    other half of REMOTE §3's rule that a new field is not a bump, and it is
//!    the one tolerance this reader gets for free.
//!
//! # Replaying a conformance corpus
//!
//! The readers take a [`Value`] and answer a [`Read`], with no socket and no
//! state between calls, so anything that can produce a reply frame can be
//! replayed through them. `corpus/` is that harness's fixture set today —
//! hand-built from the shapes REMOTE's encoders write — and it is arranged as
//! three directories by expected outcome precisely so a corpus yog emits
//! later (yog bl-32cb) drops into it as files rather than as code.
//! `corpus/README.md` is the drop-in contract.

/// The conversation's own row, whole — the deepest read of one conversation.
pub mod agent;
/// The balls themselves: the binding table, one wall's own, and its branch.
pub mod balls;
/// The fleet board: every live ball in its column, and the loops running them.
pub mod board;
/// The machines registered in one workspace, and what each one offers.
pub mod clients;
/// One config file's bytes, and the settings its schema found in them.
pub mod config;
/// The conversation list one workspace answers with.
pub mod convs;
/// What one attempt changed — the row the work diff and a science row share.
pub mod diff;
/// A new box's material, and the envelope a camera carries it in.
pub mod enrolled;
/// The strict field readers every decoder below shares.
pub(crate) mod fields;
/// What one conversation's worktree holds.
pub mod files;
/// Which config commit a conversation resolves its policy from.
pub mod governing;
/// The engine's own verb table, which is also the parity roster's source.
pub mod help;
/// The undelivered mail waiting in one conversation's inbox.
pub mod inbox;
/// The census of what an engine can answer, and the captured run.
mod kinds;
/// The config lineages one workspace holds.
pub mod lineages;
/// One sign-in run, as the engine streams it.
pub mod login;
/// Every action that crossed the engine's boundary, and where its alarm stands.
pub mod ops;
/// What a wall can sign in to, and what one row is offering.
pub mod providers;
/// The decision queue: what is asking for the operator, anywhere.
pub mod queue;
/// The conversation's spine: its operable commits, and the cards off them.
pub mod rail;
/// Reading one frame: the dispatch off `kind`, and the refusal that wears none.
mod read;
/// What one workspace's roles are set to, and how each is tuned.
pub mod roles;
/// The workspace roster — the window's altitude-0 chrome.
pub mod roster;
/// Every delivery attempt of one workspace, with what it cost and how it ended.
pub mod science;
/// Text found across the balls, workspaces and conversations an engine sees.
pub mod search;
/// What a ball has cost, and what the sum is over.
pub mod spend;
/// The start family's two receipts.
pub mod start;
/// One step's records, drilled in under the steps list.
pub mod step;
/// The steps one conversation's loop has taken.
pub mod steps;
/// The live tail's fold.
pub mod stream;
/// The conversation itself.
pub mod transcript;

pub use kinds::{Outcome, Reply};
pub use read::read;

/// The field every reply carries, and the only one a refusal shares with an
/// answer.
const OK: &str = "ok";
/// The discriminant. **Its absence is the refusal**, and that is load-bearing:
/// [`OK`] cannot be the discriminant, because a captured run spells its own
/// verdict there — a `bl close` that failed the gate is `ok: false` and is an
/// answer, not a refusal.
const KIND: &str = "kind";
/// The refusal's own text — the engine's sentence about why nothing happened.
const ERROR: &str = "error";

/// **What one reply frame turned out to be.** Three outcomes rather than a
/// nested `Result`, because the window paints three different things and the
/// distinction is the whole product of this module:
///
/// - an answer it draws,
/// - a refusal it shows in the engine's own words,
/// - bytes it could not read, which is a statement about *this seat* and
///   carries this seat's own sentence.
///
/// Nothing here is a panic path and nothing is a silent drop: every frame that
/// arrives becomes one of the three, and all three are paintable.
///
/// **Not `Eq`**, because [`start::Prepared`] carries the body it must hand back
/// verbatim and arbitrary JSON is not `Eq`. Nothing in this crate keys on a
/// reply, so the equality given up is one no caller spends.
#[derive(Debug, Clone, PartialEq)]
pub enum Read {
    /// The engine answered, and this is the answer typed.
    Answer(Reply),
    /// The engine refused, in its own words (§7.3's story). A gesture that
    /// never ran, a gate that said no, an address that resolved to nothing.
    Refusal(String),
    /// Bytes this seat cannot read: a malformed envelope, or a `kind` this
    /// build does not know. The sentence names what was wrong, and — for an
    /// unknown kind — naming it *is* the upgrade prompt, exactly as the
    /// version preface's mismatch is.
    Unreadable(String),
}

#[cfg(test)]
mod tests;
