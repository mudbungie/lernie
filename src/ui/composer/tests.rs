//! The composer: what it refuses to fire, what it composes when it does, and
//! the draft that survives a mis-click.

/// What the keyboard does to the composer: the two keys of the field, and
/// the Tab order every control is already in.
mod keys;
/// What a refused act does to the box it came out of.
mod refund;
/// The box with nothing aimed: what it offers and what it refuses.
mod unaimed;

use super::{HINT, NOWHERE, render, start};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{pane, seated};
use crate::ui::Model;
use crate::ui::theme::glyph;
use serde_json::json;

/// **One box, three subjects, and what decides is the selection.** With no workspace
/// aimed at there is neither a conversation to speak to nor one to begin, and
/// the hint says so over a box that still takes a verb (bl-2cb0) — a workspace
/// with nothing selected on it is where a conversation is *begun*.
#[test]
fn what_the_composer_is_for_follows_from_what_is_selected() {
    // **Built one at a time**: four models side by side are more than the
    // stack array clippy allows (`large_stack_arrays`), so the table holds
    // how to build each rather than the four built.
    let cases: [(fn() -> Model, String); 4] = [
        (Model::default, NOWHERE.to_owned()),
        (
            || Model {
                aim: None,
                ..seated()
            },
            NOWHERE.to_owned(),
        ),
        (
            || Model {
                conversation: None,
                ..seated()
            },
            start::goal("home", false),
        ),
        (seated, HINT.to_owned()),
    ];
    for (build, expected) in cases {
        let mut model = build();
        let painted = pane(|ui| render(ui, &mut model));
        assert!(
            painted.lines().any(|line| line.starts_with(&expected)),
            "{expected:?}:\n{painted}"
        );
    }
}

/// **Enter on the box**: an idle frame, the caret in the box, the key.
fn enter(window: &Window, model: &mut Model) {
    window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, model));
    });
    window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
    window.frame(
        vec![crate::paint_probe::frame::press(egui::Key::Enter)],
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| render(ui, model));
        },
    );
}

/// **It composes and does not send.** The gesture lands in the outbox, built by
/// the same verb row `lernie message` spends — so a key and a typed command
/// build one object.
#[test]
fn sending_composes_the_deposit_the_command_line_would_have_and_posts_nothing() {
    let mut model = seated();
    model.draft = "ship it".to_owned();
    let window = Window::new();
    enter(&window, &mut model);
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "message", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2", "content": "ship it"})
        )]
    );
    assert_eq!(model.draft, "", "what was sent is no longer a draft");
}

/// **The cut is `/interrupt` with its words**, composed through the same verb
/// row the deposit's sibling spends (`crate::ui::slash`), and it takes the
/// words with it.
#[test]
fn cutting_composes_the_interrupt_off_the_slash_line() {
    let mut model = seated();
    assert!(crate::ui::slash::fire(
        &mut model,
        crate::verbs::INTERRUPT.word,
        "no, this"
    ));
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "interrupt", "workspace": "home",
                    "agent": "20260830T051200Z-a1b2", "content": "no, this"})
        )]
    );
}

/// **A cut with nothing to say fires nothing**, because that gesture is
/// `/stop` and has its own word.
#[test]
fn an_empty_cut_fires_nothing_because_that_gesture_has_its_own_word() {
    let mut model = seated();
    assert!(!crate::ui::slash::fire(
        &mut model,
        crate::verbs::INTERRUPT.word,
        ""
    ));
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
    enter(&window, &mut model);
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
    assert_eq!(model.draft, "   ", "the draft is still the operator's");
}

/// **The address is the aim's, not the row's name.** They differ exactly where
/// an entry renames, and a deposit that carried the host's spelling would be
/// routed to this box's own server instead.
#[test]
fn the_deposit_carries_the_address_the_channel_resolves() {
    let mut model = seated();
    model.aim = Some(crate::ui::Aim {
        channel: "home".to_owned(),
        address: "home".to_owned(),
    });
    model.draft = "ship it".to_owned();
    let window = Window::new();
    enter(&window, &mut model);
    assert_eq!(model.outbox[0].envelope["workspace"], json!("home"));
}

/// **The advance composes its own gesture and takes no draft**: `/nudge`
/// spends no words, so what was typed before it is still there after.
#[test]
fn the_advance_composes_its_own_gesture_and_takes_no_draft() {
    let mut model = seated();
    model.draft = "not this".to_owned();
    assert!(crate::ui::slash::fire(
        &mut model,
        crate::verbs::NUDGE.word,
        ""
    ));
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
/// composed that this end already knew the server would refuse.
#[test]
fn a_started_conversation_the_server_cannot_resolve_yet_has_no_box() {
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
    for gone in [HINT, glyph::PROMPT] {
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
/// the keys elsewhere, and the glyph wears the brand on both.
#[test]
fn the_field_glows_while_the_conversation_is_asking_and_the_glyph_wears_the_brand() {
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
        let hint = if glows { crate::ui::ask::HEADING } else { HINT };
        assert!(
            crate::paint_probe::seen_of(&output)
                .iter()
                .any(|run| run.text.starts_with(hint)),
            "the hint reads {hint:?}"
        );
        let lead = crate::paint_probe::seen_of(&output)
            .into_iter()
            .rfind(|run| run.text == glyph::PROMPT)
            .expect("the glyph is on the glass");
        assert_eq!(lead.ink, crate::ui::theme::BRAND);
    }
}

/// **The hint names the ways in while it fits on one line, and drops them
/// from the right where it does not** (bl-4e55): every one at the desk, the
/// lead alone in no room at all, and never a part cut in half.
#[test]
fn the_hint_names_the_ways_in_and_drops_them_from_the_right() {
    let mut wide = String::new();
    let mut narrow = String::new();
    let window = crate::paint_probe::frame::Window::new();
    window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            wide = super::hint(ui, HINT, &super::parts());
            ui.allocate_ui(egui::vec2(40.0, 20.0), |ui| {
                narrow = super::hint(ui, HINT, &super::parts());
            });
        });
    });
    for way in [
        "Ctrl+X for a conversation",
        "Ctrl+O for the model",
        "/login for a provider",
    ] {
        assert!(wide.contains(way), "{way:?}: {wide}");
    }
    assert_eq!(narrow, HINT);
}
