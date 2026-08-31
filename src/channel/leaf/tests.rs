//! The grade: the one leaf this walk names, everything it leaves alone, and the
//! DER it is prepared to be handed.

use std::path::Path;

use rustls::pki_types::CertificateDer;
use rustls::pki_types::pem::PemObject;

use super::{FOOT, ORG_UNIT, attributes, client, elements, refusal, subject, tlv};
use crate::channel::material::{CHAIN, REMEDY};
use crate::test_support::{Scratch, mint};

/// This box's own leaf, as DER, out of a scratch directory the mint filled.
fn leaf_of(dir: &Path) -> Vec<u8> {
    CertificateDer::from_pem_file(dir.join(CHAIN))
        .expect("a leaf")
        .to_vec()
}

/// **The whole point of the file**: the fault is named here, about a file on
/// this disk, before a socket is opened — instead of arriving as an
/// authorization sentence from the far end about a fault that is entirely this
/// box's own configuration.
#[test]
fn a_foot_grade_leaf_is_named_as_this_box_s_own_misconfiguration() {
    let scratch = Scratch::new();
    mint::material(scratch.path());
    mint::foot(scratch.path(), true);
    let at = scratch.join(CHAIN);
    let said = refusal(&leaf_of(scratch.path()), &at).expect("refused");
    assert!(said.contains(&at.display().to_string()), "{said}");
    assert!(said.contains(mint::SEAT_NAME), "{said}");
    assert!(said.contains("foot grade"), "{said}");
    assert!(said.contains("operator grade by definition"), "{said}");
    assert!(
        said.contains(REMEDY),
        "the remedy is the operator's act: {said}"
    );
}

/// A leaf that says foot and names nobody is exactly as wrong, and the sentence
/// still has a file to point at — which is the half of it that matters.
#[test]
fn a_foot_grade_leaf_that_names_no_client_still_names_the_file() {
    let scratch = Scratch::new();
    mint::material(scratch.path());
    mint::foot(scratch.path(), false);
    let at = scratch.join(CHAIN);
    let said = refusal(&leaf_of(scratch.path()), &at).expect("refused");
    assert!(said.contains("the leaf is foot grade"), "{said}");
    assert!(said.contains(&at.display().to_string()), "{said}");
}

/// **Default-operator is the authority's own rule**: a subject that says
/// nothing about a grade is the whole boundary, so the ordinary minted pair —
/// and any certificate minted before the grade existed — passes without a
/// word.
#[test]
fn an_operator_grade_leaf_says_nothing() {
    let scratch = Scratch::new();
    mint::material(scratch.path());
    let der = leaf_of(scratch.path());
    assert_eq!(refusal(&der, &scratch.join(CHAIN)), None);
    assert_eq!(
        client(&der).as_deref(),
        Some(mint::SEAT_NAME),
        "and the subject's own name is the last common name in it"
    );
}

/// **It identifies one fault; it never validates.** Everything this walk cannot
/// read is silence, because a walk that refused it would be a second, weaker
/// certificate parser standing in front of rustls — refusing leaves the engine
/// would have accepted, over a check that was never a security property.
#[test]
fn nothing_this_walk_cannot_read_is_ever_refused() {
    let at = Path::new("client.pem");
    for bytes in [
        b"".as_slice(),
        b"not a certificate at all".as_slice(),
        // A well-formed outer SEQUENCE whose body is not a TBSCertificate.
        &[0x30, 0x03, 0x02, 0x01, 0x07],
        // A TBSCertificate that runs out before `subject`.
        &[0x30, 0x05, 0x30, 0x03, 0x02, 0x01, 0x07],
    ] {
        assert_eq!(refusal(bytes, at), None, "{bytes:?}");
        assert_eq!(client(bytes), None, "{bytes:?}");
    }
}

/// A TBSCertificate with no serial number at all has no anchor to count from,
/// so there is nothing to read and nothing to say.
#[test]
fn a_body_with_no_serial_number_has_no_subject_to_find() {
    // SEQUENCE { SEQUENCE { SEQUENCE {}, SEQUENCE {}, SEQUENCE {} } } — three
    // constructed fields and not an INTEGER among them.
    let der = [0x30, 0x08, 0x30, 0x06, 0x30, 0x00, 0x30, 0x00, 0x30, 0x00];
    assert_eq!(subject(&der), None);
}

/// The DER lengths this walk will and will not take. The refusals are the
/// forms DER itself forbids or this walk will not serve, and each is answered
/// by reading nothing rather than by guessing.
#[test]
fn the_length_forms_this_walk_takes_and_the_ones_it_refuses() {
    assert_eq!(tlv(&[]), None, "no tag");
    assert_eq!(tlv(&[0x30]), None, "no length");
    assert_eq!(tlv(&[0x30, 0x80]), None, "the indefinite form DER forbids");
    assert_eq!(tlv(&[0x30, 0x85, 1, 1, 1, 1, 1]), None, "wider than served");
    assert_eq!(tlv(&[0x30, 0x81]), None, "a truncated long form");
    assert_eq!(tlv(&[0x30, 0x02, 0xaa]), None, "a truncated value");
    assert_eq!(
        tlv(&[0x30, 0x81, 0x01, 0xaa]),
        Some((0x30, [0xaa].as_slice(), [].as_slice())),
        "the long form for one byte"
    );
    assert_eq!(
        tlv(&[0x02, 0x01, 0x07, 0xff]),
        Some((0x02, [0x07].as_slice(), [0xff].as_slice())),
        "and what follows a value is handed back whole"
    );
}

/// **A malformed tail ends the walk rather than failing it**, which is what
/// makes every read above total: the elements before it are still what the
/// bytes said.
#[test]
fn a_malformed_tail_yields_the_elements_read_before_it() {
    assert_eq!(
        elements(&[0x02, 0x01, 0x07, 0x02, 0x09, 0xaa]),
        vec![(0x02, [0x07].as_slice())]
    );
    assert!(elements(&[]).is_empty());
}

/// The two attribute shapes that are skipped rather than mis-read: a value in a
/// string type that is not UTF-8, and a sequence whose first element is not an
/// object identifier at all.
#[test]
fn an_attribute_this_walk_cannot_decode_is_skipped_and_never_guessed() {
    // Name ::= SEQUENCE OF SET OF SEQUENCE { type OID, value ANY }
    let rdn = |body: Vec<u8>| {
        let mut set = vec![0x31, u8::try_from(body.len()).unwrap_or(0)];
        set.extend(body);
        set
    };
    let attribute = |body: Vec<u8>| {
        let mut seq = vec![0x30, u8::try_from(body.len()).unwrap_or(0)];
        seq.extend(body);
        seq
    };
    let oid = [&[0x06, 0x03][..], &ORG_UNIT[..]].concat();

    // A BMPString (UTF-16BE) value under the right object identifier — here
    // `f\u{f6}`, whose second code unit leaves bytes no UTF-8 decoder accepts.
    let wide = attribute([oid.clone(), vec![0x1e, 0x04, 0x00, 0x66, 0x00, 0xf6]].concat());
    // The right value under something that is not an object identifier.
    let untyped = attribute(vec![0x02, 0x01, 0x07, 0x0c, 0x04, 0x66, 0x6f, 0x6f, 0x74]);
    // And the readable one, so the walk is proved to be looking at all.
    let good = attribute([oid, vec![0x0c, 0x04, 0x66, 0x6f, 0x6f, 0x74]].concat());

    let name: Vec<u8> = [rdn(wide), rdn(untyped), rdn(good)].concat();
    assert_eq!(attributes(&name, ORG_UNIT), vec![FOOT.to_owned()]);
}
