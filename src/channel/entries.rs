//! **The client-side workspace** (yog's `docs/REMOTE.md` §2, §8.2; DESIGN §4.6)
//! — what this box holds so it can participate in a workspace hosted on
//! another box.
//!
//! An **entry** is a directory carrying the channel facts that reach one
//! workspace: the host engine's anchors, this box's leaf and key for it, the
//! host's address, and optionally the name that workspace bears *there*. It is
//! the client's half of the pair a server-side registration is the other half
//! of — **possession, where registration is permission** — exactly as a channel
//! needs both a certificate and its issuer's trust.
//!
//! ```text
//! <data root>/wire/workspaces/<leaf>/
//!     ca.pem      the HOST engine's anchors — that operator's trust root
//!     client.pem  this box's leaf for this workspace
//!     client.key  this box's private key for it
//!     address     the host engine, host:port — "the server", entire
//!     workspace   OPTIONAL: the name the workspace bears on its host, when it
//!                 differs from <leaf>; absent, the leaf is the name
//! ```
//!
//! **It is not a second noun, and there is no server object.** A workspace is
//! one word at both ends; "entry" names a *spelling* of it. The client-side
//! unit is the (server, workspace) participation and never the server, so
//! nothing here enumerates a server or holds a fact about one — a server is the
//! [`ADDRESS`](super::material::ADDRESS) inside an entry, entire. Two entries
//! naming one address are two trust relationships that happen to terminate at
//! one listener.
//!
//! **The fifth file exists because a host's namespace is the host's fact**
//! (§9.6: names are global per server), and two hosts may both call something
//! `home`. So `<leaf>` is the *client's* name — what this box's roster paints
//! and what every gesture typed here resolves against — and [`WORKSPACE`] is
//! what that workspace answers to on the far side. The remedy for a collision
//! between two entries is a local rename, which is `mv`, never a server-side
//! rewrite. The mapping between the two names is spent at exactly one place,
//! the channel boundary, in both directions ([`envelope`](crate::envelope)).
//!
//! **Separation is the absence of a mechanism.** Entries share nothing — not
//! anchors (two servers are two operators' trust roots), not leaves (one
//! certificate is one client identity), not addresses, not conversations. So
//! there is no inheritance from the flat root and no path by which one entry
//! can be read through another; the only structure below is a `readdir` and a
//! read per directory.
//!
//! **A refusal is one entry's, never the set's.** [`Entry::channel`] carries
//! its own `Result`, so a half-provisioned entry says so while every other
//! entry stands: a box holding three workspaces does not lose the two that are
//! fine. That is also why an entry that exists is the answer to its own name
//! even when it cannot be dialled — falling through to the flat root would send
//! a gesture to the wrong engine on the strength of a missing file.
//!
//! **Nothing here writes anything.** Material reaches an entry by the
//! operator's hand, out of channel, forever (REMOTE §1.4).

use std::path::{Path, PathBuf};

use super::material::{self, Material, REMEDY};

/// The material directory's leaf under the data root. The flat root itself:
/// the box's own client relationship, held without naming it.
pub const WIRE: &str = "wire";
/// The entries directory's leaf under [`WIRE`] — the one level of naming that
/// turns the flat client shape into a workspace this box holds elsewhere.
pub const ENTRIES: &str = "workspaces";
/// The optional file naming the workspace **on its host**, for when that
/// differs from the entry's leaf.
pub const WORKSPACE: &str = "workspace";

/// One workspace this box participates in elsewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The directory name — the *client's* name for the workspace, which the
    /// roster paints and every gesture resolves against.
    pub leaf: String,
    /// The name the workspace answers to on its host: the [`WORKSPACE`] file
    /// when it states one, the leaf otherwise.
    pub workspace: String,
    /// Its material, or the sentence saying why it has none.
    pub channel: Result<Material, String>,
}

impl Entry {
    /// **This entry, opened** — the one place an entry becomes a client of its
    /// host. Its `Err` is the entry's own sentence where the material would not
    /// read, and the channel's where it read but will not open; both are one
    /// fact to every caller — *this channel cannot be dialled, and here is
    /// why* — so there is one function and not one per cause.
    pub fn open(&self) -> Result<super::Channel, String> {
        super::Channel::open(&self.channel.clone()?)
    }
}

/// The flat root under a data root: the box's own client material.
pub fn flat(data_root: &Path) -> PathBuf {
    data_root.join(WIRE)
}

/// Where entries live under a data root.
pub fn dir(data_root: &Path) -> PathBuf {
    flat(data_root).join(ENTRIES)
}

/// Every entry in `dir`, sorted by [`leaf`](Entry::leaf).
///
/// **A directory that will not read is zero entries, not a refusal.** Absent,
/// unreadable and empty are one fact — this box holds no workspace elsewhere —
/// and that fact is the shape every box had before §8.2 existed.
pub fn read_dir(dir: &Path) -> Vec<Entry> {
    let Ok(listing) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut held: Vec<Entry> = listing
        .flatten()
        .map(|found| found.path())
        // An entry *is* a directory (§8.2). A stray file beside them names no
        // intent and is not an entry with a problem.
        .filter(|path| path.is_dir())
        .map(|path| entry(&path))
        .collect();
    held.sort_by(|a, b| a.leaf.cmp(&b.leaf));
    held
}

/// One directory read as the entry it claims to be.
fn entry(dir: &Path) -> Entry {
    let leaf = dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let channel = match material::read_dir(dir) {
        Ok(Some(held)) => Ok(held),
        // Nothing provisioned is silence at the entries directory, where
        // absence means this box holds no channel. Here it is a refusal: a
        // directory somebody made names an intent, and an intent with no
        // material behind it is the half-provisioned failure one step earlier.
        Ok(None) => Err(format!("{} is an empty entry: {REMEDY}", dir.display())),
        Err(refusal) => Err(refusal),
    };
    let workspace = named(dir, &leaf);
    Entry {
        leaf,
        workspace,
        channel,
    }
}

/// The name this workspace bears on its host. Absent, unreadable and empty are
/// one branch for the reason [`material`]'s address read has one: they are one
/// fact — the entry states no host-side name — and the leaf is then the name.
fn named(dir: &Path, leaf: &str) -> String {
    let stated = std::fs::read_to_string(dir.join(WORKSPACE))
        .unwrap_or_default()
        .trim()
        .to_owned();
    if stated.is_empty() {
        leaf.to_owned()
    } else {
        stated
    }
}

#[cfg(test)]
mod tests;
