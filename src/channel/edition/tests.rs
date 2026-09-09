//! **The gate that makes the vendored constants true**, both directions.
//!
//! [`super`] declares three facts a released binary carries; the corpus states
//! them and is not packaged. So every one of the three is recomputed here from
//! `corpus/shapes.json` and asserted equal — a stamp with no row is as red as a
//! row with no stamp, because a ledger that only fails in one direction is a
//! ledger that rots quietly in the other.

use std::collections::{BTreeMap, BTreeSet};

use super::{DEPRECATED, EDITION, FLOOR, STAMPS, spells, spells_at, stamp};
use crate::test_support::corpus::record;

/// One ledger row, as this file compares them.
type Row = (String, String, u32);

/// **The corpus's own stamps, collapsed to one per field path.** Upstream keys
/// a signature by `<path>:<type>` and spells one path under two types where a
/// value is nullable, so the edition a PATH appeared at is the earliest of its
/// spellings — the later one is the same fact gaining a second shape.
fn stamped() -> BTreeMap<(String, String), u32> {
    let mut found: BTreeMap<(String, String), u32> = BTreeMap::new();
    for (shape, signature) in record().shapes {
        for (spelled, edition) in signature {
            let (path, _) = spelled
                .rsplit_once(':')
                .unwrap_or_else(|| panic!("{shape}: {spelled} names no type"));
            let edition = u32::try_from(edition).expect("an edition");
            let at = (shape.clone(), path.to_owned());
            let held = found.entry(at).or_insert(edition);
            *held = (*held).min(edition);
        }
    }
    found
}

/// The rows above the floor — the whole of what this major has grown.
fn above_the_floor() -> BTreeSet<Row> {
    stamped()
        .into_iter()
        .filter(|(_, edition)| *edition > FLOOR)
        .map(|((shape, path), edition)| (shape, path, edition))
        .collect()
}

/// **The floor and this build's edition are the corpus's own.** The floor is
/// stated there; the edition is the newest stamp in it, which is exactly what
/// the preface claims this build can spell.
#[test]
fn the_floor_and_the_edition_are_read_back_out_of_the_corpus() {
    let corpus = record();
    assert_eq!(
        u64::from(FLOOR),
        corpus.floor,
        "the vendored FLOOR is not the corpus's — refresh one or correct the other"
    );
    let newest = stamped().into_values().max().expect("a stamped corpus");
    assert_eq!(
        EDITION, newest,
        "the vendored EDITION is not the newest stamp in the corpus — a seat \
         that overstates what it can spell is a seat that greys nothing"
    );
}

/// **The ledger holds exactly the post-floor paths**, both directions. A path
/// the corpus stamps above the floor and this list does not carry would be
/// painted as though every engine could spell it; a row the corpus does not
/// stamp would grey a control over a field that is required.
#[test]
fn the_ledger_holds_exactly_what_the_corpus_stamps_above_the_floor() {
    let carried: BTreeSet<Row> = STAMPS
        .iter()
        .map(|(shape, path, edition)| ((*shape).to_owned(), (*path).to_owned(), *edition))
        .collect();
    assert_eq!(
        carried.len(),
        STAMPS.len(),
        "a path is stamped twice in the ledger"
    );
    assert_eq!(
        carried,
        above_the_floor(),
        "the vendored edition ledger and the corpus disagree about what this \
         major has grown — add the row in the commit that re-vendors the corpus"
    );
}

/// **The deprecation list is upstream's, whole.** A name arriving in it is a
/// removal announced with three yog releases still to run, and the only thing
/// that can make that visible to a person is a diff — so the re-vendor which
/// brings one in reddens here until the row is written down.
#[test]
fn the_deprecation_list_is_the_corpus_own() {
    let carried: Vec<String> = DEPRECATED.iter().map(|&named| named.to_owned()).collect();
    assert_eq!(
        carried,
        record().deprecated,
        "upstream has announced a removal this build has not written down — \
         REMOTE §3.2's window rule gives three releases to answer it"
    );
}

/// **A path at or below the floor is required, so every engine of this major
/// spells it.** That is the whole reason the ledger only carries what is above:
/// there is nothing to discover below it.
#[test]
fn a_path_the_ledger_does_not_carry_reads_as_the_floor() {
    assert_eq!(stamp(STAMPS, "reply/workspaces", "/rows/[]/name"), FLOOR);
    assert!(spells(FLOOR, "reply/workspaces", "/rows/[]/name"));
}

/// **A post-floor path is spelled by a newer engine and not by an older one**,
/// which is the one question this module exists to answer. Asked against a
/// ledger the test states, because at the cut the vendored one is empty and an
/// arm nothing has run is an arm nobody has evidence about.
#[test]
fn a_post_floor_path_is_greyed_by_an_engine_that_cannot_spell_it() {
    let grown: &[(&str, &str, u32)] = &[("reply/follow", "/rows/[]/parked", FLOOR + 2)];
    assert_eq!(stamp(grown, "reply/follow", "/rows/[]/parked"), FLOOR + 2);
    assert!(!spells_at(grown, FLOOR, "reply/follow", "/rows/[]/parked"));
    assert!(!spells_at(
        grown,
        FLOOR + 1,
        "reply/follow",
        "/rows/[]/parked"
    ));
    assert!(spells_at(
        grown,
        FLOOR + 2,
        "reply/follow",
        "/rows/[]/parked"
    ));
    // A path the same engine has always had is spelled at any edition, and a
    // shape that is not this one is a different question with the same key.
    assert!(spells_at(grown, FLOOR, "reply/follow", "/rows/[]/says"));
    assert!(spells_at(grown, FLOOR, "reply/steps", "/rows/[]/parked"));
}
