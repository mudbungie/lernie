//! The enrollment act end to end: what it says, and every way it can fail to.
//!
//! Split at the design-time budget on the seam the act itself has. [`said`] is
//! what a spent enrollment PRODUCES — the symbol, the line it encodes, the
//! entry a named destination gets, and the tree that is still untouched without
//! one. [`faults`] is every way the act does not get there: the engine
//! answering no, a frame this seat cannot read, a channel that will not open,
//! and the answer that crossed and came back to nobody. They change for
//! different reasons, which is the test that a seam is real.
//!
//! The harness they share stays here, and `spent` is the whole of it: one
//! scripted answer over a throwaway root, with the tree walked either side.

use serde_json::{Value, json};

use super::enroll;
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

/// Stand a wired root up with one scripted answer, and run the act on it with
/// no destination named — the ordinary path, which writes nothing.
fn spent(answer: Vec<Value>) -> (crate::cli::Verdict, Vec<String>, Vec<String>) {
    let scratch = Scratch::new();
    let _engine = wire::wired(&scratch, &wire::flat(), vec![answer]);
    let before = tree(scratch.path());
    let verdict = enroll(scratch.path(), "home", "phone-1", "foot", None, None);
    let after = tree(scratch.path());
    (verdict, before, after)
}

mod faults;
mod said;
