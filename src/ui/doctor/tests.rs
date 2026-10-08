//! What the doctor pane says in every state it can be in, and the control that
//! opens it driven through the real window.

use super::{CLOSE, HEADING, NO_CHECKS, NOT_ANSWERED, OPEN, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{checked, click, doctored, own, pane, seated};
use crate::ui::{Diagnosis, Lookup, Model};

/// A closed pane paints nothing and says so.
#[test]
fn a_shut_pane_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood, "a shut pane reports that it painted nothing");
    assert!(!painted.contains(HEADING), "{painted}");
}

/// **One row per check, in each ok state**: a check that holds says so and
/// carries no remedy; one that does not says so and carries the engine's
/// remedy verbatim under it.
#[test]
fn every_check_paints_its_state_its_fact_and_the_remedy_where_there_is_one() {
    let mut model = doctored();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "(this box's own engine)",
        "ok  listener: what listener was found to be",
        "NOT OK  address: what address was found to be",
        "the engine's own remedy for address",
        CLOSE,
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
    assert!(
        !painted.contains("remedy for listener"),
        "a holding check states no remedy:\n{painted}"
    );
}

/// **The two empty states are different sentences.**
#[test]
fn every_empty_state_is_its_own_sentence() {
    let unanswered = Model {
        lookup: Some(Lookup::Doctor),
        ..seated()
    };
    let quiet = Model {
        diagnoses: vec![Diagnosis {
            channel: own().channel,
            rows: Vec::new(),
        }],
        ..unanswered.clone()
    };
    for (mut model, expected, absent) in [
        (unanswered, NOT_ANSWERED, NO_CHECKS),
        (quiet, NO_CHECKS, NOT_ANSWERED),
    ] {
        let painted = pane(|ui| {
            render(ui, &mut model);
        });
        assert!(painted.contains(expected), "{expected:?}:\n{painted}");
        assert!(!painted.contains(absent), "{absent:?}:\n{painted}");
    }
}

/// **The pane opens from the roster's strip and closes from its own control**,
/// and opening it composes the ask — which is why the opening control is the
/// one that carries `doctor`'s token.
#[test]
fn the_strip_opens_it_asking_and_its_own_word_shuts_it() {
    let window = Window::new();
    let mut model = seated();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.doctoring());
    assert_eq!(model.outbox.len(), 1);
    assert_eq!(
        crate::envelope::op(&model.outbox[0].envelope),
        crate::verbs::DOCTOR
    );
    model.diagnoses = vec![Diagnosis {
        channel: own().channel,
        rows: vec![checked("listener", true)],
    }];
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.doctoring());
}
