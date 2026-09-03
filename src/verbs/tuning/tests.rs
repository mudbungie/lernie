//! The tuning family: two rows spelled by the one builder, and two doors whose
//! whole reason to exist is a field that is not a string.

use super::{EFFORT, MODEL, OFF, PRIORITY, ROLES, effort, levels, model, priority, roles, word};
use crate::verbs::{find, table};
use serde_json::json;

/// **The two rows are rows**, so they go through the table's one builder and
/// `lernie roles` and a click compose one object.
#[test]
fn the_read_and_the_assignment_are_rows_in_the_one_table() {
    assert_eq!(find("roles"), Some(ROLES));
    assert_eq!(find("model"), Some(MODEL));
    assert!(table().contains(&ROLES) && table().contains(&MODEL));
    assert!(ROLES.addresses_a_workspace() && MODEL.addresses_a_workspace());
    assert_eq!(ROLES.usage(), "lernie roles <workspace>");
    assert_eq!(
        MODEL.usage(),
        "lernie model <workspace> <role> <provider> <model>"
    );
}

/// The two doors onto them, which is what the window composes by name.
#[test]
fn the_row_doors_build_the_envelope_the_command_line_would_have() {
    assert_eq!(
        roles("home".to_owned()),
        json!({"op": "roles", "workspace": "home"})
    );
    assert_eq!(
        model(
            "home".to_owned(),
            "worker".to_owned(),
            "housevendor".to_owned(),
            "house-model-1".to_owned()
        ),
        json!({"op": "model", "workspace": "home", "role": "worker",
               "provider": "housevendor", "model": "house-model-1"})
    );
    assert_eq!(
        ROLES
            .envelope(vec!["home".to_owned()])
            .expect("the row's arity"),
        roles("home".to_owned()),
        "the door and the typed word are one gesture"
    );
}

/// **`off` is `null` on the wire and there is no word for it there**, which is
/// the whole reason `effort` is a door and not a row: a row would send the
/// string `"off"`, which is a fifth level the boundary refuses by name.
#[test]
fn a_level_is_a_string_or_the_absence_and_the_absence_is_null() {
    assert_eq!(
        effort(
            "home".to_owned(),
            "worker".to_owned(),
            Some("high".to_owned())
        ),
        json!({"op": "effort", "workspace": "home", "role": "worker", "level": "high"})
    );
    assert_eq!(
        effort("home".to_owned(), "worker".to_owned(), None),
        json!({"op": "effort", "workspace": "home", "role": "worker", "level": null}),
        "the key is written and its value is null — absent is a different frame"
    );
    assert!(find(EFFORT).is_none(), "it is a door and has no row");
}

/// **The lane is a bool**, for the same reason and with the same consequence.
#[test]
fn the_priority_lane_is_a_bool_on_the_wire() {
    for on in [true, false] {
        assert_eq!(
            priority("home".to_owned(), "compactor".to_owned(), on),
            json!({"op": "priority", "workspace": "home", "role": "compactor", "on": on})
        );
    }
    assert!(find(PRIORITY).is_none(), "it is a door and has no row");
}

/// **Four things a control offers, and the fourth is not a word.** The list is
/// `Option`s rather than strings precisely so a pane never has to translate one
/// back — the seat it paints hands the same value the envelope carries.
#[test]
fn the_levels_a_control_offers_are_the_three_words_and_the_absence() {
    let offered = levels();
    assert_eq!(
        offered,
        vec![
            Some("low".to_owned()),
            Some("medium".to_owned()),
            Some("high".to_owned()),
            None
        ]
    );
    let said: Vec<String> = offered.iter().map(|level| word(level.as_ref())).collect();
    assert_eq!(said, vec!["low", "medium", "high", OFF]);
}
