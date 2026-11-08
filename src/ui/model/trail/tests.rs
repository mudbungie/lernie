//! The trail pane between frames: the door, and the union it files into.

use crate::reply::{Read, Reply};
use crate::test_support::window::{own, seated, trailed};
use crate::ui::{Channel, Model};

/// The pane takes no subject, so it opens from a seat that has aimed at
/// nothing — which is the seat most likely to be asking what this box did.
#[test]
fn the_pane_opens_with_nothing_aimed_at_and_closes_again() {
    let mut model = Model::default();
    assert!(!model.trailing());
    model.begin_trail();
    assert!(model.trailing());
    assert!(model.covered(), "the pane covers the conversation");
    // **One door closes all three of the window's own panes**, and Escape is
    // that door — a second spelling would be a second place a later pane has
    // to be added to.
    model.escape();
    assert!(!model.trailing());
}

/// **Floating, a word fired over it stands it down** (`Model::uncover`);
/// **docked, it stands beside the pane the word opened** (DESIGN §4.48 item
/// 8) — the one field, read through the one query.
#[test]
fn another_pane_stands_the_trail_down_floating_and_beside_it_docked() {
    let mut model = Model::default();
    model.begin_trail();
    assert!(crate::ui::slash::fire(
        &mut model,
        crate::ui::commands::WORD,
        ""
    ));
    assert!(!model.trailing());
    assert!(model.commanding());
    let mut docked = Model {
        room: true,
        ..Model::default()
    };
    docked.settings.mode = crate::settings::Mode::Advanced;
    docked.begin_trail();
    assert!(!docked.covered(), "a docked pane is not a cover");
    assert!(crate::ui::slash::fire(
        &mut docked,
        crate::ui::commands::WORD,
        ""
    ));
    assert!(docked.trailing() && docked.commanding());
    assert!(docked.covered());
    docked.escape();
    assert!(
        docked.trailing() && !docked.commanding(),
        "Esc closed the overlay first"
    );
    docked.escape();
    assert!(!docked.trailing(), "then the rail");
}

/// **Nothing on the glass can invalidate it**, because nothing on the glass is
/// its subject — the decision queue's own rule, read on a second pane.
#[test]
fn aiming_and_selecting_leave_it_standing() {
    let mut model = seated();
    model.begin_trail();
    model.select("c-2");
    assert!(
        model.trailing(),
        "a selection retired a pane it is not about"
    );
    let aim = model.aim.clone().expect("the seated fixture is aimed");
    model.aim_at(&aim.channel, &aim.address);
    assert!(model.trailing(), "an aim retired a pane it is not about");
}

/// **One channel's answer replaces its own section and no other.** A box
/// serving three servers does not lose the two that answered.
#[test]
fn an_answer_replaces_its_own_channel_and_leaves_the_others_standing() {
    let mut model = seated();
    let other = Channel {
        name: "elsewhere".to_owned(),
        ..own().channel
    };
    model.absorb(
        &own().channel,
        Read::Answer(Reply::Ops(vec![trailed("bl close x", "live")])),
    );
    model.absorb(
        &other,
        Read::Answer(Reply::Ops(vec![trailed("bz login", "acked")])),
    );
    assert_eq!(model.trails.len(), 2);
    model.absorb(&own().channel, Read::Answer(Reply::Ops(Vec::new())));
    assert_eq!(
        model.trails.len(),
        2,
        "a section was added rather than replaced"
    );
    assert!(
        model.trails[0].rows.is_empty(),
        "the answer did not replace"
    );
    assert_eq!(
        model.trails[1].rows.len(),
        1,
        "the other channel was disturbed"
    );
}

/// The rows outlive the close, exactly as the queue's do: the next open is
/// about the same trail and the standing read replaces them anyway.
#[test]
fn closing_the_pane_keeps_what_the_channels_said() {
    let mut model = seated();
    model.absorb(
        &own().channel,
        Read::Answer(Reply::Ops(vec![trailed("bl close x", "live")])),
    );
    model.begin_trail();
    model.close_lookup();
    assert_eq!(model.trails.len(), 1);
}
