//! The enrollment act end to end, and the thing it must not do.

use serde_json::{Value, json};

use super::{KEPT, enroll};
use crate::cli::Stream;
use crate::test_support::{Scratch, wire};

/// The fabricated material an engine answers with. Every string is marked
/// `notreal`, and the key deliberately carries no private-key banner: the
/// disclosure gate reads every committed byte of this tree.
const CA: &str = "-----BEGIN CERTIFICATE-----\nnotreal-ca\n-----END CERTIFICATE-----\n";
const CERT: &str = "-----BEGIN CERTIFICATE-----\nnotreal-leaf\n-----END CERTIFICATE-----\n";
const KEY: &str = "-----BEGIN notreal KEY-----\nnotreal-key\n-----END notreal KEY-----\n";

/// One `enrolled` answer.
fn minted() -> Value {
    json!({
        "ok": true,
        "kind": "enrolled",
        "grade": "foot",
        "name": "phone-1",
        "address": "engine.invalid:7737",
        "ca": CA,
        "cert": CERT,
        "key": KEY,
    })
}

/// Every path under `at`, relative and sorted, with each file's length — a
/// snapshot a test can compare against itself.
///
/// **The length matters**: a defect that overwrote a file rather than adding
/// one would leave the path set identical, and this is the cheapest thing that
/// still sees it.
fn tree(at: &std::path::Path) -> Vec<String> {
    let mut found = Vec::new();
    walk(at, at, &mut found);
    found.sort();
    found
}

fn walk(root: &std::path::Path, at: &std::path::Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(at) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(root, &path, out);
        } else {
            let len = std::fs::metadata(&path).map_or(0, |held| held.len());
            let shown = path.strip_prefix(root).unwrap_or(&path).display();
            out.push(format!("{shown} ({len})"));
        }
    }
}

/// Stand a wired root up with one scripted answer, and run the act on it.
fn spent(answer: Vec<Value>) -> (crate::cli::Verdict, Vec<String>, Vec<String>) {
    let scratch = Scratch::new();
    let _engine = wire::wired(&scratch, &wire::flat(), vec![answer]);
    let before = tree(scratch.path());
    let verdict = enroll(scratch.path(), "home", "phone-1", "foot");
    let after = tree(scratch.path());
    (verdict, before, after)
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

/// **The material is not printed**, which is the other half of the same rule:
/// stdout is a scrollback, a shell history and whatever it was piped into. What
/// the act prints is the picture and the three fields that are not secret.
#[test]
fn the_answer_is_drawn_and_never_spelled_out() {
    let (verdict, ..) = spent(vec![minted()]);
    assert_eq!(verdict.stream, Stream::Out);
    for secret in ["notreal-key", "notreal-leaf", "notreal-ca", "BEGIN"] {
        assert!(
            !verdict.text.contains(secret),
            "{secret} reached stdout:\n{}",
            verdict.text
        );
    }
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

/// The gesture that crossed is the row's own envelope, addressed at the wall —
/// so the window and the command line spend one serialization.
#[test]
fn the_gesture_that_crossed_is_the_verb_table_s_own() {
    let scratch = Scratch::new();
    let engine = wire::wired(&scratch, &wire::flat(), vec![vec![minted()]]);
    let _ = enroll(scratch.path(), "home", "phone-1", "foot");
    let heard = engine.heard();
    let request = heard.last().expect("the engine was handed a request");
    assert_eq!(
        request,
        &crate::verbs::enroll("home".to_owned(), "phone-1".to_owned(), "foot".to_owned())
    );
}

/// **A refusal is the engine answering**, so it is this run's product: it goes
/// to stdout with the rest of the stream and only the exit code says no. It
/// carries no material to withhold.
#[test]
fn a_refusal_is_answered_rather_than_drawn() {
    let (verdict, ..) = spent(vec![json!({"ok": false, "error": "not operator-grade"})]);
    assert_eq!(verdict.stream, Stream::Out);
    assert_eq!(verdict.code, 1);
    assert!(
        verdict.text.contains("not operator-grade"),
        "{}",
        verdict.text
    );
}

/// A frame this seat cannot read is a statement about **this seat**, and it
/// fails rather than answering — there is nothing to draw.
#[test]
fn an_unreadable_frame_fails_in_this_seat_s_own_words() {
    let (verdict, ..) = spent(vec![json!({"ok": true, "kind": "enrolled"})]);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(verdict.text.contains("grade"), "{}", verdict.text);
}

/// **A well-formed answer of the wrong kind is not "unreadable"** — this seat
/// read it perfectly well. Saying otherwise would send an operator to upgrade
/// something that is fine.
#[test]
fn an_answer_of_another_kind_says_nothing_was_minted() {
    let (verdict, ..) = spent(vec![json!({"ok": true, "kind": "nudged"})]);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(verdict.text.contains("did not mint"), "{}", verdict.text);
}

/// A channel that will not answer is a fact about this box or the far end, and
/// earns the sentence alone.
#[test]
fn a_root_with_no_channel_fails_before_anything_is_asked() {
    let scratch = Scratch::new();
    let verdict = enroll(scratch.path(), "home", "phone-1", "foot");
    assert_eq!(verdict.stream, Stream::Err);
    assert_eq!(verdict.code, 1);
    assert_eq!(tree(scratch.path()), Vec::<String>::new());
}

/// **Material too big for any symbol is a refusal, not a smaller picture.**
/// REMOTE §8.4 measures the envelope at about 1567 bytes against a 2331-byte
/// ceiling, so this is a recipe that moved — an RSA key, a longer chain — and
/// saying the size is what makes that legible rather than mysterious.
#[test]
fn material_too_big_for_a_symbol_says_so_and_draws_nothing() {
    let mut oversized = minted();
    oversized["ca"] = Value::String("notreal".repeat(400));
    let (verdict, before, after) = spent(vec![oversized]);
    assert_eq!(verdict.stream, Stream::Err);
    assert!(verdict.text.contains("will not fit"), "{}", verdict.text);
    assert!(
        verdict.text.contains("2331"),
        "the ceiling: {}",
        verdict.text
    );
    assert!(!verdict.text.contains("notreal"), "it printed the material");
    assert_eq!(before, after, "a refusal still wrote something");
}

/// An engine that closes without a frame has answered nothing, and that is its
/// own sentence rather than a reading of an absent object.
#[test]
fn an_empty_stream_says_the_engine_answered_nothing() {
    let (verdict, ..) = spent(Vec::new());
    assert!(verdict.text.contains("nothing at all"), "{}", verdict.text);
}
