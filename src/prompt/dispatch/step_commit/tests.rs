//! The system slot's composition (ARCH §2.3 *Goal and soul are pinned
//! files*, §2.8): goal, identity, soul — one string, three tree facts.

use super::compose_system;

#[test]
fn an_unnamed_agent_states_no_identity() {
    // Byte-identical to the pre-name slot: absence is the general path
    // with empty inputs, so no blank line and no empty sentence is left
    // behind for the model to read something into (§2.8).
    assert_eq!(
        compose_system("hello", None, "system body"),
        "<goal>\nhello\n</goal>\n\nsystem body"
    );
}

#[test]
fn a_named_agent_is_told_its_name_between_the_goal_and_the_soul() {
    let slot = compose_system("hello", Some("pale-otter"), "system body");
    assert_eq!(
        slot,
        "<goal>\nhello\n</goal>\n\nYour name is pale-otter.\n\nsystem body"
    );
    // The goal still leads, which is the whole of §2.8's pinning claim.
    assert!(slot.starts_with("<goal>"), "{slot}");
    // One sentence, no instruction attached (§2.8).
    assert_eq!(slot.matches("pale-otter").count(), 1, "{slot}");
}
