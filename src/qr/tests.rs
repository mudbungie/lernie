//! **The encoder against an oracle it did not write**, and the two things a
//! matrix comparison cannot say.
//!
//! [`oracle`] holds three whole symbols from an independent implementation and
//! [`tables`] holds what that implementation says about all forty versions.
//! Between them the test is: encode the same bytes, get the same modules.
//!
//! Both directions, as everywhere here. A comparison that enumerated nothing —
//! an empty fixture, a walk that never ran — would pass forever, so the
//! enumerating tests assert their own count.

use super::{LIMIT, Symbol, TooLong};

mod oracle;
mod tables;

/// The payload the oracle's per-version symbols carry: a deterministic byte
/// rule rather than forty literals, stated once here and once in the generator
/// that produced the fixtures.
fn payload(version: usize, len: usize) -> Vec<u8> {
    (0..len)
        .map(|at| u8::try_from((at * 31 + version) % 256).unwrap_or(0))
        .collect()
}

/// A symbol rendered the way the oracle writes one: `#` dark, `.` light, no
/// quiet zone.
fn as_oracle(symbol: &Symbol) -> String {
    (0..symbol.side())
        .map(|y| {
            (0..symbol.side())
                .map(|x| if symbol.dark(x, y) { '#' } else { '.' })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Where two symbols first differ, as a sentence — a whole 177-line matrix
/// printed twice says nothing a reader can act on.
fn differ(got: &str, want: &str) -> Option<String> {
    got.lines()
        .zip(want.lines())
        .enumerate()
        .find(|(_, (a, b))| a != b)
        .map(|(row, (a, b))| {
            let column = a.chars().zip(b.chars()).position(|(x, y)| x != y);
            format!("row {row}, first differing column {column:?}\n  got  {a}\n  want {b}")
        })
        .or_else(|| {
            (got.lines().count() != want.lines().count()).then(|| {
                format!(
                    "{} rows, oracle has {}",
                    got.lines().count(),
                    want.lines().count()
                )
            })
        })
}

/// **The reporter reports**, which is a test because nothing else ever runs it:
/// [`differ`] only speaks on a failure, so on a green suite it is dead code
/// that will be read for the first time on the day something breaks. A
/// comparison whose explanation has never executed is a comparison that can
/// fail into silence.
#[test]
fn the_difference_reporter_names_the_row_the_column_and_a_missing_row() {
    assert_eq!(
        differ("##\n..", "##\n.."),
        None,
        "identical is no difference"
    );
    let row = differ("##\n..", "##\n.#").unwrap_or_default();
    assert!(row.contains("row 1"), "{row}");
    assert!(row.contains("Some(1)"), "the differing column: {row}");
    let short = differ("##", "##\n..").unwrap_or_default();
    assert!(short.contains("1 rows, oracle has 2"), "{short}");
}

/// **The three pinned symbols, module for module.** Each was chosen for one
/// thing nothing smaller exercises — see [`oracle`] — and the mask is pinned
/// with them, so the penalty rules are under test too.
#[test]
fn the_pinned_symbols_match_the_oracle_module_for_module() {
    let cases: [(&str, Vec<u8>); 3] = [
        (oracle::V1, b"hello".to_vec()),
        (oracle::V7, payload(7, 122)),
        (oracle::V10, payload(10, 213)),
    ];
    for (want, bytes) in cases {
        let symbol = Symbol::encode(&bytes).expect("the oracle's payload fits");
        assert_eq!(
            differ(&as_oracle(&symbol), want),
            None,
            "a {}-module symbol disagrees with the oracle",
            symbol.side()
        );
    }
}

/// **Every version, filled to its exact byte capacity.** Forty full matrices
/// would be forty fixtures read for one number, so what is pinned is the shape
/// the oracle reports: the side, the mask its penalty rules chose, and the dark
/// count — which moves if a codeword moves, if a block splits differently, if
/// an alignment pattern lands a module off, or if the mask choice changes.
#[test]
fn every_version_has_the_side_mask_and_dark_count_the_oracle_reports() {
    for &(version, len, side, mask, dark) in &tables::FULL {
        let symbol = Symbol::encode(&payload(version, len)).expect("the stated capacity fits");
        let counted = (0..side)
            .flat_map(|y| (0..side).map(move |x| (x, y)))
            .filter(|&(x, y)| symbol.dark(x, y))
            .count();
        assert_eq!(
            (symbol.side(), symbol.mask(), counted),
            (side, mask, dark),
            "version {version} at {len} bytes"
        );
    }
    assert_eq!(
        tables::FULL.len(),
        40,
        "the version walk enumerated nothing"
    );
}

/// **One byte past the stated capacity picks the next version**, at every
/// boundary the table names — the choice being a function of length is what
/// makes a hand-picked version impossible to get wrong.
#[test]
fn one_byte_past_a_capacity_is_the_next_version_up() {
    for pair in tables::FULL.windows(2) {
        let [(version, len, ..), (next, ..)] = pair else {
            continue;
        };
        let over = Symbol::encode(&payload(*version, len + 1)).expect("the next version holds it");
        assert_eq!(over.side(), 17 + 4 * next, "one past version {version}");
    }
}

/// **Past the last version there is no symbol**, and the refusal is a value
/// naming what was offered rather than a panic.
#[test]
fn a_payload_past_the_ceiling_is_refused_by_name() {
    assert_eq!(
        Symbol::encode(&vec![0; LIMIT + 1]),
        Err(TooLong { len: LIMIT + 1 })
    );
    assert!(
        Symbol::encode(&vec![0; LIMIT]).is_ok(),
        "the ceiling itself"
    );
    assert!(
        TooLong { len: LIMIT + 1 }
            .to_string()
            .contains(&LIMIT.to_string())
    );
}

/// The empty payload is a symbol, not a case: a version 1 with nothing in it
/// but padding.
#[test]
fn nothing_to_encode_is_the_smallest_symbol_there_is() {
    let symbol = Symbol::encode(&[]).expect("an empty payload still fits");
    assert_eq!(symbol.side(), 21);
}

/// Outside the symbol is light, which is the answer the quiet zone gives — so
/// a renderer that walks past the edge draws paper rather than reading off the
/// end of a row into the next one.
#[test]
fn every_coordinate_outside_the_symbol_is_light() {
    let symbol = Symbol::encode(b"edge").expect("it fits");
    let side = symbol.side();
    assert!(!symbol.dark(side, 0), "past the right edge");
    assert!(!symbol.dark(0, side), "past the bottom edge");
    assert!(symbol.dark(0, 0), "and the corner finder is still dark");
}

mod block;
mod structure;
