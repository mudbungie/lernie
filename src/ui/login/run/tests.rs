//! What the followed run paints: both streams, the settled exit, the
//! run-by-hand command, and the empty fold that is a reading rather than a
//! silence.

use super::{BY_HAND, NO_RUN, settled};
use crate::reply::login::{Line, Signin};
use crate::test_support::window::{pane, signing};
use crate::ui::Model;

/// **Both streams reach the glass**, because bz writes the authorize URL and
/// every remedy to stderr — a pane that kept only stdout paints a blank.
#[test]
fn the_run_paints_both_streams_and_its_settled_exit_and_fallback() {
    let mut model = signing();
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    for word in [
        "open https://provider.invalid/auth",
        "waiting for the browser",
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
    let mut model = Model {
        signin: Some(Signin {
            lines: vec![Line {
                text: "no device endpoint".to_owned(),
                err: true,
            }],
            outcome: Some(78),
            fallback: Some("bz --login --provider housevendor".to_owned()),
        }),
        ..signing()
    };
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    for word in [
        &settled(78),
        BY_HAND,
        "bz --login --provider housevendor",
        "no device endpoint",
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
    assert_eq!(settled(0), "signed in");
}

/// **A pair with no run is a reading, not a silence** — the lane opens on one
/// empty frame and the pane says what that frame says.
#[test]
fn a_row_nobody_has_signed_in_to_says_so() {
    let mut model = Model {
        signin: Some(Signin::default()),
        ..signing()
    };
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    assert!(painted.lines().any(|line| line == NO_RUN), "{painted}");
    let mut unheard = Model {
        signin: None,
        ..signing()
    };
    let painted = pane(|ui| {
        super::super::render(ui, &mut unheard);
    });
    assert!(
        !painted.contains(NO_RUN),
        "nothing has been heard at all:\n{painted}"
    );
}
