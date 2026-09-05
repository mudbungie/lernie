//! The clients pane between frames: the aim that gates it, and what it retires
//! with.

use crate::test_support::window::{machines, seated};
use crate::ui::Model;

/// **The aim is the gate.** The read the pane stands up carries a workspace,
/// and a workspace is what an aim is — so a seat aimed at nothing cannot open
/// it at all.
#[test]
fn the_pane_opens_only_where_the_window_is_aimed_at_a_wall() {
    let mut nowhere = Model::default();
    nowhere.begin_clients();
    assert!(!nowhere.showing(crate::ui::Listing::Clients));
    let mut model = seated();
    model.begin_clients();
    assert!(model.showing(crate::ui::Listing::Clients));
    assert!(model.covered(), "it covers the conversation");
    model.close_clients();
    assert!(!model.showing(crate::ui::Listing::Clients));
}

/// **Closing keeps the rows** — the next open on the same wall is about the
/// same machines, and the standing read replaces them anyway.
#[test]
fn closing_the_pane_keeps_what_the_wall_answered() {
    let mut model = machines();
    model.close_clients();
    assert!(!model.showing(crate::ui::Listing::Clients));
    assert!(model.machines.is_some(), "the rows are the engine's");
}

/// **Aiming somewhere else takes the pane AND its rows.** They are one
/// workspace's registrations, so a pane left standing over a new aim would
/// paint one wall's machines under another's name.
#[test]
fn aiming_at_another_wall_retires_the_pane_and_its_rows() {
    let mut model = machines();
    model.aim_at("(this box's own engine)", "elsewhere");
    assert!(!model.showing(crate::ui::Listing::Clients));
    assert_eq!(model.machines, None);
}

/// **Escape closes it**, on the ladder's own rung and composing nothing.
#[test]
fn escape_puts_the_pane_down() {
    let mut model = machines();
    model.escape();
    assert!(!model.showing(crate::ui::Listing::Clients));
    assert!(model.machines.is_some(), "the rows are the engine's");
    assert!(model.outbox.is_empty());
}
