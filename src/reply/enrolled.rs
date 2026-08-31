//! **A new box's material** (yog's `docs/REMOTE.md` §8.4) — the six fields the
//! `enroll` act answers, and the one envelope a camera carries them in.
//!
//! # The reply this seat must not keep
//!
//! Every other kind in this vocabulary is something the window draws and may
//! redraw. This one carries a **private key that is not this box's** — minted
//! on the engine's CA for a device that does not exist yet — and the seat's
//! whole job with it is to put it on a screen and forget it. Nothing here
//! writes, caches or logs; [`Enrolled`] is a value the window holds while a
//! symbol is on screen and drops with the pane. See DESIGN §3.
//!
//! # The envelope is a re-saying, not a second field list
//!
//! [`Enrolled::envelope`] is here rather than beside the QR encoder for one
//! reason: it names the same six fields the reader above it names, and two
//! lists of one field set drift. REMOTE §8.4 is the authority for its shape and
//! this module implements it:
//!
//! - **compact JSON**, no whitespace;
//! - the six fields **verbatim**, PEM as minted — DER-plus-base64 was measured,
//!   buys about a tenth of the bytes and costs the property worth keeping,
//!   which is a field an operator can paste into `openssl x509 -text`;
//! - under `"yog-enroll": 1`, the marker a scanner recognises it by and the
//!   version it will be told about if the fields ever move;
//! - and **without `ok` and `kind`**, which say what a *wire answer* is. A
//!   photograph is not one.
//!
//! Key order is `serde_json`'s own, which is sorted, and that is the same order
//! the engine's encoder writes — one less thing for two ends to disagree about.
//! It is not semantic either way: a scanner parses the object.

use serde_json::{Map, Value};

use super::fields;

/// The reply kind this module reads.
pub(crate) const KIND: &str = "enrolled";

/// The marker a scanner recognises the envelope by, and the version of the
/// field set under it.
const MARKER: &str = "yog-enroll";
const VERSION: u64 = 1;

/// The six fields, spelled once. The reader and the envelope both read this
/// list, which is what makes it one list.
const GRADE: &str = "grade";
const NAME: &str = "name";
const ADDRESS: &str = "address";
const CA: &str = "ca";
const CERT: &str = "cert";
const KEY: &str = "key";

/// **What a new box needs to dial this engine, and nothing more.**
///
/// `address` is the engine's own wire address *as clients dial it*, not the
/// port a `:0` request became — REMOTE §8.4 makes the engine refuse the second,
/// because a symbol carrying a runtime port would be stale before it was
/// scanned. So a seat has nothing to check here: an address that arrived is an
/// address the engine already stood behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enrolled {
    /// `operator` or `foot` (REMOTE §4.2), minted into the subject.
    pub grade: String,
    /// The subject common name, which **is** the client identity.
    pub name: String,
    /// Where the new box dials.
    pub address: String,
    /// The trust anchor both ends verify against.
    pub ca: String,
    /// The new box's own leaf.
    pub cert: String,
    /// That leaf's private key. **The one field on this surface that is a
    /// secret**, and the reason this reply is never written down.
    pub key: String,
}

impl Enrolled {
    /// **The envelope a symbol carries**, per REMOTE §8.4 — compact JSON, the
    /// six fields verbatim, under the marker and its version.
    pub fn envelope(&self) -> String {
        let mut map = Map::new();
        map.insert(MARKER.to_owned(), Value::from(VERSION));
        for (key, value) in [
            (GRADE, &self.grade),
            (NAME, &self.name),
            (ADDRESS, &self.address),
            (CA, &self.ca),
            (CERT, &self.cert),
            (KEY, &self.key),
        ] {
            map.insert(key.to_owned(), Value::String(value.clone()));
        }
        Value::Object(map).to_string()
    }

    /// **What may be said out loud about an enrollment**: which grade, under
    /// what name, dialling where. The three fields that are not material.
    ///
    /// It exists so that no caller has to decide which fields are safe to
    /// print — a decision that only has to be got wrong once, and that a
    /// terminal's scrollback then keeps.
    pub fn caption(&self) -> String {
        let Self {
            grade,
            name,
            address,
            ..
        } = self;
        format!("{name} — {grade} at {address}")
    }
}

/// Read the material. **Rung 1 throughout**: every field is required and every
/// refusal names the field, because a half-read enrollment is a symbol that
/// scans into a box that cannot dial.
pub(crate) fn enrolled(obj: &Map<String, Value>) -> Result<Enrolled, String> {
    Ok(Enrolled {
        grade: fields::text(obj, GRADE)?,
        name: fields::text(obj, NAME)?,
        address: fields::text(obj, ADDRESS)?,
        ca: fields::text(obj, CA)?,
        cert: fields::text(obj, CERT)?,
        key: fields::text(obj, KEY)?,
    })
}

#[cfg(test)]
mod tests;
