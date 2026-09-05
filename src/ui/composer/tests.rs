//! The composer: what it refuses to fire, what it composes when it does, and
//! the draft that survives a mis-click.

use super::{INTERRUPT, NOWHERE, NUDGE, SEND, render, start};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, seated};
use crate::ui::Model;
use serde_json::json;

/// **One box, three subjects, and what decides is the selection.** With no wall
/// aimed at there is neither a conversation to speak to nor one to begin, and
/// that is the only case the composer refuses outright — a wall with nothing
/// selected on it is where a conversation is *begun*, which used to be half of
/// this refusal.
#[test]
fn what_the_composer_is_for_follows_from_what_is_selected() {
    for (model, expected) in [
        (Model::default(), NOWHERE),
        (
            Model {
                aim: None,
                ..seated()
            },
            NOWHERE,
        ),
        (
            Model {
                conversation: None,
                ..seated()
            },
            start::START,
        ),
        (seated(), SEND),
    ] {
        let mut model = model;
        let painted = pane(|ui| render(ui, &mut model));
        assert!(
            painted.lines().any(|line| line == expected),
            "{expected:?}:\n{painted}"
        );
    }
}

/// **It composes and does not send.** The gesture lands in the outbox, built by
/// the same verb row `lernie message` spends — so a click and a typed command
/// build one object.
#[test]
fn sending_composes_the_deposit_the_command_line_would_have_and_posts_nothing() {
    let mut model = seated();
    model.draft = "ship it".to_owned();
    let window = Window::new();
    click(&window, SEND, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "message", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2", "content": "ship it"})
        )]
    );
    assert_eq!(model.draft, "", "what was sent is no longer a draft");
}

/// **The cut is the deposit with a different word on it**, and it spends the
/// same box: one box, and the verb is chosen by which control was pressed. So
/// the two share a body and this asserts the half that differs — the envelope's
/// own op, and that the draft went with it.
#[test]
fn cutting_composes_the_interrupt_off_the_same_box_the_deposit_spends() {
    let mut model = seated();
    model.draft = "no, this".to_owned();
    let window = Window::new();
    click(&window, INTERRUPT, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "interrupt", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2", "content": "no, this"})
        )]
    );
    assert_eq!(model.draft, "", "what was said is no longer a draft");
}

/// **An empty cut fires nothing either**, and for a sharper reason than an
/// empty deposit's: a driver killed with nothing said is `stop`, which is its
/// own control one row down. The guard is the deposit's own, shared rather than
/// restated, and this is the direction that proves the sharing.
#[test]
fn an_empty_draft_cuts_nothing_because_that_gesture_has_its_own_control() {
    let mut model = seated();
    let window = Window::new();
    click(&window, INTERRUPT, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
}

/// **An empty draft fires nothing**, and a draft that was not sent survives:
/// the content crosses verbatim and an empty message is a turn nobody asked
/// for, while a mis-click that cost what was typed is unforgivable.
#[test]
fn an_empty_draft_fires_nothing_and_costs_nothing() {
    let mut model = seated();
    model.draft = "   ".to_owned();
    let window = Window::new();
    click(&window, SEND, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
    assert_eq!(model.draft, "   ", "the draft is still the operator's");
}

/// **The address is the aim's, not the row's name.** They differ exactly where
/// an entry renames, and a deposit that carried the host's spelling would be
/// routed to this box's own engine instead.
#[test]
fn the_deposit_carries_the_address_the_channel_resolves() {
    let mut model = seated();
    model.aim = Some(crate::ui::Aim {
        channel: "home".to_owned(),
        address: "home".to_owned(),
    });
    model.draft = "ship it".to_owned();
    let window = Window::new();
    click(&window, SEND, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert_eq!(model.outbox[0].envelope["workspace"], json!("home"));
}

/// **Enter sends.** A composer an operator has to leave the keyboard for, once
/// per message, is a composer they stop using; the button beside it is how they
/// find out Enter works at all.
#[test]
fn enter_sends_what_was_typed() {
    let mut model = seated();
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    };
    let at = crate::paint_probe::frame::locate_in(&window, SEND, &mut body).expect("the hint");
    crate::paint_probe::frame::click(&window, at, &mut body);
    window.frame(vec![egui::Event::Text("ship it".to_owned())], &mut body);
    window.frame(
        vec![crate::paint_probe::frame::press(egui::Key::Enter)],
        &mut body,
    );
    window.frame(Vec::new(), &mut body);
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "message", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2", "content": "ship it"})
        )]
    );
}

/// **Every control here is already keyboard-operable, and this proves it rather
/// than assuming it.** egui moves focus with Tab and fires a focused control
/// with Space, so neither act wants a binding of its own — and a binding that
/// could fire something a click cannot would be a second surface.
#[test]
fn tab_and_space_fire_the_composer_s_controls_with_no_binding_of_their_own() {
    let mut model = seated();
    model.draft = "ship it".to_owned();
    let window = Window::new();
    // The body borrows the model, so it lives in a scope of its own and the
    // assertions read the model back after it.
    let mut reached = Vec::new();
    {
        let mut body = |ctx: &egui::Context| {
            egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
        };
        window.frame(Vec::new(), &mut body);
        for _ in 0..6 {
            window.frame(
                vec![crate::paint_probe::frame::press(egui::Key::Tab)],
                &mut body,
            );
            reached.push(window.focused());
            window.frame(
                vec![crate::paint_probe::frame::press(egui::Key::Space)],
                &mut body,
            );
            window.frame(Vec::new(), &mut body);
        }
    }
    assert!(
        reached.contains(&Some(egui::Id::new(crate::ui::keys::BOX_ID))),
        "the box is in the tab order too: {reached:?}"
    );
    for op in ["message", "nudge"] {
        assert!(
            model
                .outbox
                .iter()
                .any(|said| said.envelope["op"] == json!(op)),
            "{op:?} was never fired from the keyboard: {:?}",
            model.outbox
        );
    }
}

/// **The advance is a control beside the composer**, because it is the one
/// thing an operator does to a conversation with nothing to say — and it is
/// composed through the same table, so it carries no draft with it.
#[test]
fn the_advance_composes_its_own_gesture_and_takes_no_draft() {
    let mut model = seated();
    model.draft = "not this".to_owned();
    let window = Window::new();
    click(&window, NUDGE, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "nudge", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2"})
        )]
    );
    assert_eq!(model.draft, "not this", "the draft is untouched");
}

/// **The one case with no box at all.** A conversation this window started is
/// not addressable until its driver writes the branch, and this seat knows it:
/// the start's own sentence stands where the box was, so nothing can be
/// composed that this end already knew the engine would refuse.
#[test]
fn a_started_conversation_the_engine_cannot_resolve_yet_has_no_box() {
    let mut model = Model {
        conversation: Some("brisk-otter".to_owned()),
        start: Some(crate::ui::model::Start {
            address: "home".to_owned(),
            goal: "port it".to_owned(),
            phase: crate::ui::model::Phase::Started("brisk-otter".to_owned()),
            spread: None,
        }),
        ..seated()
    };
    let painted = pane(|ui| render(ui, &mut model));
    assert!(
        painted.contains("started «brisk-otter» in home"),
        "{painted}"
    );
    for gone in [SEND, NUDGE] {
        assert!(
            !painted.lines().any(|line| line == gone),
            "{gone:?} is still on the glass: {painted}"
        );
    }
    assert!(model.outbox.is_empty());
}
