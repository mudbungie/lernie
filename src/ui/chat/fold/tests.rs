//! The fold as a value: what stands, what is hidden, and the row that has
//! nothing to hide.

use super::{Fold, of};
use crate::test_support::window::numbered;

#[test]
fn a_short_answer_is_not_folded_at_all() {
    assert_eq!(of(""), None);
    assert_eq!(of("Exit code: 0\n/home/u/ws\n"), None);
}

#[test]
fn a_long_answer_stands_as_a_few_lines_and_says_what_it_is_hiding() {
    let content = numbered(700);
    let Some(Fold { head, whole }) = of(&content) else {
        panic!("seven hundred lines is folded");
    };
    assert!(head.starts_with("line 001\n"), "{head:?}");
    assert!(!head.contains("line 007"), "{head:?}");
    assert_eq!(head.lines().count(), 6, "{head:?}");
    assert_eq!(
        whole,
        format!("show all 700 lines, {} bytes", content.len())
    );
}

/// **One long line is the other overrun**, and a line bound alone lets it
/// through: a pane that wraps paints thirty kilobytes on one line as hundreds
/// of rows of glass.
#[test]
fn one_very_long_line_is_folded_by_its_characters() {
    let content = "x".repeat(30_000);
    let Some(Fold { head, whole }) = of(&content) else {
        panic!("one thirty-kilobyte line is folded");
    };
    assert_eq!(head.chars().count(), 500);
    assert_eq!(whole, "show all 1 line, 30000 bytes");
}

/// **The cut is on a character, never on a byte.** A head that sliced a
/// multi-byte glyph in half would panic where the pane paints it, which is the
/// one place a transcript must not fail.
#[test]
fn the_head_is_cut_on_a_character_boundary() {
    let content = "é".repeat(600);
    let head = of(&content).expect("six hundred glyphs is folded").head;
    assert_eq!(head.chars().count(), 500);
    assert!(content.starts_with(&head));
}
