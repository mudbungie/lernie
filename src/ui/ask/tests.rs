//! The ask: when it stands, what it offers, and how it is answered.

use super::{Choice, HEADING, HINT, answer, choices, confirm, moved, pick, row, words};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{asked, click, painted, seated};
use crate::ui::{Model, theme};
use serde_json::json;

/// The op of the last gesture composed.
fn last_op(model: &Model) -> Option<serde_json::Value> {
    model.outbox.last().map(|p| p.envelope["op"].clone())
}

/// **It stands while the selected conversation's row is in what is waiting**,
/// and not otherwise.
#[test]
fn it_stands_on_the_selected_conversation_s_row_and_not_otherwise() {
    assert!(row(&seated()).is_none());
    let mut model = asked();
    assert!(row(&model).is_some());
    // Two frames, because a bottom panel sizes from the last one.
    let window = Window::new();
    window.text(|ctx| crate::ui::render(ctx, &mut model));
    let text = window.text(|ctx| crate::ui::render(ctx, &mut model));
    for word in [
        HEADING,
        HINT,
        "1. seen",
        "2. pass",
        "6. hold",
        "held at the boundary",
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    model.conversation = Some("c-9".to_owned());
    assert!(row(&model).is_none());
    let text = painted(&mut model);
    assert!(!text.contains(HEADING), "{text}");
}

/// **The choices are the queue's acts**: `seen` alone where nothing is
/// parked, the three scopes of `pass` then `refuse` and `hold` where a call
/// is — each worded as the queue pane words it, each tagged with its op.
#[test]
fn the_choices_are_the_queue_s_acts() {
    let model = asked();
    let held = row(&model).expect("a row");
    let offered: Vec<String> = choices(&held).into_iter().map(Choice::word).collect();
    assert_eq!(
        offered,
        vec![
            "seen",
            "pass",
            "pass for this conversation",
            "pass for this workspace",
            "refuse",
            "hold"
        ]
    );
    assert_eq!(Choice::Seen.op(), "seen");
    assert_eq!(Choice::Verdict("pass", "call").op(), "answer");
    let quiet = crate::reply::queue::QueueRow {
        held: None,
        ..held.clone()
    };
    assert_eq!(choices(&quiet), vec![Choice::Seen]);
    let said: Vec<String> = words(&held).into_iter().map(|(s, _)| s).collect();
    assert!(said.iter().any(|s| s.starts_with("held at the boundary")));
    assert!(said.iter().any(|s| s.starts_with("flagged")));
    assert!(said.contains(&"Unauthorized".to_owned()));
}

/// **Each way of answering composes the queue's own gesture**: a digit, the
/// arrows and Enter, a click on a row, and the door itself.
#[test]
fn a_digit_the_arrows_and_a_click_each_answer() {
    let mut model = asked();
    assert!(pick(&mut model, 1));
    assert_eq!(last_op(&model), Some(json!("seen")));
    assert!(pick(&mut model, 3));
    assert_eq!(last_op(&model), Some(json!("answer")));
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["scope"].clone()),
        Some(json!("conversation"))
    );
    assert!(!pick(&mut model, 9), "no ninth row");
    assert!(!pick(&mut model, 0), "the digits count from one");

    let window = Window::new();
    let key = |model: &mut Model, key| {
        window.frame(vec![press(key)], |ctx| crate::ui::render(ctx, model));
    };
    let mut model = asked();
    key(&mut model, egui::Key::ArrowDown);
    key(&mut model, egui::Key::ArrowDown);
    key(&mut model, egui::Key::ArrowUp);
    key(&mut model, egui::Key::Enter);
    assert_eq!(last_op(&model), Some(json!("answer")));
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["verdict"].clone()),
        Some(json!("pass"))
    );
    key(&mut model, egui::Key::Num5);
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["verdict"].clone()),
        Some(json!("refuse"))
    );

    let mut model = asked();
    click(&window, "6. hold", |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(
        model.outbox.last().map(|p| p.envelope["verdict"].clone()),
        Some(json!("hold"))
    );
}

/// **A digit typed into the box is text, not a pick**: the keys are the
/// ask's only while nothing holds the keyboard.
#[test]
fn a_digit_in_the_box_picks_nothing() {
    let mut model = asked();
    let window = Window::new();
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
    window.frame(vec![press(egui::Key::Num1)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
    // And a control holding the keyboard — a Tab landed on it — is egui's
    // to fire, not the ask's: the digit stays with the control.
    window.focus(egui::Id::new("a control the Tab landed on"));
    window.frame(vec![press(egui::Key::Num1)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
}

/// **The doors and the moves are quiet where nothing asks.**
#[test]
fn nothing_asks_nothing_answers() {
    let mut model = seated();
    let window = Window::new();
    window.frame(Vec::new(), |ctx| {
        moved(ctx, &model, 1);
        confirm(ctx, &mut model);
    });
    assert!(model.outbox.is_empty());
    assert!(!pick(&mut model, 1));
    let held = row(&asked()).expect("a row");
    answer(&mut model, &held, Choice::Seen);
    assert_eq!(last_op(&model), Some(json!("seen")));
}

/// **The box wears the attention accent**: its frame's stroke and its name.
#[test]
fn the_box_wears_the_attention_accent() {
    let mut model = asked();
    let window = Window::new();
    let output = window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let accent = theme::accent(theme::State::Attention);
    let name = crate::paint_probe::seen_of(&output)
        .into_iter()
        .find(|run| run.text == HEADING)
        .expect("the name is on the glass");
    assert_eq!(name.ink, accent);
    assert!(
        crate::paint_probe::strokes_of(&output)
            .iter()
            .any(|(_, ink)| *ink == accent),
        "the frame is green"
    );
}
