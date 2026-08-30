//! The composer: what it refuses to fire, what it composes when it does, and
//! the draft that survives a mis-click.

use super::{NOWHERE, NUDGE, SEND, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, seated};
use crate::ui::Model;
use serde_json::json;

/// With nothing aimed at there is nothing to say it to, and the refusal names
/// **both** halves of the address — either can be the one that is missing, and
/// a bare "nothing selected" makes the operator guess which.
#[test]
fn with_no_address_the_composer_names_both_halves_of_one() {
    for model in [
        Model::default(),
        Model {
            conversation: None,
            ..seated()
        },
        Model {
            aim: None,
            ..seated()
        },
    ] {
        let mut model = model;
        let painted = pane(|ui| render(ui, &mut model));
        assert!(painted.contains(NOWHERE), "{painted}");
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
        vec![json!({"op": "message", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2", "content": "ship it"})]
    );
    assert_eq!(model.draft, "", "what was sent is no longer a draft");
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
    assert_eq!(model.outbox[0]["workspace"], json!("home"));
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
        vec![json!({"op": "message", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2", "content": "ship it"})]
    );
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
        vec![json!({"op": "nudge", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2"})]
    );
    assert_eq!(model.draft, "not this", "the draft is untouched");
}
