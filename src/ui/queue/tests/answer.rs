//! **What this pane does about a PARKED CALL** — the capability boundary's
//! answer, offered here because this is the one pane that already says what is
//! parked (DESIGN §4.34, REMOTE §5, §9.11).
//!
//! Its own file on the seam [`super`]'s own doc draws: that one is what the
//! queue SAYS in every state it can be in, and this is the one act on it whose
//! subject is not the row but the invocation behind it. It is also the half
//! that grew at PROTOCOL 18, when the answer gained a reach.

use super::super::{WIDER, render, wider};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, queued, waiting};
use crate::ui::{Asking, Model};

/// **The three verdicts are offered on a row that is holding a call, and on no
/// other** — `answer` is scoped to the exact invocation parked at the far end,
/// so a control on a row with nothing parked would fire a gesture the engine
/// refuses by name (bl-bce2).
#[test]
fn the_verdicts_are_offered_only_where_something_is_parked() {
    let mut model = queued();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in crate::verbs::VERDICTS {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
    let quiet = crate::reply::queue::QueueRow {
        held: None,
        ..waiting("home", "20260830T051200Z-a1b2")
    };
    let mut bare = Model {
        waiting: vec![Asking {
            channel: crate::test_support::window::own().channel,
            rows: vec![quiet],
        }],
        ..queued()
    };
    let painted = pane(|ui| {
        render(ui, &mut bare);
    });
    assert!(!painted.contains("refuse"), "{painted}");
}

/// **Firing one composes the answer with that word and its reach, and nothing
/// else**: which call it lands on is read at the far end off the
/// conversation's own hold mark, so this end names no invocation.
///
/// The narrow row's buttons carry `call` — the reading an operator gets
/// without asking for another, and the one the wire requires them to state
/// rather than leave off (PROTOCOL 18).
#[test]
fn a_verdict_composes_the_answer_and_names_no_invocation() {
    let window = Window::new();
    let mut model = queued();
    click(&window, "refuse", |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(serde_json::json!({
            "op": "answer", "workspace": "home", "scope": "call",
            "agent": "20260830T051200Z-a1b2", "verdict": "refuse"
        }))]
    );
}

/// **The two wider reaches are rows of their own, each naming itself** (yog
/// bl-94a5). They are written out per scope rather than picked from a
/// selector, which is what makes *wider* impossible to arrive at by accident:
/// a sticky picker fires the next row at whatever the last one was set to, and
/// the wire refuses a default for that same reason.
#[test]
fn the_wider_reaches_are_offered_as_rows_that_say_what_they_settle() {
    let mut model = queued();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains(WIDER), "{painted}");
    for scope in crate::verbs::SCOPES.into_iter().skip(1) {
        for word in crate::verbs::VERDICTS {
            assert!(
                painted.contains(&wider(word, scope)),
                "{word}/{scope}:\n{painted}"
            );
        }
    }
    // A row with nothing parked offers no reach at all, on the narrow row's
    // own terms: there is no held call for one to be about.
    let quiet = crate::reply::queue::QueueRow {
        held: None,
        ..waiting("home", "20260830T051200Z-a1b2")
    };
    let mut bare = Model {
        waiting: vec![Asking {
            channel: crate::test_support::window::own().channel,
            rows: vec![quiet],
        }],
        ..queued()
    };
    let painted = pane(|ui| {
        render(ui, &mut bare);
    });
    assert!(!painted.contains(WIDER), "{painted}");
}

/// **Firing a wider seat states that reach**, and the seat's own label is what
/// it states — one sentence, pressed and answered, with nothing held between
/// the two frames for it to go stale in.
#[test]
fn a_wider_seat_composes_the_answer_at_the_reach_its_label_names() {
    let window = Window::new();
    let mut model = queued();
    click(&window, &wider("pass", "workspace"), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(serde_json::json!({
            "op": "answer", "workspace": "home", "scope": "workspace",
            "agent": "20260830T051200Z-a1b2", "verdict": "pass"
        }))]
    );
}
