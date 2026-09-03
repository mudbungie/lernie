//! The two arms, and the two questions a caller may ask of either.

use super::Reach;

/// **The sentence survives the classification.** Every caller that does not act
/// on the difference — the asker, the follow lane, argv — reads one string, so
/// the arm may never eat a word of the transport's own wording.
#[test]
fn both_arms_say_what_the_transport_said() {
    assert_eq!(
        Reach::Unsent("connect: refused".to_owned()).said(),
        "connect: refused"
    );
    assert_eq!(
        Reach::Unanswered("receive: eof".to_owned()).said(),
        "receive: eof"
    );
}

/// **Only the unanswered arm crossed**, which is the whole of what the poster
/// reads: an act that crossed is in doubt, and one that did not is safe to make
/// again.
#[test]
fn only_the_unanswered_arm_reached_the_far_end() {
    assert!(!Reach::Unsent("x".to_owned()).crossed());
    assert!(Reach::Unanswered("x".to_owned()).crossed());
}
