//! Matching-ladder tests: each rung wins only when every rung above
//! found nothing, uniqueness is enforced at the winning rung, and the
//! `*** End of File` preference for the file's end is honored.

use super::super::seek::{Error, LADDER, Rung, seek};

fn lines(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

#[test]
fn exact_unique_match_wins_on_the_first_rung() {
    let hay = lines(&["a", "b", "c"]);
    assert_eq!(
        seek(&hay, &lines(&["b", "c"]), 0, false),
        Ok((1, Rung::Exact))
    );
}

#[test]
fn trailing_whitespace_drift_matches_on_the_second_rung() {
    let hay = lines(&["fn main() {   ", "}"]);
    let got = seek(&hay, &lines(&["fn main() {"]), 0, false);
    assert_eq!(got, Ok((0, Rung::TrailingWs)));
}

#[test]
fn edge_whitespace_drift_matches_on_the_third_rung() {
    let hay = lines(&["    indented"]);
    assert_eq!(
        seek(&hay, &lines(&["indented"]), 0, false),
        Ok((0, Rung::EdgeWs))
    );
}

#[test]
fn unicode_punctuation_drift_matches_on_the_fourth_rung() {
    // Smart quotes, em-dash, and NBSP in the file; ASCII in the patch.
    let hay = lines(&["say \u{201C}hi\u{201D} \u{2014} it\u{2019}s\u{00A0}fine"]);
    let needle = lines(&["say \"hi\" - it's fine"]);
    assert_eq!(seek(&hay, &needle, 0, false), Ok((0, Rung::Normalized)));
}

#[test]
fn guillemets_and_wide_spaces_normalize_too() {
    let hay = lines(&["\u{00AB}q\u{00BB}\u{2003}\u{2212}5"]);
    assert_eq!(
        seek(&hay, &lines(&["\"q\" -5"]), 0, false),
        Ok((0, Rung::Normalized))
    );
}

#[test]
fn nothing_at_any_rung_is_not_found() {
    let hay = lines(&["a", "b"]);
    let err = seek(&hay, &lines(&["zzz"]), 0, false).unwrap_err();
    assert_eq!(err, Error::NotFound);
    assert!(err.to_string().contains("tried exact"), "{err}");
}

#[test]
fn a_needle_longer_than_the_haystack_is_not_found() {
    let hay = lines(&["a"]);
    assert_eq!(
        seek(&hay, &lines(&["a", "b"]), 0, false),
        Err(Error::NotFound)
    );
}

#[test]
fn a_start_past_the_last_fit_is_not_found() {
    let hay = lines(&["a", "b"]);
    assert_eq!(seek(&hay, &lines(&["a"]), 2, false), Err(Error::NotFound));
}

#[test]
fn two_matches_at_the_winning_rung_are_ambiguous_not_guessed() {
    let hay = lines(&["dup", "x", "dup"]);
    let err = seek(&hay, &lines(&["dup"]), 0, false).unwrap_err();
    assert_eq!(
        err,
        Error::Ambiguous {
            rung: Rung::Exact,
            count: 2
        }
    );
    assert_eq!(err.to_string(), "2 matches at the exact rung");
}

#[test]
fn ambiguity_on_a_lower_rung_is_still_a_decline_not_a_descent() {
    // No exact match anywhere; two matches once trailing ws is ignored.
    let hay = lines(&["x ", "y", "x  "]);
    let err = seek(&hay, &lines(&["x"]), 0, false).unwrap_err();
    assert_eq!(
        err,
        Error::Ambiguous {
            rung: Rung::TrailingWs,
            count: 2
        }
    );
}

#[test]
fn the_cursor_bound_disambiguates_repeats() {
    let hay = lines(&["dup", "x", "dup"]);
    assert_eq!(seek(&hay, &lines(&["dup"]), 1, false), Ok((2, Rung::Exact)));
}

#[test]
fn eof_prefers_the_end_even_when_the_block_also_appears_earlier() {
    let hay = lines(&["tail", "mid", "tail"]);
    assert_eq!(seek(&hay, &lines(&["tail"]), 0, true), Ok((2, Rung::Exact)));
}

#[test]
fn eof_end_check_walks_the_ladder_too() {
    let hay = lines(&["a", "tail   "]);
    assert_eq!(
        seek(&hay, &lines(&["tail"]), 0, true),
        Ok((1, Rung::TrailingWs))
    );
}

#[test]
fn eof_falls_back_to_an_ordinary_scan_when_the_end_does_not_match() {
    let hay = lines(&["block", "after"]);
    assert_eq!(
        seek(&hay, &lines(&["block"]), 0, true),
        Ok((0, Rung::Exact))
    );
}

#[test]
fn an_empty_needle_locates_the_insertion_point() {
    let hay = lines(&["a", "b"]);
    assert_eq!(seek(&hay, &[], 1, false), Ok((1, Rung::Exact)));
    assert_eq!(seek(&hay, &[], 0, true), Ok((2, Rung::Exact)));
}

#[test]
fn the_ladder_labels_are_the_documented_names_in_descent_order() {
    let labels: Vec<&str> = LADDER.iter().map(|r| r.label()).collect();
    assert_eq!(
        labels,
        [
            "exact",
            "ignore-trailing-whitespace",
            "ignore-edge-whitespace",
            "unicode-normalized"
        ]
    );
}
