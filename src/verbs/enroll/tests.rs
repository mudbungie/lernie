//! **The one optional field on this surface**, asserted from both sides: the
//! gesture a stated route becomes, and the gesture an unstated one does not.

use serde_json::json;

use super::{ADDRESS, ENROLL, enroll};

/// **Unstated is UNSAID.** A route nobody gave writes no field — not `null`,
/// which the engine would then have to read as a second spelling of absence
/// (yog's codec says so from the other side: *"a null would be a second
/// spelling of the same absence"*). The bare gesture is byte-for-byte the one
/// an operator would have written by hand before the word existed.
#[test]
fn an_enrollment_that_states_no_route_carries_no_address_field() {
    let gesture = enroll(
        "ops".to_owned(),
        "box-1".to_owned(),
        "foot".to_owned(),
        None,
    );
    assert_eq!(
        gesture,
        json!({"op": "enroll", "workspace": "ops", "name": "box-1", "grade": "foot"})
    );
    assert_eq!(gesture.get(ADDRESS), None);
}

/// **A stated route rides as `address`** — REMOTE §8.4's own field name, which
/// is what the engine reads (`opt_str_of(o, "address")`), not the `--at` the
/// operator typed.
#[test]
fn a_stated_route_rides_the_gesture_under_the_wire_s_own_name() {
    let gesture = enroll(
        "ops".to_owned(),
        "box-1".to_owned(),
        "foot".to_owned(),
        Some("host.containers.internal:7773".to_owned()),
    );
    assert_eq!(
        gesture,
        json!({
            "op": "enroll",
            "workspace": "ops",
            "name": "box-1",
            "grade": "foot",
            "address": "host.containers.internal:7773",
        })
    );
}

/// The row is unchanged by the field: `address` is not a parameter, so the
/// three words are still the whole arity and the usage line still says so.
#[test]
fn the_optional_field_is_not_a_fourth_parameter() {
    assert_eq!(ENROLL.params, &["workspace", "name", "grade"]);
    assert_eq!(ENROLL.usage(), "lernie enroll <workspace> <name> <grade>");
}
