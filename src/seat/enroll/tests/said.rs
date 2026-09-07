//! **What a spent enrollment produces**: one artifact, said every way a box can
//! take it (bl-1554, bl-a8fd).
//!
//! Split from [`super::faults`] at the design-time budget on the seam the
//! module doc above draws — this is the act arriving, and that is every way it
//! does not.

use super::super::{ELSEWHERE, KEPT};
use super::{CA, CERT, KEY, acted, as_form, minted, spent, tree};
use crate::cli::Stream;
use crate::render::Form;
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

/// **`--into` files the entry, and the material then reaches stdout by no
/// route at all** (bl-1554 for the files, bl-768a for the silence). A foot is a
/// box the operator is not sitting at, so the four files are the whole of what
/// they would otherwise do by hand off a picture they had to decode first —
/// and having asked for files, the operator did not also ask for a private key
/// in a scrollback nothing can shred.
#[test]
fn a_named_destination_gets_the_four_files_and_the_material_is_not_said() {
    let scratch = Scratch::new();
    let _engine = wire::wired(&scratch, &wire::flat(), vec![vec![minted()]]);
    let into = scratch.path().join("carry");
    let (verdict, _) = acted(
        scratch.path(),
        "phone-1",
        "foot",
        None,
        Some(&into),
        Form::Rendered,
    );
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
    // The key itself, its PEM banner, the envelope that carries it and the
    // picture of that envelope: four ways the same bytes could reach a
    // terminal, and the assertion is over all four rather than over the one
    // this fix happened to remove.
    for withheld in [KEY, "notreal-key", "-----BEGIN", "yog-enroll"] {
        assert!(
            !verdict.text.contains(withheld),
            "{withheld:?} reached stdout after the material was filed:\n{}",
            verdict.text
        );
    }
    for glyph in ['\u{2588}', '\u{2580}'] {
        assert!(
            !verdict.text.contains(glyph),
            "the symbol was drawn beside the files:\n{}",
            verdict.text
        );
    }
    // What IS said: who was minted, where the files went, and that the key is
    // in them and nowhere else.
    assert!(verdict.text.contains("phone-1"), "{}", verdict.text);
    assert!(verdict.text.contains(ELSEWHERE), "{}", verdict.text);
    assert!(!verdict.text.contains(KEPT), "{}", verdict.text);
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
    let _ = acted(
        scratch.path(),
        "phone-1",
        "foot",
        None,
        None,
        Form::Rendered,
    );
    let heard = engine.heard();
    let request = heard.last().expect("the engine was handed a request");
    assert_eq!(
        request,
        &crate::verbs::enroll(
            "home".to_owned(),
            "phone-1".to_owned(),
            "foot".to_owned(),
            None,
        )
    );
}

/// **A stated route crosses** (bl-971c). It is the field that produces no error
/// message anywhere when it is wrong — a foot handed the engine's own LAN
/// address dials nothing, prints nothing and waits, while the engine holds the
/// registration and the roster shows it absent — so what is asserted is that
/// the word an operator typed reaches the wire under §8.4's own name.
#[test]
fn a_stated_route_crosses_as_the_enrollment_s_address() {
    let scratch = Scratch::new();
    let engine = wire::wired(&scratch, &wire::flat(), vec![vec![minted()]]);
    let (verdict, _) = acted(
        scratch.path(),
        "phone-1",
        "foot",
        Some("host.containers.internal:7773".to_owned()),
        None,
        Form::Rendered,
    );
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    let heard = engine.heard();
    let request = heard.last().expect("the engine was handed a request");
    assert_eq!(
        request.get(crate::verbs::ADDRESS),
        Some(&serde_json::Value::String(
            "host.containers.internal:7773".to_owned()
        )),
        "{request}"
    );
}

/// **`--json` prints the envelope and nothing else** (bl-ac76). One line, and
/// it parses: the whole product of a run a script can capture with `head`, a
/// `read`, or a pipe into a formatter. It printed the human rendering under
/// the flag — an ANSI-coloured symbol ahead of the line — so the first lines a
/// script kept were a picture of material the engine had already shredded, and
/// the name was spent.
#[test]
fn the_machine_form_is_the_envelope_alone() {
    let (verdict, warned) = as_form(None, Form::Json);
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert_eq!(verdict.stream, Stream::Out);
    assert!(warned.is_empty(), "{warned:?}");
    let parsed: serde_json::Value =
        serde_json::from_str(&verdict.text).expect("the whole of stdout parses as the envelope");
    assert_eq!(
        parsed.get("yog-enroll").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(verdict.text.lines().count(), 1, "{}", verdict.text);
    // Not the picture, not the caption, not the note about keeping no copy.
    for absent in ["\u{2588}", "\u{2580}", "\u{1b}[", KEPT, " — foot at "] {
        assert!(
            !verdict.text.contains(absent),
            "{absent:?} reached the frame stream: {}",
            verdict.text
        );
    }
}

/// **The two rules compose: a machine form that FILED says nothing at all**
/// (bl-ac76 over bl-768a). The material went to the four files, so printing it
/// would be the second copy bl-768a withholds — and the receipt naming them is
/// prose about a file rather than a frame, so it is a diagnosis. stdout under
/// `--json` therefore carries the envelope exactly when the material was not
/// written down.
#[test]
fn a_machine_form_that_filed_says_nothing_on_stdout() {
    let scratch = Scratch::new();
    let _engine = wire::wired(&scratch, &wire::flat(), vec![vec![minted()]]);
    let into = scratch.path().join("carry");
    let (verdict, warned) = acted(
        scratch.path(),
        "phone-1",
        "foot",
        None,
        Some(&into),
        Form::Json,
    );
    assert_eq!(verdict.code, 0, "{}", verdict.text);
    assert_eq!(verdict.text, "", "{}", verdict.text);
    assert_eq!(tree(&into).len(), 4, "the four files were still laid down");
    let told = warned.concat();
    assert!(told.contains("filed as an entry"), "{told}");
    assert!(told.contains(ELSEWHERE), "{told}");
    assert!(
        !told.contains("yog-enroll"),
        "the material was said: {told}"
    );
}

/// **A destination that will not take the material still says it** (bl-768a),
/// and under `--json` it says exactly the envelope (bl-ac76): the reason is on
/// the other stream, where it cannot corrupt the one capture a script gets of
/// an act that cannot be repeated.
#[test]
fn a_refused_destination_leaves_the_machine_form_intact() {
    let scratch = Scratch::new();
    let _engine = wire::wired(&scratch, &wire::flat(), vec![vec![minted()]]);
    let blocked = scratch.path().join("occupied");
    std::fs::write(&blocked, "notreal").expect("the blocker was written");
    let (verdict, warned) = acted(
        scratch.path(),
        "phone-1",
        "foot",
        None,
        Some(&blocked.join("under")),
        Form::Json,
    );
    assert_eq!(verdict.code, 1, "{}", verdict.text);
    assert_eq!(verdict.stream, Stream::Out);
    assert_eq!(verdict.text.lines().count(), 1, "{}", verdict.text);
    assert!(verdict.text.contains("yog-enroll"), "{}", verdict.text);
    assert!(warned.concat().contains("not filed"), "{warned:?}");
}
