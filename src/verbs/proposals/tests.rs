//! The learning loop's two gestures as data: the read at both its depths, and
//! the settle whose two required words have no default between them.

use serde_json::json;

use super::{PROPOSAL, PROPOSALS, VERDICTS, proposal, proposals};

/// **The bare listing states no id at all** — not `null`, which the engine
/// would have to read as a second spelling of the absence it already reads
/// from a missing key. Absence is the fact: nothing was named, so nothing is
/// answered whole.
#[test]
fn a_bare_listing_carries_no_id_field() {
    let gesture = proposals("ws".to_owned(), None);
    assert_eq!(gesture, json!({"op": "proposals", "workspace": "ws"}));
    assert_eq!(gesture.get("id"), None);
    assert_eq!(PROPOSALS.usage(), "lernie proposals <workspace>");
}

/// **Naming one rides as `id`** — the wire's own field name, and the same op
/// rather than a second word, because a seat that named one has already been
/// answered the row it names.
#[test]
fn naming_a_proposal_rides_the_same_op_one_depth_down() {
    assert_eq!(
        proposals("ws".to_owned(), Some("20260906T090000Z-r001".to_owned())),
        json!({"op": "proposals", "workspace": "ws", "id": "20260906T090000Z-r001"})
    );
}

/// **The settle takes both words and neither has a default**: a verdict that
/// defaulted would make throwing work away the easy half, and a settle that
/// took *the only one* would do something different the day a second proposal
/// was staged.
#[test]
fn the_settle_requires_the_proposal_and_the_verdict() {
    assert_eq!(
        proposal(
            "ws".to_owned(),
            "20260906T090000Z-r001".to_owned(),
            "accept".to_owned()
        ),
        json!({"op": "proposal", "workspace": "ws", "id": "20260906T090000Z-r001",
               "verdict": "accept"})
    );
    assert_eq!(
        PROPOSAL.usage(),
        "lernie proposal <workspace> <id> <verdict>"
    );
    let short = PROPOSAL.envelope(vec!["ws".to_owned(), "r001".to_owned()]);
    assert!(
        short.is_err_and(|said| said.contains("takes 3 argument(s) and got 2")),
        "a settle with no verdict must refuse by arity"
    );
}

/// **The two verdicts are the wire's own words**, in the order that puts the
/// destructive half second, and there is no third.
#[test]
fn the_verdicts_are_the_wire_s_two_words_and_there_is_no_third() {
    assert_eq!(VERDICTS, ["accept", "reject"]);
}
