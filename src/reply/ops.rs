//! **The ops trail** — every action that crossed the engine's boundary, what
//! it did, and where its alarm stands (yog's `docs/REMOTE.md` §9.17, PROTOCOL
//! 11; bl-4c48).
//!
//! It is the engine's own record of what it RAN: one row per child process the
//! boundary spawned, newest last, with the classification the engine already
//! made rather than the integer a reader would have to classify itself.
//!
//! # The classification crosses, so this seat does not make one
//!
//! REMOTE §9.17 is explicit about why [`OpRow::standing`], [`OpRow::failed`]
//! and [`OpRow::exit_label`] are on the wire at all: a seat that wanted the
//! failure banner would otherwise have to re-implement the sentinel table, the
//! `128 + n` signal reading, the retirement key, the ack scan and the origin
//! grouping — *"five derivations this document names one home for apiece, and
//! whose failure mode is a seat quietly disagreeing rather than failing to
//! build"*. So nothing here reads [`OpRow::exit`] to decide anything; the
//! integer is carried because it is what an operator asks for after the label,
//! and the words are what the pane paints.
//!
//! # The standing is total, and rides verbatim
//!
//! `clean` / `detached` / `live` / `retired` / `acked` — the outcome folded
//! with the ack watermark, present on every row rather than only on failures,
//! *"which would leave a reader telling ran clean from handed off, no exit
//! observed by re-reading the exit integer"*. It is a [`String`] and not an
//! enum for [`super`]'s rung 3: a word this build has never seen paints as
//! itself, exactly as a step's `wound` does, and the one word this module has
//! a reading for is [`CLEAN`] — which it reads as *nothing to say*.

use serde_json::{Map, Value};

use super::fields;

/// The kind token this reading answers to.
pub(crate) const KIND: &str = "ops";

/// **The standing of a row with no alarm on it.** The one word this module
/// tests for, and it is tested for in order to say NOTHING: a clean run is the
/// ordinary case and a badge on every one of them would bury the rows that are
/// standing. Every other standing — the handoff, the alarm, and the two ways
/// an alarm comes down — paints as its own word.
pub const CLEAN: &str = "clean";

/// One action that crossed the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpRow {
    /// When it ran, in the engine's own spelling.
    pub ts: String,
    /// **What subject it belongs to** — the field a failure banner groups by.
    /// It is stored on the line because it cannot be derived (REMOTE §9.17).
    pub origin: String,
    /// **Where its alarm stands**, on the vocabulary above.
    pub standing: String,
    /// The row's **own** question, answerable of one line held alone — where
    /// [`Self::standing`] is a fact about its place in a tail.
    pub failed: bool,
    /// How it ended, in the engine's words. The seat paints this and never
    /// classifies [`Self::exit`] itself.
    pub exit_label: String,
    /// The status behind that label, carried because it is the next thing an
    /// operator asks for and never because anything here reads it.
    pub exit: i32,
    /// The command line that ran.
    pub argv: String,
    /// Where it ran.
    pub cwd: String,
    /// What it printed, if anything.
    pub stdout: String,
    /// What it complained, if anything.
    pub stderr: String,
}

/// One row, strictly ([`super`]'s rung 1: every refusal names its field).
pub(crate) fn row(value: &Value) -> Result<OpRow, String> {
    let obj: &Map<String, Value> = value.as_object().ok_or("ops row: not an object")?;
    Ok(OpRow {
        ts: fields::text(obj, "ts")?,
        origin: fields::text(obj, "origin")?,
        standing: fields::text(obj, "standing")?,
        failed: fields::flag(obj, "failed")?,
        exit_label: fields::text(obj, "exit_label")?,
        exit: fields::exit(obj)?,
        argv: fields::text(obj, "argv")?,
        cwd: fields::text(obj, "cwd")?,
        stdout: fields::text(obj, "stdout")?,
        stderr: fields::text(obj, "stderr")?,
    })
}

#[cfg(test)]
mod tests;
