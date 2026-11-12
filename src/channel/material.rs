//! **What the operator carried to this box**, and what its absence means
//! (yog's `docs/REMOTE.md` §1.4, §8.2; DESIGN §4.5).
//!
//! **This module only ever reads.** An entry gets here one of two ways, and
//! neither is this file's: `lernie register` files one from a one-time offer,
//! with a key this box made and never sent (REMOTE §15.1;
//! [`crate::seat::register`]); or the operator mints a leaf on the box that
//! holds the CA and carries the files here by hand, which remains for a box
//! with no route to the registration door (§15.4). The seat signs no
//! certificate either way, and never receives a private key. The flat root is
//! a third thing: the server's own boot self-provisions a loopback root for
//! the window in ITS process, an act on the operator's own box by the
//! operator's own program.
//!
//! Three answers, and they are the whole of the trust bootstrap on this side:
//!
//! - **Nothing provisioned** — `Ok(None)`. This directory holds no channel.
//!   Removing it deletes config, not code, which is why absence is an answer
//!   and not an error.
//! - **Partly provisioned** — `Err`, naming every missing file at once. Half a
//!   trust store is a misconfiguration, and one that silently degraded to *no
//!   encryption* is the failure mode mTLS exists to exclude. Every missing file
//!   at once because a remedy that reveals one gap per run is a remedy run four
//!   times.
//! - **Provisioned** — `Ok(Some(Material))`: the anchors, this box's leaf and
//!   key for this channel, and the addresses it dials, in order.
//!
//! **The four files are REMOTE §8.2's, unchanged.** An operator who
//! provisioned an entry for a yog client has provisioned one for a seat; the
//! names are the wire's, not this crate's, and renaming one would make the
//! operator's act depend on which program was installed. §8.2 names a fifth,
//! `workspace`, and it is not here on purpose: it is not material, it is the
//! name the workspace bears on its host, and it lives with the entry that
//! carries it ([`entries`](super::entries)).
//!
//! **`address` is a list** (REMOTE §15.3; DESIGN §4.49): one `host:port` per
//! line, in dial order, and a one-line file is a list of one. Two files an
//! older server handed out for the retired rendezvous — `rendezvous.pub` and
//! `pairing.salt` — may still lie in an entry; nothing here names them, so
//! they are neither read nor required.

use std::path::{Path, PathBuf};

/// The operator CA this end verifies the server against — one anchor set, and
/// the same one the server verifies this end with.
pub const ANCHORS: &str = "ca.pem";
/// This box's certificate chain for this channel. Its subject common name
/// **is** this client's identity (REMOTE §2), and its organizational unit is
/// the grade ([`leaf`](super::leaf)).
pub const CHAIN: &str = "client.pem";
/// This box's private key for that chain.
pub const KEY: &str = "client.key";
/// The `host:port` list this channel dials, one per line and in order
/// (REMOTE §15.3). One file per relationship and no flag: two spellings of
/// one address is the drift REMOTE §8 removed.
pub const ADDRESS: &str = "address";

/// **Which side of the wire a directory's material was minted on**, which is
/// the whole of what decides the remedy for its absence (bl-ad7f, bl-5cbe).
///
/// Both refusals used to be one sentence, and it was the visiting box's: *mint
/// an extra client leaf over there and carry it here*. On a single-box install
/// that is advice that does not work — the leaf already exists, the server's
/// own boot minted it, and the act is a COPY — and a `WIRE_LEAF` leaf is
/// registered in no workspace besides. It is the first thing this binary ever
/// says to a new operator, so it is the one line they copy.
///
/// It is a parameter rather than something read off the path because the
/// caller is the only one that knows: [`super::entries`] is reading a
/// directory somebody made for another box's server, and
/// [`crate::seat::route`] is reading the flat root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whose {
    /// **The flat root** — the channel this seat calls this box's own server.
    Own,
    /// **One entry** — a workspace held on another box's server.
    Elsewhere,
}

impl Whose {
    /// What a refusal names as the remedy: `lernie register` with an offer
    /// (REMOTE §15.1), or the operator's own hand — and for the second it
    /// names the four files, because *carried here by hand* said what to do
    /// and not what to carry.
    pub fn remedy(self) -> String {
        let four = format!("`{ANCHORS}`, `{CHAIN}`, `{KEY}` and `{ADDRESS}`");
        match self {
            // The single-box case first, because it is the one a stranger
            // installing all five crates on one machine is standing in.
            Self::Own => format!(
                "if a server runs on this box its material already exists — its own boot \
                 minted it — so the act is a COPY: take {four} out of that server's wire \
                 directory ($XDG_DATA_HOME/yog/wire) and put them here, under those names. \
                 `{ADDRESS}` must name a stated host:port; a server that bound `:0` asked the \
                 kernel for a port and told only its own window, so re-issue with one stated \
                 (`WIRE_HOST=<host> WIRE_PORT=<port> yog wire-certs`) before copying. If the \
                 server is on ANOTHER box, register this box with it instead: a seat already \
                 connected there mints an offer, and `lernie register <url>` claims it with a \
                 key made here"
            ),
            Self::Elsewhere => format!(
                "`lernie register <url>` files a whole entry from a one-time offer, with a \
                 key made on this box, in a directory that holds nothing yet. By hand instead, \
                 the pair is minted on the host that issued it — `WIRE_LEAF=<name> yog \
                 wire-certs` there, the assignment BEFORE the verb, because `WIRE_LEAF` is an \
                 environment variable and not an argument it parses — and carried here as \
                 {four}, under those names"
            ),
        }
    }
}

/// One channel's provisioned material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    /// The operator CA, PEM.
    pub anchors: PathBuf,
    /// This end's certificate chain, PEM.
    pub chain: PathBuf,
    /// This end's private key, PEM.
    pub key: PathBuf,
    /// Each `host:port` this channel dials, in the order the file states
    /// them. Never empty.
    pub addresses: Vec<String>,
}

impl Material {
    /// The whole list as one phrase, for a line that says what an entry dials.
    pub fn address(&self) -> String {
        self.addresses.join(", ")
    }
}

/// Read one directory as the channel it claims to be. See the module doc for
/// the three answers.
pub fn read_dir(dir: &Path, whose: Whose) -> Result<Option<Material>, String> {
    let wanted = [ANCHORS, CHAIN, KEY, ADDRESS];
    let missing: Vec<&str> = wanted
        .iter()
        .copied()
        .filter(|f| !dir.join(f).is_file())
        .collect();
    if missing.len() == wanted.len() {
        return Ok(None);
    }
    if !missing.is_empty() {
        return Err(format!(
            "{} is half-provisioned: missing {} — {}",
            dir.display(),
            missing.join(", "),
            whose.remedy()
        ));
    }
    // A file that will not read yields no address, and no address is the same
    // refusal an empty one earns: one branch, because "unreadable" and "empty"
    // are one fact about what this box can be told to dial.
    let addresses = list(&std::fs::read_to_string(dir.join(ADDRESS)).unwrap_or_default());
    if addresses.is_empty() {
        return Err(format!(
            "{} names no address; it must hold one host:port per line — {}",
            dir.join(ADDRESS).display(),
            whose.remedy()
        ));
    }
    Ok(Some(Material {
        anchors: dir.join(ANCHORS),
        chain: dir.join(CHAIN),
        key: dir.join(KEY),
        addresses,
    }))
}

/// An address file's text as the list it states: one endpoint per line, blank
/// lines and the space around an endpoint dropped.
pub fn list(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests;
