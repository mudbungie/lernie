//! **The envelope, decomposed into an entry** (yog's `docs/REMOTE.md` §8.2) —
//! the one thing an operator does with enrollment material by hand, done here
//! instead.
//!
//! # It is not a fifth fact about a channel
//!
//! An entry is four files, and their names are
//! [`crate::channel::material`]'s — the wire's own, read by this seat and by
//! every foot. This module writes that list rather than a list of its own, so
//! a name that moves moves once. What it lays down is exactly what
//! [`crate::channel::material::read_dir`] reads back, which is what makes the
//! act reversible by inspection: the operator can point this seat's `entries`
//! at the directory and be told what it holds.
//!
//! # Why a seat may write this and may not keep it
//!
//! DESIGN §4.15's rule is that the seat keeps no copy — no cache, no log, no
//! temporary anything, because a copy nobody chose is a private key with a
//! long life. A destination the operator named on the command line is the
//! opposite of that: it is the choice itself, and refusing to make it is what
//! sent a headless enrollment through four hand-run `jq` invocations off a
//! picture the operator had to decode first (bl-1554). Nothing here runs
//! unasked; with no destination stated, not a byte is written.
//!
//! # A file that already exists refuses
//!
//! Every file is created with [`std::fs::OpenOptions::create_new`], so an
//! entry already holding material is never overwritten. Re-issuing distrusts
//! nothing, so a clobbered `client.pem` would leave two live certificates
//! under one identity with the surviving key belonging to neither — the same
//! reason the engine's own mint refuses a second enrollment under one name
//! (REMOTE §8.4). A fresh directory is the remedy and the refusal says so.
//!
//! # And the private key is created narrow, not widened afterwards
//!
//! The mode rides the `open`, so the file is never readable by anyone else
//! even for the instant between creating it and correcting it. `#[cfg(unix)]`
//! guards the one call rather than the function, because a mode is the only
//! part of this that is a platform's fact.

use std::path::Path;

use crate::channel::material::{ADDRESS, ANCHORS, CHAIN, KEY};
use crate::reply::enrolled::Enrolled;

/// What a refusal offers instead. It is an act on this box, by this hand, and
/// it costs nothing — unlike the enrollment above it, which is spent.
const FRESH: &str = "the enrollment above is already minted, so nothing is lost by writing it \
     somewhere else: name a directory that holds no material yet";

/// **Lay the material down as an entry**, and answer with the sentence saying
/// where it went.
pub(super) fn written(dir: &Path, material: &Enrolled) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (leaf, body) in [
        (ANCHORS, material.ca.as_str()),
        (CHAIN, material.cert.as_str()),
        (KEY, material.key.as_str()),
        (ADDRESS, material.address.as_str()),
    ] {
        laid(&dir.join(leaf), body)?;
    }
    Ok(format!(
        "filed as an entry in {}: {ANCHORS}, {CHAIN}, {KEY}, {ADDRESS}. Carry the directory to \
         the box it is for, under its own `wire/workspaces/<name>/`",
        dir.display()
    ))
}

/// One file, created narrow and never over something that is already there.
///
/// **Opening and writing share one refusal**, because they are one event to the
/// operator — *this file did not get written, and here is the reason the
/// operating system gave*. The path is in the sentence, so which of the four it
/// was is never in doubt.
fn laid(path: &Path, body: &str) -> Result<(), String> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    // The address file is read back with a `trim`, and PEM already ends in one,
    // so exactly one trailing newline is right for all four and is what a
    // terminal and an editor both expect of a text file.
    let text = format!("{}\n", body.trim_end());
    options
        .open(path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, text.as_bytes()))
        .map_err(|e| format!("{}: {e} — {FRESH}", path.display()))
}

#[cfg(test)]
mod tests;
