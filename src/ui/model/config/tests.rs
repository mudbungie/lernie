//! The config pane between frames: the aim that gates it, the one question it
//! holds, and what it retires with.

use crate::test_support::window::{configured, seated};
use crate::ui::{Configuring, Model};
use crate::verbs::Where;

/// **The aim is the gate.** The lineage read carries a workspace, and a
/// workspace is what an aim is — so a seat aimed at nothing cannot open it.
#[test]
fn the_pane_opens_only_where_the_window_is_aimed_at_a_wall() {
    let mut nowhere = Model::default();
    nowhere.begin_configuring();
    assert_eq!(nowhere.configuring, None);
    let mut model = seated();
    model.begin_configuring();
    assert_eq!(model.configuring, Some(Configuring::default()));
    assert!(model.covered(), "it covers the conversation");
    assert_eq!(model.configured(), None, "and points at nothing yet");
    model.close_configuring();
    assert_eq!(model.configuring, None);
}

/// **Picking a file points the pane at it and drops the last answer**: the
/// reply carries no destination, so a file left standing under a new question
/// would be unattributable.
#[test]
fn picking_a_file_points_the_pane_at_it_and_drops_the_last_bytes() {
    let mut model = configured();
    assert!(model.config.is_some());
    model.read_config(&Where::Cadence);
    assert_eq!(model.configured(), Some(Where::Cadence));
    assert_eq!(model.config, None, "the last file's bytes go with it");
    assert!(
        model.outbox.is_empty(),
        "the read stands rather than posting"
    );
}

/// **A pick with no pane open does nothing**, which is the state
/// [`Model::begin_configuring`] refuses to create made unreachable rather than
/// merely unlikely.
#[test]
fn a_pick_with_no_pane_open_points_at_nothing() {
    let mut model = seated();
    model.read_config(&Where::Cadence);
    assert_eq!(model.configured(), None);
}

/// **Escape closes it**, on the ladder's own rung, and aiming elsewhere takes
/// the pane and both answers with it — a destination carries the aim's own
/// workspace inside it.
#[test]
fn escape_puts_it_down_and_a_new_aim_retires_it() {
    let mut model = configured();
    model.escape();
    assert_eq!(model.configuring, None);
    assert!(model.config.is_some(), "the answers are the engine's");
    let mut moved = configured();
    moved.aim_at("(this box's own engine)", "elsewhere");
    assert_eq!(moved.configuring, None);
    assert_eq!(moved.config, None);
    assert_eq!(moved.lineages, None);
}
