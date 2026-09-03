//! What a control says about what it composed.

use super::Posted;
use serde_json::json;

/// **The two constructors differ in exactly one fact and keep the envelope
/// whole.** Nothing here reads the gesture, so a verb this build has never
/// heard of rides through unchanged — the same promise `crate::envelope` makes
/// one layer down.
#[test]
fn an_act_and_a_read_carry_the_same_envelope() {
    let envelope = json!({"op": "nudge", "workspace": "home", "agent": "a"});
    let act = Posted::act(envelope.clone());
    let read = Posted::read(envelope.clone());
    assert_eq!(act.envelope, envelope);
    assert_eq!(read.envelope, envelope);
    assert!(act.act);
    assert!(!read.act);
}

/// **The derivation this type exists instead of, refused against the vendored
/// vocabulary.** The tempting rule is *a gesture naming no workspace is a
/// read* — true of the three window-level reads this build composes, and false
/// of the protocol: the corpus carries ops with no workspace slot that plainly
/// change the world. A rule that holds by coincidence is one the next control
/// breaks in silence, so the fact is recorded at the control instead.
#[test]
fn naming_no_workspace_does_not_make_a_gesture_a_read() {
    let nameless: Vec<String> = crate::test_support::corpus::record()
        .shapes
        .into_iter()
        .filter_map(|(name, signature)| {
            let op = name.strip_prefix("request/")?;
            let addressed = signature
                .iter()
                .any(|slot| slot.starts_with("/workspace") || slot.starts_with("/prepared"));
            (!addressed).then(|| op.to_owned())
        })
        .collect();
    for changes_the_world in ["create", "close", "complete", "deliver", "update", "retire"] {
        assert!(
            nameless.contains(&changes_the_world.to_owned()),
            "{changes_the_world} is an act with no workspace slot, so the \
             predicate cannot stand in for this field; nameless: {nameless:?}"
        );
    }
}
