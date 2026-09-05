//! The wall's three acts as rows, and the pin pair that are assertions rather
//! than a toggle.

use super::{PIN, UNPIN, pin, unpin};
use crate::verbs::{find, table};
use serde_json::json;

/// **Both are rows in the one table**, so `lernie pin` and a click compose one
/// object — and each addresses a workspace, which is what makes it routable.
#[test]
fn the_pin_pair_are_rows_in_the_one_table() {
    assert_eq!(find("pin"), Some(PIN));
    assert_eq!(find("unpin"), Some(UNPIN));
    assert!(table().contains(&PIN) && table().contains(&UNPIN));
    assert!(PIN.addresses_a_workspace() && UNPIN.addresses_a_workspace());
    assert_eq!(PIN.usage(), "lernie pin <workspace>");
    assert_eq!(UNPIN.usage(), "lernie unpin <workspace>");
}

/// **Two ops and not one taking a bool**, which is the whole of upstream's
/// *"says what it means rather than flipping whatever it found"*: each door
/// builds the assertion it names, and two seats sending the same one agree.
#[test]
fn each_door_builds_the_assertion_it_names() {
    assert_eq!(
        pin("home".to_owned()),
        json!({"op": "pin", "workspace": "home"})
    );
    assert_eq!(
        unpin("home".to_owned()),
        json!({"op": "unpin", "workspace": "home"})
    );
    assert_eq!(
        PIN.envelope(vec!["home".to_owned()])
            .expect("the row's arity"),
        pin("home".to_owned()),
        "the door and the typed word are one gesture"
    );
}
