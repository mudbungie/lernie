//! The balls family's four rows: each builds the envelope the wire takes, and
//! the two widths are read off the parameters rather than listed twice.

use serde_json::json;

use super::{BALLS, BOARD, MARKS, WORKSPACE_BALLS, balls, board, marks, workspace_balls};

/// The two that name no workspace build a bare envelope, and the two that do
/// carry exactly the address they were handed.
#[test]
fn the_four_doors_build_the_envelopes_the_wire_takes() {
    assert_eq!(balls(), json!({ "op": "balls" }));
    assert_eq!(board(), json!({ "op": "board" }));
    assert_eq!(
        workspace_balls("ws".to_owned()),
        json!({ "op": "workspace-balls", "workspace": "ws" })
    );
    assert_eq!(
        marks("ws".to_owned()),
        json!({ "op": "marks", "workspace": "ws" })
    );
}

/// **The width is the parameters', not a second list.** Two of the four fan
/// because they have no way to name a channel, and two are aimed — the one
/// predicate `crate::offframe::asker` reads them apart by.
#[test]
fn two_of_the_four_address_a_workspace_and_two_address_every_channel() {
    assert!(!BALLS.addresses_a_workspace());
    assert!(!BOARD.addresses_a_workspace());
    assert!(WORKSPACE_BALLS.addresses_a_workspace());
    assert!(MARKS.addresses_a_workspace());
}

/// The usage line is computed off the row, so a parameter added cannot leave a
/// stale line behind.
#[test]
fn the_usage_lines_come_off_the_rows() {
    assert_eq!(BALLS.usage(), "lernie balls");
    assert_eq!(BOARD.usage(), "lernie board");
    assert_eq!(
        WORKSPACE_BALLS.usage(),
        "lernie workspace-balls <workspace>"
    );
    assert_eq!(MARKS.usage(), "lernie marks <workspace>");
}
