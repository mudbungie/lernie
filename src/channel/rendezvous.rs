//! **The rendezvous material an entry may carry** (yog's `docs/REMOTE.md`
//! §13.2; DESIGN §4.40): two more files beside `ca.pem`, and everything both
//! ends derive from them.
//!
//! **Carried here by hand, like the four files beside it** (REMOTE §1.4). The
//! engine's mint grows `rendezvous.key` — a seed it never shares — and
//! `pairing.salt`; what crosses into a client's entry is the seed's PUBLIC
//! half and the salt, exactly as `ca.pem` crosses. The seat reads them and
//! writes nothing. An entry holding neither is today's entry — an address and
//! nothing to rendezvous for (§13.4's severability) — and one holding only
//! one of them is the half-provisioned misconfiguration a missing leaf is.
//!
//! **One salt, four derivations, all HKDF-SHA256** over the pairing salt
//! under the salt `yog rendezvous`, one info label each, 32 bytes out — and
//! every byte here must equal the engine's, because the commons stores what
//! the engine sealed and reads what this end seals. The DHT salt each item is
//! filed under (`presence salt`, `inbox salt`), the AEAD key both are sealed
//! with (`seal key`), and an ed25519 SEED (`inbox key`) the inbox is signed
//! under: a keypair *derived from the pairing salt*, so the seat mints no
//! keypair of its own to write it and the engine holds no client key to poll
//! it. The suite pins every derivation to bytes the engine's own code
//! produced.

use std::path::Path;

use ring::hkdf::{HKDF_SHA256, Salt};

use crate::dht::Keypair;

/// The engine's rendezvous PUBLIC key, 32 bytes of hex — presence is signed
/// under it, and it is what names the item on the commons.
pub const PUBLIC: &str = "rendezvous.pub";
/// The pairing salt, 32 bytes of hex — the secret this entry shares with
/// that one engine. The engine's own file bears the same name.
pub const SALT: &str = "pairing.salt";

/// The two sealed items, and the two halves the seat speaks.
pub mod item;
/// The simultaneous open from one port.
pub mod punch;

/// What the two files hold, decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pairing {
    /// The engine's rendezvous public key.
    pub engine: [u8; 32],
    /// The pairing salt.
    pub salt: [u8; 32],
}

impl Pairing {
    /// The keypair the inbox is signed with, which both ends can derive.
    pub fn inbox_keypair(&self) -> Result<Keypair, String> {
        Keypair::from_seed(self.derive(b"inbox key"))
    }

    /// The DHT salt the presence item is filed under.
    pub fn presence_salt(&self) -> Vec<u8> {
        self.derive(b"presence salt").to_vec()
    }

    /// The DHT salt the inbox item is filed under.
    pub fn inbox_salt(&self) -> Vec<u8> {
        self.derive(b"inbox salt").to_vec()
    }

    /// The AEAD key both items are sealed under.
    pub fn seal_key(&self) -> [u8; 32] {
        self.derive(b"seal key")
    }

    /// HKDF-SHA256 over the pairing salt, one label per derived fact.
    fn derive(&self, label: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        // HKDF-SHA256 expands to exactly 32 bytes for a 32-byte buffer, so
        // neither step can refuse; the fallback is unreachable and named.
        if let Ok(okm) = Salt::new(HKDF_SHA256, b"yog rendezvous")
            .extract(&self.salt)
            .expand(&[label], HKDF_SHA256)
        {
            let _ = okm.fill(&mut out);
        }
        out
    }
}

/// Read the pairing out of `dir`: `Ok(None)` is an entry with no roving,
/// `Err` is half of it, which earns the same remedy as half a trust store.
pub fn read_dir(dir: &Path) -> Result<Option<Pairing>, String> {
    match (hex_file(&dir.join(PUBLIC)), hex_file(&dir.join(SALT))) {
        (None, None) => Ok(None),
        (Some(engine), Some(salt)) => Ok(Some(Pairing { engine, salt })),
        _ => Err(format!(
            "{} holds half its rendezvous material: {PUBLIC} and {SALT} are carried together \
             — each 32 bytes of hex, the engine's public rendezvous key and the pairing salt \
             its wire directory holds beside ca.pem",
            dir.display()
        )),
    }
}

/// Exactly 32 bytes of hex in `path`, or nothing — a file that will not read
/// or does not decode is no material, the same as an absent one.
fn hex_file(path: &Path) -> Option<[u8; 32]> {
    let text = std::fs::read_to_string(path).ok()?;
    <[u8; 32]>::try_from(unhex(text.trim())?).ok()
}

/// Lowercase hex — the suite's, for laying the fixture down as files.
#[cfg(test)]
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

/// The inverse of [`hex`]; `None` on any byte that is not one.
pub(crate) fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    text.as_bytes()
        .chunks(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect()
}

#[cfg(test)]
pub(crate) mod tests;
