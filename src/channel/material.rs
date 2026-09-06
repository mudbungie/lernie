//! **What the operator carried to this box**, and what its absence means
//! (yog's `docs/REMOTE.md` §1.4, §8.2; DESIGN §4.5).
//!
//! **The seat mints nothing.** The certificate and its key are issued by the
//! operator's own CA, on the box that holds it, and carried here by hand. So
//! this module only ever *reads*, and there is no bootstrap flow to secure: a
//! seat that could provision itself over the wire would be a seat any wire
//! could provision. That is the whole of REMOTE §1.4 on this side — the
//! engine's own boot self-provisions a loopback root for the window in ITS
//! process, which is an act on the operator's own box by the operator's own
//! program, and it is not this crate's.
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
//!   key for this channel, and the one address it dials.
//!
//! **The four files are REMOTE §8.2's, unchanged.** An operator who
//! provisioned an entry for a yog client has provisioned one for a seat; the
//! names are the wire's, not this crate's, and renaming one would make the
//! operator's act depend on which program was installed. §8.2 names a fifth,
//! `workspace`, and it is not here on purpose: it is not material, it is the
//! name the workspace bears on its host, and it lives with the entry that
//! carries it ([`entries`](super::entries)).

use std::path::{Path, PathBuf};

/// The operator CA this end verifies the engine against — one anchor set, and
/// the same one the engine verifies this end with.
pub const ANCHORS: &str = "ca.pem";
/// This box's certificate chain for this channel. Its subject common name
/// **is** this client's identity (REMOTE §2), and its organizational unit is
/// the grade ([`leaf`](super::leaf)).
pub const CHAIN: &str = "client.pem";
/// This box's private key for that chain.
pub const KEY: &str = "client.key";
/// The `host:port` this channel dials. One address per relationship and no
/// flag: two spellings of one address is the drift REMOTE §8 removed.
pub const ADDRESS: &str = "address";

/// **Which side of the wire a directory's material was minted on**, which is
/// the whole of what decides the remedy for its absence (bl-ad7f, bl-5cbe).
///
/// Both refusals used to be one sentence, and it was the visiting box's: *mint
/// an extra client leaf over there and carry it here*. On a single-box install
/// that is advice that does not work — the leaf already exists, the engine's
/// own boot minted it, and the act is a COPY — and a `WIRE_LEAF` leaf is
/// registered in no workspace besides. It is the first thing this binary ever
/// says to a new operator, so it is the one line they copy.
///
/// It is a parameter rather than something read off the path because the
/// caller is the only one that knows: [`super::entries`] is reading a
/// directory somebody made for another box's engine, and
/// [`crate::seat::route`] is reading the flat root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Whose {
    /// **The flat root** — the channel this seat calls this box's own engine.
    Own,
    /// **One entry** — a workspace held on another box's engine.
    Elsewhere,
}

impl Whose {
    /// What a refusal names as the remedy. **It is an act by the operator's
    /// own hand, out of channel, always** (REMOTE §1.4) — never a target this
    /// binary could be asked to run — and it names the four files, because
    /// *carried here by hand* said what to do and not what to carry.
    pub fn remedy(self) -> String {
        let four = format!("`{ANCHORS}`, `{CHAIN}`, `{KEY}` and `{ADDRESS}`");
        match self {
            // The single-box case first, because it is the one a stranger
            // installing all five crates on one machine is standing in.
            Self::Own => format!(
                "if an engine runs on this box its material already exists — its own boot \
                 minted it — so the act is a COPY: take {four} out of that engine's wire \
                 directory ($XDG_DATA_HOME/yog/wire) and put them here, under those names. \
                 `{ADDRESS}` must name a stated host:port; an engine that bound `:0` asked the \
                 kernel for a port and told only its own window, so re-issue with one stated \
                 (`WIRE_HOST=<host> WIRE_PORT=<port> yog wire-certs`) before copying. If the \
                 engine is on ANOTHER box, mint a leaf there instead (`WIRE_LEAF=<name> yog \
                 wire-certs` — the assignment before the verb) and carry the same four files \
                 here; the seat mints nothing"
            ),
            Self::Elsewhere => format!(
                "the pair is minted on the host that issued it — `WIRE_LEAF=<name> yog \
                 wire-certs` there, the assignment BEFORE the verb, because `WIRE_LEAF` is an \
                 environment variable and not an argument it parses — and carried here by hand \
                 as {four}, under those names; the seat mints nothing"
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
    /// `host:port` — the engine this channel dials.
    pub address: String,
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
    let address = std::fs::read_to_string(dir.join(ADDRESS))
        .unwrap_or_default()
        .trim()
        .to_owned();
    if address.is_empty() {
        return Err(format!(
            "{} names no address; it must hold one host:port — {}",
            dir.join(ADDRESS).display(),
            whose.remedy()
        ));
    }
    Ok(Some(Material {
        anchors: dir.join(ANCHORS),
        chain: dir.join(CHAIN),
        key: dir.join(KEY),
        address,
    }))
}

#[cfg(test)]
mod tests;
