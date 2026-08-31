//! **The grade, read off this box's own certificate before anything is
//! dialled** (yog's `docs/REMOTE.md` §4.2; DESIGN §4.4).
//!
//! REMOTE §4.2 gives a certificate one of two grades and gives the grade one
//! home — *"the subject's organizational unit, read by the walk that already
//! reads the common name: `CN=<client>, OU=foot` is a foot"*. A seat is
//! **operator grade by definition**: *"a face that could not ask is not a
//! face"* (§2). So a seat configured with a foot-grade leaf is a
//! misconfiguration, and it is one this box can see in its own files.
//!
//! # This is a DIAGNOSIS and not an enforcement, and the difference decides
//! every line below
//!
//! Enforcement is the **engine's**, at the chokepoint where the client identity
//! is already spent for scoping, fail-closed and in band. A seat holding a
//! foot-grade leaf is therefore already refused, correctly, with or without
//! this file. Nothing here is a security property and nothing here may behave
//! as though it were.
//!
//! What is missing without it is a **sentence**. That refusal arrives as an
//! authorization answer from the far end, about a fault that is entirely this
//! box's own configuration — the operator pointed a seat at the wrong pair.
//! Reading the grade here turns it into a sentence about a file on this disk,
//! before a socket is opened.
//!
//! # So it identifies one fault; it never validates
//!
//! [`refusal`] answers `Some` **only** where it positively read `OU=foot`, and
//! `None` for everything else — bytes that are not a certificate, a subject it
//! cannot walk, an attribute in a string type it will not decode. That is the
//! whole shape of it, and it is what keeps a diagnostic aid from becoming an
//! outage: a walk that refused what it could not read would be a second, weaker
//! certificate parser standing in front of rustls, refusing leaves the engine
//! would have accepted. **Default-operator is REMOTE §4.2's own rule** — a
//! certificate minted before the grade existed keeps working — and reading it
//! any other way here would be this end inventing a policy the authority does
//! not have.
//!
//! The sibling foot component does the mirror of this and fails **closed**,
//! which is not a disagreement: its obligation is to carry a foot leaf and
//! refuse to be configured with anything else, so an unreadable leaf is a
//! refusal there and is silence here. The two ends answer different questions.
//!
//! # The walk, and why it is structural
//!
//! This crate links no certificate library beyond rustls' own PEM reader, so
//! the grade is read by a DER walk — structural ASN.1 rather than a byte
//! search, and the structure is the point: the **issuer** carries a common name
//! too and it comes FIRST, so a scan for the common-name object identifier
//! would answer the operator CA's name for every leaf on the box.
//!
//! What it reads, per RFC 5280:
//!
//! ```text
//! Certificate     ::= SEQUENCE { tbsCertificate, signatureAlgorithm, signature }
//! TBSCertificate  ::= SEQUENCE { [0] version OPTIONAL, serialNumber INTEGER,
//!                                signature, issuer, validity, subject, … }
//! Name            ::= SEQUENCE OF SET OF SEQUENCE { type OID, value ANY }
//! ```
//!
//! The optional `[0] version` is why `subject` is located **relative to the
//! serial number** rather than at a fixed index: the serial is the first field
//! certainly present, and `subject` is four constructed values past it. A
//! version-1 certificate and a version-3 one then take one path, not two.

use std::path::Path;

/// DER tags this walk names.
const INTEGER: u8 = 0x02;
const OID: u8 = 0x06;
/// `id-at-commonName` — ASN.1 `{joint-iso-itu-t(2) ds(5) attributeType(4)
/// commonName(3)}`, in its DER encoding. Spelled as bytes rather than as the
/// dotted arc string because the dotted form of four small arcs is
/// indistinguishable from an IPv4 address, to a reader and to `make leak-scan`.
const COMMON_NAME: [u8; 3] = [0x55, 0x04, 0x03];
/// `id-at-organizationalUnitName` — the same arc one attribute over, and the
/// home REMOTE §4.2 gives the grade.
const ORG_UNIT: [u8; 3] = [0x55, 0x04, 0x0b];
/// The organizational unit that says foot. One word, written by the operator's
/// CA or not at all.
const FOOT: &str = "foot";
/// How many constructed fields separate `serialNumber` from `subject`:
/// signature, issuer, validity, subject.
const SERIAL_TO_SUBJECT: usize = 4;

/// **The one misconfiguration this box can name on its own**, as the sentence
/// to answer with — or `None`, which is every other leaf and every byte string
/// this walk cannot read.
///
/// `at` is the file the bytes came out of, because the whole value of saying
/// this locally is naming the file the operator has to replace.
pub fn refusal(der: &[u8], at: &Path) -> Option<String> {
    if !says_foot(der) {
        return None;
    }
    // The client name, when the same subject states one. It is not required to
    // say the sentence — the file is what has to be replaced either way — and
    // a leaf that says `foot` and names nobody is exactly as wrong.
    let named = client(der).map_or_else(String::new, |name| format!(" {name:?}"));
    Some(format!(
        "{}: the leaf{named} is foot grade — a foot may advertise, wait and \
         complete, and nothing else (REMOTE §4.2), so a seat holding it can \
         ask the engine nothing. A seat is operator grade by definition. Mint \
         an operator-grade pair on the box that holds the CA and carry it here; \
         {}",
        at.display(),
        super::material::REMEDY
    ))
}

/// Whether the subject says foot.
fn says_foot(der: &[u8]) -> bool {
    subject(der).is_some_and(|name| attributes(name, ORG_UNIT).iter().any(|unit| unit == FOOT))
}

/// The subject common name — the client identity the engine reads back off the
/// presented certificate (REMOTE §2), so a seat that says its own name learned
/// it from the same bytes the engine will.
///
/// The **last** common name wins. A distinguished name is written most-general
/// first in DER and most-specific last (RFC 4514 renders it reversed), so the
/// final one is the leaf's own.
fn client(der: &[u8]) -> Option<String> {
    attributes(subject(der)?, COMMON_NAME).pop()
}

/// The `Name` bytes of the certificate's **subject** — located relative to the
/// serial number, for the reason the module doc gives.
fn subject(der: &[u8]) -> Option<&[u8]> {
    let (_, certificate, _) = tlv(der)?;
    let (_, tbs, _) = tlv(certificate)?;
    let fields = elements(tbs);
    let serial = fields.iter().position(|(tag, _)| *tag == INTEGER)?;
    let &(_, subject) = fields.get(serial + SERIAL_TO_SUBJECT)?;
    Some(subject)
}

/// Every value of attribute `oid` in a `Name`, in DER order, decoded as UTF-8.
/// Every string type these attributes are minted in — `UTF8String`,
/// `PrintableString`, `IA5String` — is UTF-8 or a subset of it, and one that is
/// not (`BMPString` is UTF-16) fails the decode and is skipped rather than
/// mis-read.
fn attributes(name: &[u8], oid: [u8; 3]) -> Vec<String> {
    let mut found = Vec::new();
    for (_, rdn) in elements(name) {
        for (_, attribute) in elements(rdn) {
            let parts = elements(attribute);
            let (Some(&(tag, kind)), Some(&(_, value))) = (parts.first(), parts.get(1)) else {
                continue;
            };
            if tag != OID || kind != oid {
                continue;
            }
            if let Ok(text) = std::str::from_utf8(value) {
                found.push(text.to_owned());
            }
        }
    }
    found
}

/// One DER type-length-value off the front of `bytes`: its tag, its contents,
/// and what follows it. `None` for a truncated header, a truncated value, or a
/// length DER does not permit — the indefinite form (`0x80`), which BER allows
/// and DER forbids, and a length wider than this walk will serve.
fn tlv(bytes: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    let tag = *bytes.first()?;
    let first = *bytes.get(1)?;
    let (len, header) = if first < 0x80 {
        (usize::from(first), 2)
    } else {
        let width = usize::from(first & 0x7f);
        if width == 0 || width > 4 {
            return None;
        }
        let mut len: usize = 0;
        for i in 0..width {
            len = (len << 8) | usize::from(*bytes.get(2 + i)?);
        }
        (len, 2 + width)
    };
    // Saturating rather than checked: an unreachable overflow arm is an
    // untestable branch, and a saturated end simply fails the read below.
    let end = header.saturating_add(len);
    let value = bytes.get(header..end)?;
    Some((tag, value, bytes.get(end..).unwrap_or_default()))
}

/// Every element of a constructed value, in order. A trailing byte run that is
/// not a whole TLV ends the walk — a malformed tail yields the elements read
/// before it, which is what makes every read above total.
fn elements(mut body: &[u8]) -> Vec<(u8, &[u8])> {
    let mut out = Vec::new();
    while let Some((tag, value, rest)) = tlv(body) {
        out.push((tag, value));
        body = rest;
    }
    out
}

#[cfg(test)]
mod tests;
