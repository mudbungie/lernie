//! The composer: what it refuses to fire, what it composes when it does, and
//! the draft that survives a mis-click.

/// What the keyboard does to the composer: the two keys of the field, and
/// the Tab order every control is already in.
mod keys;
/// What a refused act does to the box it came out of.
mod refund;

use super::{ASKING, HINT, INTERRUPT, NOWHERE, NUDGE, SEND, render, start};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, seated};
use crate::ui::Model;
use crate::ui::theme::{self, glyph};
use serde_json::json;

/// The compact control's label, as the operator reads it.
fn worded(glyph: &str, word: &str) -> String {
    theme::worded(glyph, word)
}

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
    click(&window, &worded(glyph::INTERRUPT, INTERRUPT), |ctx| {
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
    click(&window, &worded(glyph::INTERRUPT, INTERRUPT), |ctx| {
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

/// **The advance is a control beside the composer**, because it is the one
/// thing an operator does to a conversation with nothing to say — and it is
/// composed through the same table, so it carries no draft with it.
#[test]
fn the_advance_composes_its_own_gesture_and_takes_no_draft() {
    let mut model = seated();
    model.draft = "not this".to_owned();
    let window = Window::new();
    click(&window, &worded(glyph::NUDGE, NUDGE), |ctx| {
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
    for gone in [SEND.to_owned(), worded(glyph::NUDGE, NUDGE)] {
        assert!(
            !painted.lines().any(|line| line == gone),
            "{gone:?} is still on the glass: {painted}"
        );
    }
    assert!(model.outbox.is_empty());
}

/// **The field glows while the selected conversation is asking and its hint
/// says so** (`docs/STYLE.md` §2): the attention tint stands under the box on
/// an asking row and not on a quiet one, the hint names the asking there and
/// the keys elsewhere, and the send wears the brand on both.
#[test]
fn the_field_glows_while_the_conversation_is_asking_and_the_send_wears_the_brand() {
    let glow = crate::ui::theme::tint(crate::ui::theme::State::Attention);
    for (attention, glows) in [(0, false), (2, true)] {
        let mut model = seated();
        model.convs[0].attention = attention;
        let window = Window::new();
        let output = window.frame(Vec::new(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
        });
        let glowing = crate::paint_probe::fills_of(&output)
            .iter()
            .any(|(_, ink)| *ink == glow);
        assert_eq!(glowing, glows, "attention {attention}");
        let hint = if glows { ASKING } else { HINT };
        assert!(
            crate::paint_probe::seen_of(&output)
                .iter()
                .any(|run| run.text == hint),
            "the hint reads {hint:?}"
        );
        let send = crate::paint_probe::seen_of(&output)
            .into_iter()
            .rfind(|run| run.text == SEND)
            .expect("the send is on the glass");
        assert_eq!(send.ink, crate::ui::theme::BRAND);
    }
}
