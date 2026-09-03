//! **The standing assertion, and the other direction for every arm of it.**
//!
//! One test does the standing work: it walks the real window in every world at
//! every width the layout promises a shape for, collects the `act:` tags off
//! the accessibility tree, reads the ledger out loud, and holds the four
//! assertions in `super`.
//!
//! Everything else here is those assertions' **other direction** — a synthetic
//! subject each arm must complain about, and a parser refusal for each line
//! shape the ledger's subset excludes. A gate is not shown to work by a green
//! suite; a gate that matches nothing is green forever.

use std::collections::BTreeSet;

use super::{complaints, exempt, inventory, roster};
use crate::snapshot::{SIZES, promised, seat, worlds};

/// **The walk**: every tag the tree offers, over every world at every judged
/// width, printed per screen so a failure says which screen was short.
fn walked() -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for world in worlds::all() {
        for (size, width, height) in SIZES {
            if !promised(width) {
                continue;
            }
            let harness = seat(world.model.clone(), width, height);
            let seen = inventory(&harness);
            println!("{} at {size}: {seen:?}", world.name);
            found.extend(seen);
        }
    }
    found
}

/// **The gate** (yog's `docs/PARITY.md` §5).
#[test]
fn every_control_op_is_either_tagged_here_or_recorded_absent_with_a_citation() {
    let roster = roster::roster();
    // **The other direction**: a roster that classed nothing a control would
    // satisfy every assertion below forever, which is how a gate like this
    // actually dies.
    assert!(
        !roster.control.is_empty(),
        "the vendored help fixture classes no row a control — the read is broken, not the roster"
    );
    let exemptions = exempt::read();
    let found = walked();
    println!(
        "parity: {} of the roster's {} control-classed ops carry a control here: {found:?}",
        found.intersection(&roster.control).count(),
        roster.control.len()
    );
    // **The roster of absences is read out on every run**, never only on a
    // failure: an absence that is allowed is still an absence, and one nobody
    // is ever shown is one nobody deletes.
    for row in &exemptions {
        println!("parity: {} is ABSENT — {}", row.op, row.why);
    }
    let said = complaints(&roster, &exemptions, &found);
    assert!(
        said.is_empty(),
        "this seat and the roster have drifted:\n{}",
        said.join("\n")
    );
}

#[test]
fn a_help_row_is_read_for_its_op_and_its_class_and_refused_when_it_states_neither() {
    let control = serde_json::json!({ "verb": "message", "surface": "control" });
    assert_eq!(
        roster::classify(&control).expect("a control row"),
        ("message".to_owned(), true)
    );
    let machine = serde_json::json!({ "verb": "capture", "surface": "machine" });
    assert_eq!(
        roster::classify(&machine).expect("a machine row"),
        ("capture".to_owned(), false)
    );
    let unclassified = serde_json::json!({ "verb": "message" });
    let said = roster::classify(&unclassified).expect_err("no classification");
    assert!(said.contains("refresh the corpus"), "{said}");
    let unknown = serde_json::json!({ "verb": "message", "surface": "ornament" });
    let said = roster::classify(&unknown).expect_err("an unreadable classification");
    assert!(said.contains("\"ornament\""), "{said}");
    let nameless = serde_json::json!({ "surface": "control" });
    let said = roster::classify(&nameless).expect_err("no verb");
    assert!(said.contains("names no verb"), "{said}");
}

/// A roster made up on the spot, so each arm can be asked directly.
fn stated(control: &[&str], machine: &[&str]) -> roster::Roster {
    let control: BTreeSet<String> = control.iter().map(|op| (*op).to_owned()).collect();
    let mut ops = control.clone();
    ops.extend(machine.iter().map(|op| (*op).to_owned()));
    roster::Roster { ops, control }
}

/// A ledger made up on the spot.
fn recorded(rows: &[&str]) -> Vec<exempt::Exemption> {
    rows.iter()
        .map(|op| exempt::Exemption {
            op: (*op).to_owned(),
            why: "bl-0000 — a reason".to_owned(),
        })
        .collect()
}

/// An inventory made up on the spot.
fn tagged(ops: &[&str]) -> BTreeSet<String> {
    ops.iter().map(|op| (*op).to_owned()).collect()
}

#[test]
fn a_control_op_with_neither_a_tag_nor_a_line_is_the_complaint() {
    let said = complaints(
        &stated(&["message", "stop"], &[]),
        &[],
        &tagged(&["message"]),
    );
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said.concat().contains("\"stop\""), "{said:?}");
    assert!(said.concat().contains("nor a line for it"), "{said:?}");
}

#[test]
fn a_tag_naming_no_op_in_the_roster_is_a_complaint() {
    let said = complaints(
        &stated(&["message"], &[]),
        &[],
        &tagged(&["message", "mesage"]),
    );
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said.concat().contains("act:mesage"), "{said:?}");
    assert!(said.concat().contains("mistyped tag"), "{said:?}");
}

#[test]
fn an_exemption_the_roster_no_longer_classes_a_control_has_rotted() {
    let said = complaints(
        &stated(&["message"], &["capture"]),
        &recorded(&["capture"]),
        &tagged(&["message"]),
    );
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said.concat().contains("has rotted"), "{said:?}");
}

#[test]
fn an_exemption_for_an_op_that_is_now_tagged_is_stale() {
    let said = complaints(
        &stated(&["message"], &[]),
        &recorded(&["message"]),
        &tagged(&["message"]),
    );
    assert_eq!(said.len(), 1, "{said:?}");
    assert!(said.concat().contains("is stale"), "{said:?}");
}

#[test]
fn a_seat_whose_ledger_covers_exactly_what_it_does_not_surface_is_quiet() {
    let said = complaints(
        &stated(&["message", "stop"], &["capture"]),
        &recorded(&["stop"]),
        &tagged(&["message"]),
    );
    assert!(said.is_empty(), "{said:?}");
}

/// The whole of the ledger's grammar, in one subject.
const LEDGER: &str = "\n# a comment\n\nstop = \"bl-213c — no conversation acts\"\n";

#[test]
fn the_ledger_reads_a_quoted_reason_past_comments_and_blank_lines() {
    let rows = exempt::parse(LEDGER).expect("the subset");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].op, "stop");
    assert!(rows[0].why.starts_with("bl-213c"), "{}", rows[0].why);
}

#[test]
fn an_empty_ledger_is_the_success_state_and_not_a_refusal() {
    assert!(
        exempt::parse("# nothing absent\n")
            .expect("no rows")
            .is_empty()
    );
}

#[test]
fn every_shape_the_subset_excludes_is_refused_and_names_its_line() {
    for (text, why) in [
        ("[absent]\n", "not `op = \"reason\"`"),
        ("two words = \"bl-1\"\n", "names no single op"),
        (" = \"bl-1\"\n", "names no single op"),
        ("stop = bl-1\n", "not a quoted string"),
        ("stop = \"because\"\n", "cites no ball"),
    ] {
        let said = exempt::parse(text).expect_err(text);
        assert!(said.contains("line 1"), "{said}");
        assert!(said.contains(why), "{said}");
    }
    let said = exempt::parse("stop = \"bl-1\"\nstop = \"bl-2\"\n").expect_err("a repeat");
    assert!(said.contains("line 2"), "{said}");
    assert!(said.contains("recorded twice"), "{said}");
}

/// **The committed ledger is read by the real reader**, so the file's own
/// syntax is judged by the gate rather than only by the test above.
#[test]
fn the_committed_ledger_parses_and_every_row_cites_a_ball() {
    let rows = exempt::read();
    assert!(!rows.is_empty(), "parity.toml records nothing");
    for row in &rows {
        assert!(row.why.contains("bl-"), "{} cites nothing", row.op);
    }
}
