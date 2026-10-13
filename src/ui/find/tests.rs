//! What the find pane says in every state it can be in, the sentences computed
//! beside the paint read as values, and the act driven through the real window.

use super::{
    FIND, HEADING, NEEDS_WORDS, NOT_ADDRESSABLE, NOT_ASKED, NOTHING, WORD, looked_for, render,
    unread,
};
use crate::paint_probe::frame::Window;
use crate::reply::search::Found;
use crate::test_support::window::{click, escape, finding, hit, own, pane, seated};
use crate::ui::{Hits, Lookup, Model};

/// A closed pane paints nothing and says so.
#[test]
fn a_shut_pane_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood, "a shut pane reports that it painted nothing");
    assert!(!painted.contains(HEADING), "{painted}");
}

/// **The answered pane paints every line a hit can carry**, plus the standing
/// sentence saying why none of them can be acted on (yog bl-ef16) and the
/// engine's own echo of the needle.
#[test]
fn the_answered_pane_paints_the_subject_the_field_the_excerpt_and_why_it_is_read_only() {
    let mut model = finding();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "(this box's own engine)",
        NOT_ADDRESSABLE,
        "looked for \"gate\"",
        "conversation  /ws/home  20260830T051200Z-a1b2",
        "summary +12",
        "the gate said no",
        "could not be read: p: balls unlistable",
        FIND,
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **There is no control that spends a hit**, and that is the ball's scope
/// rather than an omission: a row's workspace is the engine's own path, so a
/// gesture built from it would be refused. The queue's own *go to it* word must
/// not appear here.
#[test]
fn no_control_on_this_pane_spends_a_hit() {
    let mut model = finding();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(!painted.contains(crate::ui::queue::GO), "{painted}");
    // And an idle frame of the whole window over an answered pane composes
    // nothing at all: the only gesture here is the one an operator fires.
    let _ = crate::test_support::window::painted(&mut model);
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
}

/// **The two empty states are different sentences**, and a channel that
/// answered and found nothing says so under its own header rather than being
/// silent — unlike the queue, where a quiet section is noise: here the
/// question was asked of that engine and it answered.
#[test]
fn every_empty_state_is_its_own_sentence() {
    let mut unasked = Model {
        lookup: Some(Lookup::Finding),
        ..seated()
    };
    let painted = pane(|ui| {
        render(ui, &mut unasked);
    });
    assert!(painted.contains(NOT_ASKED), "{painted}");
    assert!(!painted.contains(NOT_ADDRESSABLE), "{painted}");
    let mut empty = Model {
        found: vec![Hits {
            channel: own().channel,
            found: Found {
                needle: "gate".to_owned(),
                rows: Vec::new(),
                unreadable: Vec::new(),
            },
        }],
        ..unasked
    };
    let painted = pane(|ui| {
        render(ui, &mut empty);
    });
    assert!(painted.contains(NOTHING), "{painted}");
    assert!(!painted.contains(NOT_ASKED), "{painted}");
}

/// **The act is disabled and not absent while there is no needle**, and the
/// sentence beside it says what would make it live — a greyed control says a
/// thing is not live and nothing about why.
#[test]
fn the_act_stands_unlive_with_its_reason_beside_it() {
    let mut unarmed = Model {
        lookup: Some(Lookup::Finding),
        ..seated()
    };
    let painted = pane(|ui| {
        render(ui, &mut unarmed);
    });
    assert!(painted.contains(FIND), "the control stays on the glass");
    assert!(painted.contains(NEEDS_WORDS), "{painted}");
    let mut armed = finding();
    let painted = pane(|ui| {
        render(ui, &mut armed);
    });
    assert!(!painted.contains(NEEDS_WORDS), "{painted}");
}

/// The two sentences computed beside the paint, read as values.
#[test]
fn what_was_looked_for_and_what_could_not_be_read_are_two_claims() {
    let found = Found {
        needle: "gate".to_owned(),
        rows: vec![hit("ball")],
        unreadable: vec!["p: balls unlistable".to_owned()],
    };
    assert_eq!(looked_for(&found), "looked for \"gate\"");
    assert_eq!(
        unread(&found.unreadable[0]),
        "could not be read: p: balls unlistable"
    );
}

/// **The pane opens from the roster asking nothing, and its own control spends
/// the needle** — driven through the real window rather than by calling the
/// door.
#[test]
fn the_roster_opens_it_and_the_act_inside_it_composes_the_search() {
    let window = Window::new();
    let mut model = seated();
    assert!(crate::ui::slash::fire(&mut model, WORD, ""));
    assert!(model.finding());
    assert!(model.outbox.is_empty(), "opening asks nothing");
    model.needle = "gate".to_owned();
    click(&window, FIND, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::read(crate::verbs::search(
            "gate".to_owned()
        ))]
    );
    assert!(model.finding(), "searching does not close the pane");
    escape(&window, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.finding());
}

/// **A needle with no space in it stays inside the box** (bl-4f8d): every
/// glyph of the answer is laid within the phone-shaped window — the echo of a
/// 300-character needle and an excerpt as long.
#[test]
fn a_needle_with_no_space_in_it_is_laid_inside_the_window() {
    let long = "x".repeat(300);
    let mut model = finding();
    model.needle.clone_from(&long);
    model.found[0].found.needle.clone_from(&long);
    model.found[0].found.rows[0].excerpt = "y".repeat(300);
    let window = Window::sized(420.0, 820.0);
    for _ in 0..2 {
        window.text(|ctx| crate::ui::render(ctx, &mut model));
    }
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let seen = crate::paint_probe::seen_of(&output);
    assert!(
        seen.iter().any(|run| run.text.contains("xxxxxxxxxx")),
        "the needle is painted"
    );
    // The needle's own field scrolls inside itself, as a one-line box does;
    // every other run is the pane's and must be laid inside the window.
    for run in seen.into_iter().filter(|run| run.text != long) {
        assert!(
            run.laid.max.x <= 420.5,
            "{:?} laid to {}",
            run.text,
            run.laid.max.x
        );
    }
}
