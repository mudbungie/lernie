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

/// **The box is seeded from the file the first time it answers**, and picking
/// another destination drops it with the bytes it was about.
#[test]
fn the_box_is_seeded_from_the_file_and_goes_with_the_destination() {
    let mut model = configured();
    assert_eq!(model.drafted(), None, "nothing is edited before an answer");
    model.draft_config("beat: 1\n");
    assert_eq!(
        model.drafted().map(|draft| draft.text),
        Some("beat: 1\n".to_owned())
    );
    model.read_config(&Where::Cadence);
    assert_eq!(model.drafted(), None, "a new file is a new box");
}

/// **Reverting puts the engine's answer back in the box**, which is the way
/// out of an edit and the way to take another writer's bytes — one act.
#[test]
fn reverting_puts_the_engine_s_answer_back_in_the_box() {
    let mut model = configured();
    model.draft_config("beat: 1\n");
    if let Some(text) = model.draft_box() {
        *text = "beat: 2\n".to_owned();
    }
    assert!(model.drafted().is_some_and(|d| d.unwritten("beat: 1\n")));
    model.revert_config("beat: 9\n");
    assert_eq!(
        model.drafted().map(|draft| draft.text),
        Some("beat: 9\n".to_owned())
    );
    assert!(!model.drafted().is_some_and(|d| d.moved("beat: 9\n")));
}

/// **A destination naming a workspace is ROUTED by it** — the envelope carries
/// the address and `crate::seat::route` resolves it, so naming a channel here
/// would bypass the rename an entry performs.
#[test]
fn a_write_to_a_wall_s_own_file_names_no_channel() {
    let mut model = configured();
    model.write_config(
        &Where::Brazen {
            workspace: "home".to_owned(),
        },
        "[models]\n".to_owned(),
    );
    let posted = model.outbox.first().expect("one act");
    assert!(posted.act, "a write changes the world");
    assert_eq!(posted.channel, None, "routed by its workspace");
    assert_eq!(posted.envelope["text"], "[models]\n");
}

/// **A destination naming the ENGINE is addressed down the aimed channel**,
/// which is what stops the poster fanning it onto every engine this box is a
/// client of (bl-4855).
#[test]
fn a_write_to_an_engine_s_own_file_names_the_aimed_channel() {
    let mut model = configured();
    model.write_config(&Where::Cadence, "beat: 1\n".to_owned());
    let posted = model.outbox.first().expect("one act");
    assert_eq!(
        posted.channel.as_ref().map(|held| held.name.clone()),
        model.aim.as_ref().map(|aim| aim.channel.clone())
    );
}

/// And a seat that no longer holds the aimed channel composes NOTHING rather
/// than falling back to a fan — there is no channel left to address, and the
/// fan is the one answer that would be wrong.
#[test]
fn a_write_with_no_channel_left_to_name_composes_nothing() {
    let mut model = Model {
        roster: Vec::new(),
        ..configured()
    };
    model.write_config(&Where::Cadence, "beat: 1\n".to_owned());
    assert!(model.outbox.is_empty());
}

/// **The workflow name is typed, trimmed, and empty names no destination** —
/// the box is the listing, because upstream addresses a workflow by a name no
/// read this seat has enumerates.
#[test]
fn the_workflow_name_is_trimmed_and_empty_names_nothing() {
    let mut bare = seated();
    assert_eq!(bare.workflow_named(), "", "no pane, no name");
    assert_eq!(bare.workflow_box(), None);
    let mut model = configured();
    if let Some(name) = model.workflow_box() {
        *name = "  nightly  ".to_owned();
    }
    assert_eq!(model.workflow_named(), "nightly");
}

/// A box with no pane open is a box that is not on the glass, so nothing seeds
/// it and nothing reverts it.
#[test]
fn the_box_needs_a_pane_to_live_on() {
    let mut model = seated();
    model.draft_config("beat: 1\n");
    assert_eq!(model.drafted(), None);
    model.revert_config("beat: 1\n");
    assert_eq!(model.drafted(), None);
    assert_eq!(model.draft_box(), None);
}
