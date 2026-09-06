//! The vocabulary's own arms — both of every absence, and each band of an age.

use super::{age, brief, clause, indent, line, line_over, listing, quoted, tally, things, when};

/// A listing with rows, and the sentence one with none earns instead.
#[test]
fn a_listing_is_its_rows_or_the_sentence_an_empty_one_earns() {
    assert_eq!(
        listing("head", vec!["a".to_owned(), "b".to_owned()], "none"),
        "head\n  a\n  b"
    );
    assert_eq!(listing("head", Vec::new(), "none"), "head\n  none");
    assert_eq!(indent("one\ntwo"), "  one\n  two");
}

/// **A head over what it says, and a head alone when it says nothing.** An
/// empty body is the same as no body: a blank indented line is noise.
#[test]
fn a_head_carries_a_body_only_when_there_is_one() {
    assert_eq!(line_over("head", Some("said".to_owned())), "head\n  said");
    assert_eq!(line_over("head", Some(String::new())), "head");
    assert_eq!(line_over("head", None), "head");
}

/// The three absences: a clause with no value, a flag not raised, and a tally
/// of nothing. Each says nothing rather than saying `none`, `false` or `0`.
#[test]
fn an_absent_fact_says_nothing_at_all() {
    assert_eq!(clause("at", Some("noon")), Some("at noon".to_owned()));
    assert_eq!(clause("at", None), None);
    assert_eq!(when(true, "running"), Some("running".to_owned()));
    assert_eq!(when(false, "running"), None);
    assert_eq!(tally(2, "waiting"), Some("2 waiting".to_owned()));
    assert_eq!(tally(0, "waiting"), None);
    assert_eq!(things(1, "member"), Some("1 member".to_owned()));
    assert_eq!(things(4, "member"), Some("4 members".to_owned()));
    assert_eq!(things(0, "member"), None);
    assert_eq!(
        line(vec![
            Some("a".to_owned()),
            None,
            Some(String::new()),
            Some("b".to_owned())
        ]),
        "a  b"
    );
}

/// **Each band of an age, and one unit.** A roster is scanned rather than read,
/// and the second unit has never changed a decision.
#[test]
fn an_age_is_one_unit_and_the_band_it_falls_in() {
    assert_eq!(age(9), "9s");
    assert_eq!(age(59), "59s");
    assert_eq!(age(90), "1m");
    assert_eq!(age(3_599), "59m");
    assert_eq!(age(7_200), "2h");
    assert_eq!(age(86_399), "23h");
    assert_eq!(age(200_000), "2d");
}

/// **A brief is one line, and an elision is marked.** The whole text is one
/// `--json` away, so the mark is what says to ask for it.
#[test]
fn a_brief_is_one_line_and_says_when_it_cut() {
    assert_eq!(brief("  two\nlines  "), "two lines");
    let long = "x".repeat(super::BRIEF + 40);
    let cut = brief(&long);
    assert!(cut.ends_with('…'), "{cut}");
    assert_eq!(cut.chars().count(), super::BRIEF + 1);
    assert_eq!(quoted("said"), Some("\"said\"".to_owned()));
    assert_eq!(quoted("   "), None);
}
