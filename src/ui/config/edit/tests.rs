//! What the editor says and what it composes: the box seeded from the file,
//! the enablement that is its arming, the sentence for a file that moved, and
//! the two controls.

use super::{MOVED, NOTHING_TO_WRITE, REVERT, WRITE};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, configured, pane};
use crate::ui::Model;
use crate::verbs::Where;

/// **The box is seeded from the file's own bytes**, which is what makes the
/// pane an editor rather than a viewer with a blank beside it.
#[test]
fn the_box_opens_holding_the_bytes_the_engine_answered() {
    let mut model = configured();
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    assert!(painted.contains("roles:"), "{painted}");
    assert_eq!(
        model.drafted().map(|draft| draft.text),
        model.config.as_ref().map(|held| held.text.clone())
    );
}

/// **The arming is the enablement, and the refusal is spelled beside it**: a
/// box holding what the file holds has nothing to write, and a greyed control
/// says a thing is not live and nothing about what would make it live.
#[test]
fn a_box_that_holds_the_file_says_why_the_controls_are_dark() {
    let mut model = configured();
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    assert!(painted.contains(NOTHING_TO_WRITE), "{painted}");
    assert!(
        painted.contains(WRITE) && painted.contains(REVERT),
        "{painted}"
    );
    assert!(!painted.contains(MOVED), "nothing has moved: {painted}");
}

/// **A file that moved on the engine says so**, which is upstream's hash guard
/// restated as a reading: the guard does not cross the wire, and the operator
/// typing for minutes is exactly the long-lived draft it protects.
#[test]
fn a_file_that_moved_under_the_box_says_so() {
    let mut model = configured();
    model.draft_config("roles:\n  worker:\n    provider: gone\n");
    if let Some(text) = model.draft_box() {
        *text = "mine\n".to_owned();
    }
    if let Some(held) = model.config.as_mut() {
        held.text = "somebody else's\n".to_owned();
    }
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    assert!(painted.contains(MOVED), "{painted}");
}

/// **Writing composes the act carrying the whole box**, and leaves the box
/// alone: firing is not spending, because the enablement is a fact about the
/// world rather than about this seat's outbox.
#[test]
fn the_write_control_composes_the_act_and_keeps_the_box() {
    let window = Window::new();
    let mut model = configured();
    click(&window, "cadence", |ctx| crate::ui::render(ctx, &mut model));
    model.config = Some(crate::reply::config::Config {
        text: "beat: 1\n".to_owned(),
        settings: Vec::new(),
    });
    model.draft_config("beat: 1\n");
    if let Some(text) = model.draft_box() {
        *text = "beat: 2\n".to_owned();
    }
    click(&window, WRITE, |ctx| crate::ui::render(ctx, &mut model));
    let posted = model.outbox.first().expect("one act");
    assert_eq!(posted.envelope["op"], "config");
    assert_eq!(posted.envelope["text"], "beat: 2\n");
    assert_eq!(
        model.drafted().map(|draft| draft.text),
        Some("beat: 2\n".to_owned()),
        "the box is not spent on firing"
    );
}

/// **Reverting puts the file back in the box** and composes nothing.
#[test]
fn the_revert_control_takes_the_engine_s_bytes_and_composes_nothing() {
    let window = Window::new();
    let mut model = configured();
    model.draft_config("roles:\n  worker:\n    provider: gone\n");
    if let Some(text) = model.draft_box() {
        *text = "mine\n".to_owned();
    }
    click(&window, REVERT, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(
        model.drafted().map(|draft| draft.text),
        model.config.as_ref().map(|held| held.text.clone())
    );
    assert!(model.outbox.is_empty());
}

/// **A file that has not answered has no box**: there is nothing to edit, and
/// a box seeded from nothing would invite a write of nothing over a file
/// nobody has seen.
#[test]
fn a_file_that_has_not_answered_offers_no_box() {
    let mut model = Model {
        config: None,
        ..configured()
    };
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    assert!(!painted.contains(WRITE), "{painted}");
    assert_eq!(model.drafted(), None);
}

/// A file that does not exist yet reads as no bytes and is edited all the
/// same — writing it is how it comes to exist.
#[test]
fn an_empty_file_is_still_a_box() {
    let mut model = Model {
        config: Some(crate::reply::config::Config {
            text: String::new(),
            settings: Vec::new(),
        }),
        ..configured()
    };
    let painted = pane(|ui| {
        super::super::render(ui, &mut model);
    });
    assert!(painted.contains(WRITE), "{painted}");
    assert_eq!(model.drafted().map(|draft| draft.text), Some(String::new()));
    assert_eq!(
        model.configured(),
        Some(Where::Brazen {
            workspace: "home".to_owned(),
        })
    );
}
