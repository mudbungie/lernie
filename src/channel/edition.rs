//! **The edition ledger, vendored** — what this build can spell, what the
//! engine can, and the field paths where the two are allowed to differ
//! (yog's `docs/REMOTE.md` §3.2; DESIGN §4.9).
//!
//! `PROTOCOL` is a **major**: it moves only on a break — a field removed or
//! re-typed, a meaning changed under a spelling in use, a field the engine
//! newly requires — and the preface stays strict equality with no negotiation.
//! Every *addition* ships at the same major and is stamped an **edition**
//! instead: the old integer line, continued. So the two numbers answer two
//! different questions, and only one of them can refuse a connection.
//!
//! - [`FLOOR`] is the edition the current major was cut at. Every field path
//!   stamped at or below it is **required** on every engine of this major, so
//!   nothing at or below the floor can be missing and nothing about it has to
//!   be discovered.
//! - [`EDITION`] is what this build can spell — the newest stamp in the
//!   `corpus/shapes.json` it vendors — and it is stated in the preface beside
//!   the major.
//! - [`STAMPS`] is the ledger of the paths **above** the floor, which are the
//!   only ones two ends of one major can disagree about. It is empty at the
//!   cut and gains a row per addition, which is what makes it readable: the
//!   whole of what this major has grown, in one list.
//!
//! **The three are vendored, not derived at build time**, on exactly
//! `PROTOCOL`'s own precedent: upstream is the authority, this repository
//! carries a copy, and a gate holds the copy equal. Here the gate is
//! [`tests`], which recomputes all three from the vendored `corpus/shapes.json`
//! and fails naming the difference — both directions, so a row for a path the
//! corpus does not stamp is as red as a stamp with no row. The corpus is
//! `#[cfg(test)]` apparatus and is not packaged; the constants are what a
//! released binary carries, and the test is what makes them true.

/// The edition the current major was cut at.
///
/// **Nothing at or below this can be absent**, which is why the ledger below
/// only ever holds what is above it: an engine speaking this major and an
/// engine speaking it two additions later agree about every one of these paths.
pub(crate) const FLOOR: u32 = 18;

/// What this build can spell — the newest stamp in its vendored corpus.
///
/// Stated in the preface beside the major so the far end knows what this one
/// can read, and re-read from the corpus by [`tests`] on every run.
pub(crate) const EDITION: u32 = 18;

/// **The post-floor ledger**: `(shape, field path, the edition it appeared
/// at)`, for every path stamped above [`FLOOR`].
///
/// Empty at the cut, and that is the design rather than a gap — the floor is
/// the edition the major was cut at, so at that instant every path in the
/// corpus is at or below it. A row is added by the same commit that re-vendors
/// the corpus which grew the path, because [`tests`] refuses the pair
/// otherwise.
///
/// A plain slice, searched linearly. It holds what one major has added since
/// its cut, which is a handful of paths — an index over it would be a second
/// representation of a list short enough to read.
pub(crate) const STAMPS: &[(&str, &str, u32)] = &[];

/// **What upstream has announced it will remove**, vendored the same way and
/// for a different reason: this one is a WARNING with a deadline on it.
///
/// A field or a shape is removed only at a major, only after being listed here
/// in a PUBLISHED yog release, and never within three yog releases of that
/// listing (REMOTE §3.2's window rule). So a name arriving in this list is the
/// one moment a consumer can act with three releases in hand — and a vendored
/// copy is what makes the arrival visible: [`tests`] refuses a corpus whose
/// list this one does not match, so the re-vendor that brings a deprecation in
/// cannot be a silent line in a generated file.
///
/// Each entry is either a whole shape (`reply/doctor`) or one field path inside
/// one (`reply/follow/rows/[]/held`) — upstream's own spelling, concatenated,
/// with no type suffix.
///
/// **`cfg(test)`, because nothing at runtime consumes a deprecation.** A
/// released binary reads the fields it reads; the whole product of this list is
/// the diff that carries a name into it, and that diff is written by whoever
/// runs the refresh. It sits beside the other two ledgers rather than inside
/// the test that reads it, because a ledger is found where the facts are.
#[cfg(test)]
pub(crate) const DEPRECATED: &[&str] = &[];

/// **The edition a field path appeared at**, off a ledger.
///
/// A path with no row is at or below the floor: required, and therefore
/// answered [`FLOOR`]. The ledger is a parameter so the reading can be
/// exercised against a table that HAS post-floor rows — at the cut the vendored
/// one has none, and a lookup nothing can find is a lookup nobody has evidence
/// about.
pub(crate) fn stamp(stamps: &[(&str, &str, u32)], shape: &str, key: &str) -> u32 {
    stamps
        .iter()
        .find(|(at, path, _)| *at == shape && *path == key)
        .map_or(FLOOR, |(_, _, edition)| *edition)
}

/// **Whether an engine of `engine` edition can spell one field path**, off a
/// ledger. [`stamp`]'s reading, compared.
pub(crate) fn spells_at(stamps: &[(&str, &str, u32)], engine: u32, shape: &str, key: &str) -> bool {
    stamp(stamps, shape, key) <= engine
}

/// The same question against the ledger this build vendors.
///
/// **What a `false` means is not "the field is absent"** — it is *this engine
/// cannot say*, which is a different sentence from the field's default and the
/// only reason to ask. A control whose fact is stamped above the far end's
/// edition greys rather than painting the reassuring answer.
pub(crate) fn spells(engine: u32, shape: &str, key: &str) -> bool {
    spells_at(STAMPS, engine, shape, key)
}

#[cfg(test)]
mod tests;
