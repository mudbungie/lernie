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
    // The box's own key line, because its name is also the field's hint
    // while the row still says it asks.
    assert!(!text.contains(HINT), "{text}");
}

/// **The choices are the queue's acts**: `seen` alone where nothing is
/// parked, the three scopes of `pass` then `refuse` and `hold` where a call
/// is — each worded as the queue pane words it, each tagged with its op.
#[test]
fn the_choices_are_the_queue_s_acts() {
    let model = asked();
    let (_, held) = row(&model).expect("a row");
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
    // The box takes the caret on the first frame and its arrows from the
    // second: the keys arrive after the window has painted once.
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
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

/// **The box keeps the caret while it asks, and lends the ask its keys only
/// while the draft is empty** (bl-1252): the caret is in the prompt on the
/// first frame; a digit there answers and never reaches the draft; with
/// words in the draft a digit is text, an arrow is the box's and Enter sends
/// the words as a message; and a control a Tab landed on keeps its keys.
#[test]
fn the_box_keeps_the_caret_and_an_empty_draft_lends_the_ask_its_keys() {
    let field = egui::Id::new(crate::ui::keys::BOX_ID);
    let typed = |text: &str| egui::Event::Text(text.to_owned());
    let mut model = asked();
    let window = Window::new();
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(window.focused(), Some(field), "the caret is in the box");
    window.frame(vec![press(egui::Key::Num2), typed("2")], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(last_op(&model), Some(json!("answer")));
    assert!(model.draft.is_empty(), "{:?}", model.draft);

    let mut model = asked();
    model.draft = "go on".to_owned();
    window.focus(field);
    for key in [egui::Key::Num1, egui::Key::ArrowDown] {
        window.frame(vec![press(key)], |ctx| crate::ui::render(ctx, &mut model));
    }
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
    window.frame(vec![press(egui::Key::Enter)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(last_op(&model), Some(json!("message")));
    assert_eq!(
        model.outbox.len(),
        1,
        "the words, and no answer beside them"
    );

    let mut model = asked();
    window.focus(egui::Id::new("a control the Tab landed on"));
    window.frame(vec![press(egui::Key::Num1)], |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
}

/// **The field's hint while it asks names the digits and the words**
/// (bl-1252): `1–6` where a call is parked, `1` where only `seen` is offered,
/// and no digits before the queue's row has landed.
#[test]
fn the_hint_names_the_digits_and_the_words() {
    let mut model = asked();
    assert_eq!(
        super::hint(&model),
        "it is asking — 1–6 answers, or say it — Enter sends"
    );
    for section in &mut model.waiting {
        for row in &mut section.rows {
            row.held = None;
        }
    }
    assert!(super::hint(&model).contains("— 1 answers, or say it"));
    model.waiting.clear();
    assert_eq!(super::hint(&model), "it is asking — say it — Enter sends");
    let text = painted(&mut asked());
    assert!(text.contains("1–6 answers"), "{text}");
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

/// **The box goes down on the frame `seen`'s receipt lands** (bl-6802): the
/// receipt carries the queue that remains, filed as that channel's queue
/// answer, so the selected conversation's row is gone with it — and the bar
/// says what was answered, never that this seat could not read it.
#[test]
fn the_box_goes_down_on_the_frame_the_receipt_lands() {
    let mut model = asked();
    let text = painted(&mut model);
    assert!(text.contains(HEADING), "{text}");
    let (selected, display) = row(&model)
        .map(|(_, row)| (row.agent, row.display))
        .expect("a row");
    let remains = json!({
        "workspace": "home", "agent": "c-2", "display": "c-2", "state": "quiescent",
        "uncertain": true, "preview": "", "age_secs": 7, "pending": 0, "says": "",
        "signals": [], "failure": null, "flag": null, "held": null,
    });
    let receipt = crate::reply::read(&json!({
        "ok": true, "kind": "acknowledged", "workspace": "home", "agent": selected,
        "rows": [remains],
    }));
    model.receipt(&crate::test_support::window::own().channel, "seen", receipt);
    assert!(row(&model).is_none());
    let text = painted(&mut model);
    // The box's own key line, because its name is also the field's hint
    // while the row still says it asks.
    assert!(!text.contains(HINT), "{text}");
    assert!(text.contains(&format!("seen {display} — off")), "{text}");
    assert!(!text.contains("could not read"), "{text}");
}

/// **A seen conversation that still asks is said to, not claimed answered**:
/// the receipt's own remaining queue decides the sentence.
#[test]
fn a_seen_row_that_still_asks_is_said_to() {
    let acked = crate::reply::queue::Acknowledged {
        workspace: "home".to_owned(),
        agent: "c-2".to_owned(),
        rows: vec![crate::test_support::window::waiting("home", "c-2")],
    };
    let said = crate::ui::model::Notice::seen(&acked, "c-2").line();
    assert!(said.contains("still asking"), "{said}");
}

/// **While the aimed engine will not dial, nothing is answered** (bl-8d6d):
/// the choices go faint, the box's key line goes, a click, a digit and
/// Enter post nothing, and the field's hint is the engine's own sentence.
#[test]
fn a_down_engine_s_ask_goes_faint_and_answers_nothing() {
    let why = "nothing provisioned for this channel";
    let mut model = asked();
    model.roster[0].held = crate::ui::Held::Unheld(why.to_owned());
    assert_eq!(super::hint(&model), why);
    let window = Window::new();
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let seen = crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, &mut model));
    let seen_row = seen
        .iter()
        .find(|run| run.text == "2. pass")
        .expect("the choices stand");
    assert_eq!(seen_row.ink, theme::INK_FAINT);
    assert!(!seen.iter().any(|run| run.text == HINT), "no key line");
    for key in [egui::Key::Num2, egui::Key::Enter] {
        window.frame(vec![press(key)], |ctx| crate::ui::render(ctx, &mut model));
    }
    click(&window, "6. hold", |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
}
