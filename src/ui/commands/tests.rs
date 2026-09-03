//! What the commands pane says in every state it can be in, and the control
//! that opens it driven through the real window.

use super::{CLOSE, HEADING, NO_OPS, NOT_ANSWERED, OPEN, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, commanded, helped, own, pane, seated};
use crate::ui::{Lookup, Model, Pages};

/// A closed pane paints nothing and says so, which is what lets the shell put
/// the conversation back where it was.
#[test]
fn a_shut_pane_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood, "a shut pane reports that it painted nothing");
    assert!(!painted.contains(HEADING), "{painted}");
}

/// **The answered pane paints every line a row can carry**, under the section
/// header of the channel it came down — including the classification, because
/// an op marked `machine` is one nothing here owes a control and saying so is
/// the difference between a short pane and an incomplete one.
#[test]
fn the_answered_pane_paints_the_line_the_sentence_the_page_and_who_it_is_for() {
    let mut model = commanded();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "(this box's own engine)",
        "/message <workspace>  [control]",
        "what message is for",
        "the page for message, at the length a page runs to",
        "/invocations <workspace>  [machine]",
        "what invocations is for",
        CLOSE,
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **The two empty states are different sentences.** Nobody answered yet is
/// not *this engine names no op*, which is the roster's own doctrine one noun
/// over.
#[test]
fn every_empty_state_is_its_own_sentence() {
    let unanswered = Model {
        lookup: Some(Lookup::Commands),
        ..seated()
    };
    let quiet = Model {
        pages: vec![Pages {
            channel: own().channel,
            rows: Vec::new(),
        }],
        ..unanswered.clone()
    };
    for (mut model, expected, absent) in [
        (unanswered, NOT_ANSWERED, NO_OPS),
        (quiet, NO_OPS, NOT_ANSWERED),
    ] {
        let painted = pane(|ui| {
            render(ui, &mut model);
        });
        assert!(painted.contains(expected), "{expected:?}:\n{painted}");
        assert!(!painted.contains(absent), "{absent:?}:\n{painted}");
    }
}

/// **The header carries the address the entry dials**, so two entries
/// terminating at one listener are as visible here as they are on the roster.
#[test]
fn a_section_wears_the_rosters_own_header() {
    let mut model = Model {
        lookup: Some(Lookup::Commands),
        pages: vec![Pages {
            channel: crate::ui::Channel {
                name: "elsewhere".to_owned(),
                named_there: None,
                dials: Some("a-host:9000".to_owned()),
            },
            rows: vec![helped("scan", "control")],
        }],
        ..seated()
    };
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains("a-host:9000"), "{painted}");
}

/// **The pane opens from the roster and closes from its own control**, and
/// opening it composes the ask — which is why the opening control is the one
/// that carries `help`'s token.
#[test]
fn the_roster_opens_it_asking_for_the_table_and_its_own_word_shuts_it() {
    let window = Window::new();
    let mut model = seated();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.commanding());
    assert_eq!(model.outbox, vec![crate::verbs::window::help()]);
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.commanding());
}
