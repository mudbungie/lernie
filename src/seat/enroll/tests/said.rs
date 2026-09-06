//! **What a spent enrollment produces**: one artifact, said every way a box can
//! take it (bl-1554, bl-a8fd).
//!
//! Split from [`super::faults`] at the design-time budget on the seam the
//! module doc above draws — this is the act arriving, and that is every way it
//! does not.

use super::super::{KEPT, enroll};
use super::{CA, CERT, KEY, minted, spent, tree};
use crate::cli::Stream;
use crate::test_support::{Scratch, wire};

/// **The picture and the line are one artifact, and both are said** (bl-1554,
/// bl-a8fd). The symbol is what a camera takes; the line is what the android
/// seat's own paste box asks for, verbatim — *one line of JSON beginning
/// `{"yog-enroll": 1`*. They are the same bytes, so the assertion is that the
/// text carries the envelope this seat encoded and not a re-spelling of it.
///
/// It replaces the assertion that the material never reached stdout, which was
/// the defect rather than the rule: what §4.15 rules out is a copy nobody
/// chose, and a photograph of a private key is a private key.
#[test]
fn the_symbol_and_the_line_it_encodes_are_both_said() {
    let (verdict, ..) = spent(vec![minted()]);
    assert_eq!(verdict.stream, Stream::Out);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    let envelope = crate::reply::enrolled::Enrolled {
        grade: "foot".to_owned(),
        name: "phone-1".to_owned(),
        address: "engine.invalid:7737".to_owned(),
        ca: CA.to_owned(),
        cert: CERT.to_owned(),
        key: KEY.to_owned(),
    }
    .envelope();
    // REMOTE §8.4's marker and version, and the two fields that do not travel.
    // Key order is `serde_json`'s own (sorted) and is not semantic — a scanner
    // parses the object — so the marker is asserted as a member rather than as
    // a prefix, whatever a paste box's hint text says it begins with.
    assert!(envelope.contains(r#""yog-enroll":1"#), "{envelope}");
    for absent in [r#""ok""#, r#""kind""#] {
        assert!(!envelope.contains(absent), "{absent} travelled: {envelope}");
    }
    assert!(
        verdict.text.contains(&envelope),
        "the envelope was not said:\n{}",
        verdict.text
    );
    // One line, so an operator can take it with a `tail -n` or a mouse.
    assert!(
        verdict.text.lines().any(|line| line == envelope),
        "the envelope was said across more than one line"
    );
    assert!(verdict.text.contains("phone-1"), "{}", verdict.text);
    assert!(
        verdict.text.contains("engine.invalid:7737"),
        "{}",
        verdict.text
    );
    assert!(verdict.text.contains(KEPT), "{}", verdict.text);
    // The picture itself: the half-block glyphs the terminal rendering draws.
    assert!(
        verdict.text.contains('\u{2588}') || verdict.text.contains('\u{2580}'),
        "no symbol was drawn:\n{}",
        verdict.text
    );
}

/// **`--into` files the entry, and the act still says everything it says
/// without one** (bl-1554). A foot is a box the operator is not sitting at, so
/// the four files are the whole of what they would otherwise do by hand off a
/// picture they had to decode first.
#[test]
fn a_named_destination_gets_the_four_files_and_the_material_is_still_said() {
    let scratch = Scratch::new();
    let _engine = wire::wired(&scratch, &wire::flat(), vec![vec![minted()]]);
    let into = scratch.path().join("carry");
    let verdict = enroll(scratch.path(), "home", "phone-1", "foot", Some(&into));
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert_eq!(verdict.stream, Stream::Out);
    assert!(
        verdict.text.contains("filed as an entry"),
        "{}",
        verdict.text
    );
    assert_eq!(
        tree(&into),
        vec![
            format!("address ({})", "engine.invalid:7737\n".len()),
            format!("ca.pem ({})", CA.len()),
            format!("client.key ({})", KEY.len()),
            format!("client.pem ({})", CERT.len()),
        ]
    );
    // Still said, so a run that files is not a run that withholds.
    assert!(verdict.text.contains("yog-enroll"), "{}", verdict.text);
}

/// **Nothing is written down. Anywhere.**
///
/// The assertion is over the **tree** rather than over the paths this code
/// happens to know about, because a defect here is precisely a path nobody
/// thought of — a cache, a log, a temporary file, a state root somebody added
/// later. It walks the whole root before and after and compares.
#[test]
fn the_act_writes_no_file_at_all() {
    let (verdict, before, after) = spent(vec![minted()]);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert_eq!(before, after, "the enrollment left something on the disk");
}

/// The gesture that crossed is the row's own envelope, addressed at the wall —
/// so the window and the command line spend one serialization.
#[test]
fn the_gesture_that_crossed_is_the_verb_table_s_own() {
    let scratch = Scratch::new();
    let engine = wire::wired(&scratch, &wire::flat(), vec![vec![minted()]]);
    let _ = enroll(scratch.path(), "home", "phone-1", "foot", None);
    let heard = engine.heard();
    let request = heard.last().expect("the engine was handed a request");
    assert_eq!(
        request,
        &crate::verbs::enroll("home".to_owned(), "phone-1".to_owned(), "foot".to_owned())
    );
}
