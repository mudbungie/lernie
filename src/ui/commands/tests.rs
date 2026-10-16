//! What the commands pane says in every state it can be in, and the control
//! that opens it driven through the real window.

use super::{HEADING, NO_OPS, NOT_ANSWERED, WORD, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{commanded, escape, helped, own, pane, seated};
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
    assert!(crate::ui::slash::fire(&mut model, WORD, ""));
    assert!(model.commanding());
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::read(crate::verbs::window::help())]
    );
    escape(&window, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.commanding());
}

/// **The window answers first** (bl-55ed): every row of the menu, in the
/// menu's order, and every key — then the engines' tables under a heading
/// that says they are the engines' spelling, not the box's.
#[test]
fn the_window_s_own_words_and_keys_come_before_the_engines() {
    let mut model = commanded();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    // Each line's place on the glass, which must rise as the menu's order does.
    let places: Vec<usize> = crate::ui::slash::rows(&model)
        .iter()
        .map(|row| {
            let line = row.line();
            painted
                .match_indices(&line)
                .map(|(at, _)| at)
                .next()
                .unwrap_or_else(|| panic!("{line:?}:\n{painted}"))
        })
        .collect();
    assert!(places.is_sorted(), "the menu's order:\n{painted}");
    let at = places.last().copied().unwrap_or_default();
    for row in crate::ui::keys::BINDINGS {
        let line = format!("{}  {}", row.says(), row.does);
        assert!(painted.contains(&line), "{line}:\n{painted}");
    }
    let engines = painted.find(super::ENGINES).expect("the engines' heading");
    let message = painted.find("/message <workspace>").expect("an engine row");
    assert!(at < engines && engines < message, "{painted}");
}
