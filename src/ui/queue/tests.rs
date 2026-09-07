//! What the decision queue says in every state it can be in, the sentences
//! computed beside the paint read as values, and the two controls driven.
//!
//! The capability answer's beats — the verdicts, and the two wider reaches
//! PROTOCOL 18 gave them — are [`answer`], on the seam that subject already
//! has: this file is what the pane SAYS, and that one is the one act on it
//! whose subject is the invocation behind the row rather than the row.

mod answer;

use super::{
    CLOSE, GO, HEADING, NOT_ANSWERED, NOTHING, OPEN, SEEN, flagged, headline, parked, render,
    signalled,
};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, queued, seated, seen, waiting};
use crate::ui::{Asking, Model, theme};

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

/// **The answered pane paints the whole ask**: the section it came down, the
/// headline, the flag that is the point of it, the failure clause, the parked
/// invocation, the signals, the preview and both controls.
#[test]
fn the_answered_pane_paints_every_line_a_row_can_carry() {
    let mut model = queued();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "(this box's own engine)",
        "port the paint probe  on home  [stopped]  5s",
        "2 under it",
        "flagged 2026-09-01T22:10Z — it is rewriting an unrelated crate",
        "Unauthorized",
        "held at the boundary: Bash (toolu_1) — writes",
        "held  mail  flagged",
        "it stopped on the third attempt",
        crate::ui::roster::NO_NAME_HERE,
        SEEN,
        GO,
        CLOSE,
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **The two empty states are different sentences.** Nobody answered yet is
/// not *nothing is waiting*, which is the conversation list's own doctrine one
/// noun over — and a channel that answered nothing is silent rather than a
/// header over a blank.
#[test]
fn every_empty_state_is_its_own_sentence() {
    let unanswered = Model {
        listing: Some(crate::ui::Listing::Queue),
        ..seated()
    };
    let quiet = Model {
        waiting: vec![Asking {
            channel: crate::test_support::window::own().channel,
            rows: Vec::new(),
        }],
        ..unanswered.clone()
    };
    for (mut model, expected, absent) in [
        (unanswered, NOT_ANSWERED, NOTHING),
        (quiet, NOTHING, NOT_ANSWERED),
    ] {
        let painted = pane(|ui| {
            render(ui, &mut model);
        });
        assert!(painted.contains(expected), "{expected:?}:\n{painted}");
        assert!(!painted.contains(absent), "{absent:?}:\n{painted}");
        assert!(
            !painted.contains("(this box's own engine)"),
            "a channel with nothing waiting is silent:\n{painted}"
        );
    }
}

/// **A row nobody flagged, nothing parked and no token names says none of the
/// three** — an absence painted as a line would state a fact nobody stated.
#[test]
fn the_absences_paint_nothing_at_all() {
    let row = waiting("home", "c-2");
    assert_eq!(
        (flagged(&row), parked(&row), signalled(&row)),
        (None, None, None)
    );
    assert_eq!(headline(&row), "c-2  on home  [quiescent?]  7s");
}

/// **The control that answers a row composes `seen` for the address the roster
/// resolved**, driven through the real window rather than by calling the door.
#[test]
fn the_seen_control_composes_the_answer() {
    let window = Window::new();
    let mut model = queued();
    click(&window, SEEN, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(crate::verbs::seen(
            "home".to_owned(),
            "20260830T051200Z-a1b2".to_owned()
        ))]
    );
    assert!(
        model.showing(crate::ui::Listing::Queue),
        "answering a row does not close the pane"
    );
}

/// **The control that leaves for the conversation aims, selects and stands the
/// pane down**, and composes nothing: it is a view.
#[test]
fn the_go_control_leaves_the_pane_for_the_conversation() {
    let window = Window::new();
    let mut model = queued();
    click(&window, GO, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.showing(crate::ui::Listing::Queue));
    assert_eq!(model.conversation.as_deref(), Some("20260830T051200Z-a1b2"));
    assert!(model.outbox.is_empty());
}

/// **The pane opens from the roster and closes from its own control**, which is
/// the walk `crate::snapshot::reach` bounds, driven here for the acts rather
/// than for the bound.
#[test]
fn the_roster_opens_it_and_its_own_word_shuts_it() {
    let window = Window::new();
    let mut model = seated();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.showing(crate::ui::Listing::Queue));
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.showing(crate::ui::Listing::Queue));
}

/// **Every row on this pane wears the attention rule, and none wears the
/// brand** (`docs/STYLE.md` §2, *the eye lands on green*).
///
/// It is the one pane where the green is a tautology rather than a reading: a
/// row is here BECAUSE it is asking for the operator, so the rule is on all
/// three or the pane has stopped meaning what it says. The brand is the other
/// half of the claim — this pane selects nothing, and a brand rule would say
/// the operator is standing on a row they are not.
#[test]
fn every_row_wears_the_attention_rule_and_none_wears_the_brand() {
    let mut model = queued();
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render(ui, &mut model);
        });
    };
    let fills = crate::paint_probe::fills_of(&window.frame(Vec::new(), &mut body));
    let ruled = |ink: egui::Color32| {
        fills
            .iter()
            .filter(|(rect, fill)| *fill == ink && rect.width() <= theme::RULE + 0.5)
            .count()
    };
    assert_eq!(
        ruled(theme::accent(theme::State::Attention)),
        3,
        "one green rule per waiting row"
    );
    assert_eq!(ruled(theme::BRAND), 0, "this pane selects nothing");
    let runs = seen(&window, &mut body);
    let headline = runs
        .iter()
        .find(|run| run.text.starts_with("port the paint probe"))
        .expect("the row's headline reached the glass");
    assert_eq!(headline.ink, theme::INK, "the headline is body ink");
    for (word, ink) in [
        (
            "flagged 2026-09-01T22:10Z",
            theme::accent(theme::State::Annotation),
        ),
        ("Unauthorized", theme::accent(theme::State::Error)),
        (
            "held at the boundary",
            theme::accent(theme::State::Annotation),
        ),
        ("it stopped on the third attempt", theme::INK_WEAK),
    ] {
        let run = runs
            .iter()
            .find(|run| run.text.starts_with(word))
            .unwrap_or_else(|| panic!("{word:?} is not on the glass"));
        assert_eq!(run.ink, ink, "{word:?}");
    }
}
