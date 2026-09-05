//! The words a fleet row wears: the receipt that names its op, an attempt's
//! lines, and the churn read off the shape.

use super::{attempt, changed, churn, ending, receipt};
use crate::reply::diff::Churn;
use crate::test_support::window::diff;
use crate::ui::model::Armed;

/// **The op leads**, because one `armed` reply answers two families and a
/// sentence that dropped the op would be a sentence about neither.
#[test]
fn the_receipt_names_the_op_it_answers_and_never_a_family() {
    assert_eq!(
        receipt(&Armed {
            op: "fleet".to_owned(),
            armed: true
        }),
        "fleet: it is standing"
    );
    assert_eq!(
        receipt(&Armed {
            op: "disarm".to_owned(),
            armed: false
        }),
        "disarm: it is not standing"
    );
}

/// **An attempt earns the lines it has facts for and no others** — a bare one
/// says three things, a whole one says every column upstream wrote.
#[test]
fn an_attempt_says_what_it_has_and_stays_silent_about_the_rest() {
    let bare = attempt(&crate::test_support::window::attempt("bl-9", "pending"));
    assert_eq!(bare.len(), 3, "{bare:?}");
    assert!(bare[0].contains("bl-9"), "{bare:?}");
    assert!(bare[0].contains("pending"), "{bare:?}");
    assert!(bare[1].contains("4 steps"), "{bare:?}");
    let whole = crate::reply::science::Attempt {
        goal: Some("ship it".to_owned()),
        conversation: Some("c-1".to_owned()),
        base: Some("f00d".to_owned()),
        governing: Some("dead".to_owned()),
        response: Some("done".to_owned()),
        pins: vec!["AGENTS.md".to_owned()],
        verdicts: vec![crate::reply::science::Verdict {
            sender: "judge".to_owned(),
            body: "cleaner".to_owned(),
        }],
        compacted: Some(12),
        ..crate::test_support::window::attempt("bl-1", "accepted")
    };
    let said = attempt(&whole).join("\n");
    for line in [
        "ship it",
        "in c-1",
        "from f00d",
        "governed by dead",
        "said done",
        "pinned AGENTS.md",
        "judge — cleaner",
        "12 entries compacted",
    ] {
        assert!(said.contains(line), "{line:?} is on no line: {said}");
    }
}

/// **The ending is the token plus whatever it could say**, and the seat adds
/// no reading of its own to either.
#[test]
fn the_ending_says_the_token_and_the_two_facts_beside_it() {
    let mut row = crate::test_support::window::attempt("bl-1", "accepted");
    assert_eq!(ending(&row), "accepted");
    row.outcome.commit = Some("ccc".to_owned());
    row.outcome.by = Some("at-1".to_owned());
    assert_eq!(ending(&row), "accepted at ccc by at-1");
}

/// **A diff row says its refs where it has them** and says what is missing
/// where it does not — each state's own sentence, never one for all three.
#[test]
fn a_diff_row_says_the_state_s_own_facts() {
    assert_eq!(changed(&diff("bl-1", "unreadable")), "bl-1  [unreadable]");
    let gone = crate::reply::diff::Diff {
        source: Some("work/bl-2".to_owned()),
        target: Some("main".to_owned()),
        missing: vec!["work/bl-2".to_owned()],
        ..diff("bl-2", "absent")
    };
    assert_eq!(
        changed(&gone),
        "bl-2  [absent]  work/bl-2 → main  no such ref: work/bl-2"
    );
    let moved = crate::reply::diff::Diff {
        source: Some("attempt/at-1".to_owned()),
        target: Some("work/bl-3".to_owned()),
        handle: Some("at-1".to_owned()),
        delivered: Some("ccc".to_owned()),
        truncated: Some(true),
        ..diff("bl-3", "diff")
    };
    assert_eq!(
        changed(&moved),
        "bl-3  [diff]  attempt/at-1 → work/bl-3  candidate at-1  delivered ccc  \
         (the listing was cut)"
    );
}

/// **Binary is read off the SHAPE**, because upstream writes a count or it
/// writes `binary`, never both — a reading that asked which fields are there
/// cannot disagree with the encoder.
#[test]
fn a_churn_says_its_counts_or_says_binary() {
    assert_eq!(
        churn(&Churn {
            path: "src/a.rs".to_owned(),
            added: Some(3),
            removed: Some(1),
            binary: None
        }),
        "src/a.rs  +3 −1"
    );
    assert_eq!(
        churn(&Churn {
            path: "assets/x.png".to_owned(),
            added: None,
            removed: None,
            binary: Some(true)
        }),
        "assets/x.png  binary"
    );
}
