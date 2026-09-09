//! **The version ledger**, and the constant it re-exports.
//!
//! Split from [`super`] (bl-c515) along the seam the file has: that one is the
//! preface's MECHANISM — write, confirm, refuse in a sentence naming both
//! numbers — and it does not change. This is the record of every version that
//! mechanism has carried. A module whose length is a function of how long the
//! wire has existed does not belong inside one whose length is a function of
//! what it does.
//!
//! **It no longer grows once per bump** (bl-aa01). It did, and 2 through 18 are
//! that era, kept whole in [`exact`]: the integer moved for any change to an
//! existing shape and a client re-pinned for each one. Since 19 the number is a
//! MAJOR and an addition moves an EDITION instead ([`super::super::edition`]),
//! so an entry here is a rare and deliberate act with a migration note, and the
//! file's length has stopped being a function of how busy upstream was.
//!
//! **The ledger is prose because the trap it exists for is prose-shaped**
//! (DESIGN §4.9): a corpus replay catches a shape that CHANGED, and only words
//! catch a shape that changed its MIND. Every entry therefore says what moved
//! and what this seat did about it — including *nothing*, which is the
//! commonest answer and the one a reader most needs stated.
//!
//! **The NUMBER is not here either** (bl-55b1). The repo-root `PROTOCOL` file
//! states it and `build.rs` compiles that into the constant re-exported below,
//! because the gates that read it are other repositories fetching one path out
//! of a tree they do not build — and a Rust path is not a stable address for
//! that, as this very split demonstrates. So there are two homes and neither
//! is a copy of the other: the file says WHAT the number is, and this says why
//! it is that.

/// **The exact era's entries, 2 through 18** — one integer per change, whole
/// and unedited, in a module of nothing but prose (bl-aa01). Split off here
/// because the two grow for different reasons and one of them has stopped:
/// that ledger is closed at 18, and this file is the discipline now in force.
mod exact;

/// The protocol this build speaks: the MAJOR.
///
/// **It moves only on a BREAK** (yog's `docs/REMOTE.md` §3.2, yog bl-e598; here
/// bl-aa01) — a field removed or re-typed, a frame's meaning changed under an
/// unchanged spelling, a field the engine newly requires. Everything else is an
/// ADDITION and ships under the same major, stamped with an edition in
/// `corpus/shapes.json` and carried by [`super::super::edition`]: a new field, a
/// new word in a vocabulary, a new op, a new reply kind. A peer that has not
/// heard of one of those already refuses it in band, naming it, which is the
/// boundary correcting itself rather than two protocols meeting.
///
/// **19 is the last bump of the old kind and the first of the new** (yog
/// bl-e598; bl-aa01 here). Nothing on the wire changed shape at it — it is the
/// discipline's own cut, and the floor it sets is 18: every field path stamped
/// at or below 18 is required on every engine of this major, and everything
/// stamped later is optional to read. What this seat paid for it is the whole
/// grows-only contract rather than a field: the reader's four clauses stated
/// where each lives (DESIGN §4.9), the projection and word-mutation replays
/// over the vendored corpus, the preface's second key, and the split of
/// `corpus/unpainted/` out of `corpus/unreadable/` — because under a major that
/// moves rarely, *this build does not paint that kind* stops being a temporary
/// state and becomes the ordinary one.
///
/// **The transition's safety is the bump itself.** No engine speaking 19
/// publishes until every consumer's `main` carries 19, which is gate 1 of the
/// four-repository act (`AGENTS.md`), and the consumers land the number with
/// their grows-only decoders in the same commit. After it, the two release
/// holds fire only when the file moves, which is now rare by design.
///
/// **It is not declared here** (bl-55b1). The repo-root `PROTOCOL` file states
/// it and `build.rs` compiles that into the constant re-exported below — the
/// shape yog and every consumer now carry, because the gates that read this
/// number are other repositories fetching one path out of a tree they do not
/// build, and a Rust path is not a stable address for that. **Bump it by
/// editing that line**; nothing under `src` says the number.
pub use protocol::PROTOCOL;

/// The generated constant: `build.rs` writes it from the repo-root `PROTOCOL`
/// file.
mod protocol {
    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}
