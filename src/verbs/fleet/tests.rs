//! The fleet family's six rows and one door: the envelopes they build, and the
//! fact that every one of the seven addresses a workspace.

use serde_json::json;

use super::{
    ARM, DISARM, DISBAND, SCAN, SCIENCE, WORK_DIFF, arm, disarm, disband, fleet, scan, science,
    work_diff,
};

/// **The door carries a NUMBER**, which is the whole reason it is a door: the
/// verb table is rows of named strings and refuses to grow an arm for one.
#[test]
fn the_loop_s_door_builds_the_envelope_the_wire_requires() {
    assert_eq!(
        fleet("ws".to_owned(), "proj".to_owned(), 4),
        json!({ "cap": 4, "op": "fleet", "project": "proj", "workspace": "ws" })
    );
}

/// The six rows, each through the one builder.
#[test]
fn the_six_rows_build_the_envelopes_they_name() {
    assert_eq!(
        disband("ws".to_owned()),
        json!({ "op": "disband", "workspace": "ws" })
    );
    assert_eq!(
        arm("ws".to_owned(), "claude-haiku-4-5".to_owned()),
        json!({ "model": "claude-haiku-4-5", "op": "arm", "workspace": "ws" })
    );
    assert_eq!(
        disarm("ws".to_owned()),
        json!({ "op": "disarm", "workspace": "ws" })
    );
    assert_eq!(
        scan("ws".to_owned()),
        json!({ "op": "scan", "workspace": "ws" })
    );
    assert_eq!(
        science("ws".to_owned()),
        json!({ "op": "science", "workspace": "ws" })
    );
    assert_eq!(
        work_diff("ws".to_owned()),
        json!({ "op": "work-diff", "workspace": "ws" })
    );
}

/// **Every one of the seven addresses a workspace**, which is what puts all of
/// them on the asker's aimed half rather than on its fanned one — read off the
/// parameters, never listed a second time.
#[test]
fn all_seven_address_a_workspace() {
    for verb in [DISBAND, ARM, DISARM, SCAN, SCIENCE, WORK_DIFF] {
        assert!(verb.addresses_a_workspace(), "{}", verb.word);
    }
    assert_eq!(
        crate::envelope::workspace(&fleet("ws".to_owned(), "p".to_owned(), 1)).as_deref(),
        Some("ws"),
        "the door's envelope names its wall too"
    );
}

/// The usage lines come off the rows, so a parameter added cannot leave a
/// stale line behind.
#[test]
fn the_usage_lines_come_off_the_rows() {
    assert_eq!(ARM.usage(), "lernie arm <workspace> <model>");
    assert_eq!(SCAN.usage(), "lernie scan <workspace>");
}
