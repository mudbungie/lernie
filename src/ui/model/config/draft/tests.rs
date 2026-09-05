//! The two readings a draft takes against the engine's answer, over the
//! lifecycle they exist for: opened, typed in, written, answered, and written
//! over by somebody else.

use super::Draft;

/// **A box opened on the file agrees with it**: nothing to write, and nothing
/// has moved.
#[test]
fn a_box_opened_on_the_file_has_nothing_to_say() {
    let draft = Draft::of("beat: 1\n");
    assert_eq!(draft.text, "beat: 1\n");
    assert!(!draft.unwritten("beat: 1\n"), "nothing to write");
    assert!(!draft.moved("beat: 1\n"), "and nothing has moved");
}

/// **Typing is unwritten and is not a file that moved.** The answer is still
/// what the box and the engine last agreed on, which is the whole of what the
/// anchor is for.
#[test]
fn typing_is_unwritten_and_is_not_a_file_that_moved() {
    let mut draft = Draft::of("beat: 1\n");
    draft.text = "beat: 2\n".to_owned();
    draft.settle("beat: 1\n");
    assert!(draft.unwritten("beat: 1\n"));
    assert!(
        !draft.moved("beat: 1\n"),
        "the operator moved it, not a writer"
    );
}

/// **A write this seat sent reads as neither, at both ends of its flight.**
/// In doubt, the answer is still the old bytes and they are the anchor; landed,
/// the answer IS the box — and the anchor catches up in the same frame, so a
/// later edit is not read as somebody else's write.
#[test]
fn a_write_this_seat_sent_never_reads_as_a_file_that_moved() {
    let mut draft = Draft::of("beat: 1\n");
    draft.text = "beat: 2\n".to_owned();
    draft.settle("beat: 1\n");
    assert!(
        !draft.moved("beat: 1\n"),
        "in flight, the answer is the anchor"
    );
    draft.settle("beat: 2\n");
    assert_eq!(draft.seed, "beat: 2\n", "landed, the anchor is the answer");
    assert!(
        !draft.unwritten("beat: 2\n"),
        "and there is nothing to write"
    );
    draft.text = "beat: 3\n".to_owned();
    assert!(
        !draft.moved("beat: 2\n"),
        "a later edit is the operator's own"
    );
}

/// **Another writer is the one reading left**: the answer is neither the box
/// nor what the box and the engine last agreed on.
#[test]
fn a_file_written_by_somebody_else_is_the_one_reading_left() {
    let mut draft = Draft::of("beat: 1\n");
    draft.text = "beat: 2\n".to_owned();
    draft.settle("beat: 9\n");
    assert!(draft.moved("beat: 9\n"), "neither the box nor the anchor");
    assert_eq!(draft.seed, "beat: 1\n", "and the anchor does not follow it");
}

/// And an untouched box over a file that moved says so too — its bytes are
/// stale, and writing them would put another writer's change back.
#[test]
fn an_untouched_box_over_a_moved_file_says_so() {
    let mut draft = Draft::of("beat: 1\n");
    draft.settle("beat: 9\n");
    assert!(draft.moved("beat: 9\n"));
}
