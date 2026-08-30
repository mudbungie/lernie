//! Help, answered here: the roster's alignment, one verb's page, and the word
//! that is not a verb.

use super::{page, roster};
use crate::verbs::table;

/// Every verb has a line, and the column is computed from the widest of them —
/// so a verb added tomorrow cannot leave the alignment behind.
#[test]
fn the_roster_holds_every_verb_and_aligns_on_the_widest() {
    let text = roster();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), table().len());
    for verb in table() {
        assert!(text.contains(&verb.usage()), "{} is missing", verb.word);
        assert!(text.contains(verb.summary), "{} has no summary", verb.word);
    }
    let widest = table()
        .iter()
        .map(|verb| verb.usage().len())
        .max()
        .expect("a verb");
    for (line, verb) in lines.iter().zip(table()) {
        let summary_at = line.find(verb.summary).expect("the summary");
        assert_eq!(summary_at, 2 + widest + 3, "{line:?}");
    }
}

/// One page: what to type, what it is for, and the detail folded to a width a
/// terminal holds.
#[test]
fn a_page_states_the_usage_the_summary_and_the_detail() {
    let text = page("follow").expect("a page");
    assert!(
        text.starts_with("usage: lernie follow <workspace> <agent>"),
        "{text}"
    );
    assert!(text.contains("hold the line"), "{text}");
    assert!(text.contains("never finishes"), "{text}");
    for line in text.lines() {
        assert!(line.len() <= 72, "{line:?} is {} wide", line.len());
    }
}

/// Every verb has a page, and none of them is empty — the table's `detail` is
/// not an optional field that a row may quietly skip.
#[test]
fn every_verb_answers_a_page() {
    for verb in table() {
        let text = page(verb.word).expect("a page");
        assert!(text.contains(verb.summary), "{}", verb.word);
        assert!(
            text.len() > verb.usage().len() + verb.summary.len(),
            "{}",
            verb.word
        );
    }
}

/// A word that is not a verb refuses **naming it**, and points at the list —
/// the operator who typed it is the one who needs it.
#[test]
fn a_word_that_is_not_a_verb_refuses_and_points_at_the_roster() {
    let refusal = page("wokspaces").expect_err("not a verb");
    assert!(refusal.contains("\"wokspaces\""), "{refusal}");
    assert!(refusal.contains("lernie help"), "{refusal}");
}
