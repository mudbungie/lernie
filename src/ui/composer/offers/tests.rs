//! The row under the field: which acts the engine's answer puts on it, the
//! control that opens the strip, and the box a row menu sends the cursor to.

use super::{MORE, STOP, render};
use crate::paint_probe::frame::Window;
use crate::reply::agent::Offer;
use crate::test_support::window::{click, own_row, pane, seated};
use crate::ui::composer::{INTERRUPT, NUDGE, acts};
use crate::ui::theme::{glyph, worded};
use crate::ui::{Aim, Fill, Model};
use serde_json::json;

/// The aim and the conversation the [`seated`] fixture holds, spelled once.
fn subject(model: &Model) -> (Aim, String) {
    (
        model.aim.clone().expect("the fixture is aimed at a wall"),
        model.conversation.clone().expect("and has one selected"),
    )
}

/// The seated model with the engine's row answered, offering `offers`.
fn offering(offers: Vec<Offer>) -> Model {
    let mut model = seated();
    model.records.agent = Some(crate::reply::agent::Agent {
        offers,
        ..own_row()
    });
    model
}

/// One idle frame of the row, as text.
fn painted(model: &mut Model) -> String {
    let (aim, agent) = subject(model);
    pane(|ui| render(ui, model, &aim, &agent))
}

/// Click the seat reading `label` on the row.
fn press(model: &mut Model, label: &str) {
    let (aim, agent) = subject(model);
    let window = Window::new();
    click(&window, label, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, model, &aim, &agent));
    });
}

/// **Only what the conversation offers is on the row** (bl-f251): a running
/// driver is offered the cut and the stop, a quiet one the advance, and a
/// conversation the engine has not answered about is offered all three.
#[test]
fn the_row_offers_what_the_engine_offers_and_everything_before_it_answers() {
    let cut = worded(glyph::INTERRUPT, INTERRUPT);
    let advance = worded(glyph::NUDGE, NUDGE);
    let halt = worded(glyph::STOP, STOP);
    for (offers, shown, hidden) in [
        (Some(vec![Offer::Stop]), vec![&cut, &halt], vec![&advance]),
        (Some(vec![Offer::Nudge]), vec![&advance], vec![&cut, &halt]),
        (Some(vec![]), vec![], vec![&cut, &halt, &advance]),
        (None, vec![&cut, &halt, &advance], vec![]),
    ] {
        let mut model = match offers.clone() {
            Some(offers) => offering(offers),
            None => seated(),
        };
        let painted = painted(&mut model);
        for word in shown {
            assert!(
                painted.lines().any(|l| l == word),
                "{offers:?} offers {word:?}:\n{painted}"
            );
        }
        for word in hidden {
            assert!(
                !painted.lines().any(|l| l == word),
                "{offers:?} withholds {word:?}:\n{painted}"
            );
        }
        assert!(
            painted.lines().any(|l| l == crate::ui::records::OPEN),
            "{painted}"
        );
        assert!(painted.lines().any(|l| l == MORE), "{painted}");
    }
}

/// **The stop composes the bare form**, off the row, addressed by the aim.
#[test]
fn the_stop_composes_the_bare_envelope() {
    let mut model = offering(vec![Offer::Stop]);
    press(&mut model, &worded(glyph::STOP, STOP));
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "stop", "workspace": "home", "agent": "20260830T051200Z-a1b2"})
        )]
    );
}

/// **The strip is behind the more control**: closed, none of its acts is on
/// the glass; one click opens it, and a second folds it away.
#[test]
fn the_more_control_opens_the_strip_and_closes_it_again() {
    let mut model = seated();
    let (aim, agent) = subject(&model);
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model, &aim, &agent));
    };
    let strip = |text: &str| text.lines().any(|line| line == acts::DELETE);
    assert!(!strip(&window.text(&mut body)), "closed at rest");
    let at = crate::paint_probe::frame::locate_in(&window, MORE, &mut body).expect("the control");
    crate::paint_probe::frame::click(&window, at, &mut body);
    assert!(strip(&window.text(&mut body)), "open after one click");
    crate::paint_probe::frame::click(&window, at, &mut body);
    assert!(!strip(&window.text(&mut body)), "closed after the second");
}

/// **A row menu's *delete…* opens the strip and lands the cursor in the
/// arming box** (bl-dbc9, read through the fold): a box the operator was sent
/// to cannot be behind a control they have not pressed.
#[test]
fn a_fill_request_opens_the_strip_and_seats_the_cursor() {
    let mut model = seated();
    let (aim, agent) = subject(&model);
    model.fill_in(&agent, Fill::Arming);
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model, &aim, &agent));
    };
    let text = window.text(&mut body);
    assert!(text.lines().any(|line| line == acts::ARM), "{text}");
    window.frame(Vec::new(), &mut body);
    assert_eq!(
        window.focused(),
        Some(egui::Id::new(crate::ui::keys::ARM_ID)),
        "the arming box holds the caret"
    );
}

/// **The open control stands the records pane up**, tagged with the reads it
/// stands up — and it composes nothing, because a look is not an act.
#[test]
fn the_records_control_opens_the_pane_and_composes_nothing() {
    let mut model = seated();
    press(&mut model, crate::ui::records::OPEN);
    assert!(model.showing(crate::ui::Listing::Records));
    assert!(model.outbox.is_empty());
}
