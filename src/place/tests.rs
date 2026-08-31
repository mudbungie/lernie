//! The place: the round trip, every way a file can fail to be one, and the
//! growth rule.

use super::{at, read, write};
use crate::test_support::Scratch;
use crate::ui::Aim;

/// The wall a window was aimed at.
fn aimed() -> Aim {
    Aim {
        channel: "(this box's own engine)".to_owned(),
        address: "home".to_owned(),
    }
}

/// **The round trip, both ways round.** Aimed at nothing is a place too — an
/// operator who left the roster comes back to it — so a run that wrote no aim
/// reads back as no aim rather than as the last one.
#[test]
fn where_the_window_was_pointed_comes_back_and_so_does_nowhere() {
    let scratch = Scratch::new();
    write(scratch.path(), Some(aimed())).expect("written");
    assert_eq!(read(scratch.path()), Some(aimed()));
    write(scratch.path(), None).expect("written");
    assert_eq!(read(scratch.path()), None);
}

/// **The root is made, not required.** A first run has no state directory at
/// all, and a window that could not keep its place because nobody had created a
/// directory for it would be a window that never kept one.
#[test]
fn a_state_root_that_is_not_there_yet_is_made() {
    let scratch = Scratch::new();
    let root = scratch.join("never/made/before");
    write(&root, Some(aimed())).expect("written");
    assert_eq!(read(&root), Some(aimed()));
}

/// **Every way this can fail is one answer, and it is the answer a first run
/// gets.** A forgotten selection is a keypress; a startup error is an outage,
/// and per-seat UI state may never become the second.
#[test]
fn nothing_a_file_can_be_wrong_about_refuses() {
    let scratch = Scratch::new();
    assert_eq!(read(scratch.path()), None, "no file at all");
    for body in [
        "",
        "not json",
        "[]",
        "{}",
        r#"{"aim": null}"#,
        r#"{"aim": {}}"#,
        r#"{"aim": {"channel": "own"}}"#,
        r#"{"aim": {"address": "home"}}"#,
        r#"{"aim": {"channel": 7, "address": "home"}}"#,
    ] {
        std::fs::write(at(scratch.path()), body).expect("write");
        assert_eq!(read(scratch.path()), None, "{body}");
    }
}

/// **A key this build does not know is ignored and one it wants is absence** —
/// the reply vocabulary's own rungs 3 and 4, applied to this box's own file. It
/// is what lets the next fact REMOTE §7 names be a key beside this one rather
/// than a format, and what lets an older build read a newer build's file
/// without losing the half it does understand.
#[test]
fn an_unknown_key_is_ignored_so_the_next_fact_is_a_key_and_not_a_format() {
    let scratch = Scratch::new();
    std::fs::write(
        at(scratch.path()),
        r#"{"aim": {"channel": "(this box's own engine)", "address": "home",
                    "scrolled_to": 42}, "draft": "not read yet"}"#,
    )
    .expect("write");
    assert_eq!(read(scratch.path()), Some(aimed()));
}

/// **A write answers its refusal rather than swallowing it**: by the time it
/// runs there is no window left to paint one in, and the only alternative is
/// losing the operator's place in silence.
#[test]
fn a_root_that_cannot_be_made_says_so_and_names_itself() {
    let scratch = Scratch::new();
    let blocked = scratch.join("a-file");
    std::fs::write(&blocked, b"not a directory").expect("write");
    let refusal = write(&blocked.join("under"), None).expect_err("refused");
    assert!(refusal.contains("under"), "{refusal}");
}
