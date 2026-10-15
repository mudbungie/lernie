//! **A trashcan on each row under the pointer deletes the conversation**
//! (DESIGN §4.44 item 1; bl-e837): shown under the pointer and gone off it,
//! its click asks — the confirmation on the box naming the conversation —
//! Esc keeps, Enter deletes, and Ctrl+click deletes at once.

use super::{DESK, ROW, bar, tick};
use crate::paint_probe::{Seen, frame::press, seen_of};
use crate::test_support::window::{conv, seated};
use crate::ui::bar::confirm;
use crate::ui::{Model, theme};

/// The trashcan the bar painted, if any.
fn trash(output: &egui::FullOutput) -> Option<Seen> {
    bar(output)
        .into_iter()
        .find(|run| run.text == theme::glyph::TRASH)
}

/// **One frame carrying `events`**, Ctrl held or not, at `time`.
fn frame(
    ctx: &egui::Context,
    model: &mut Model,
    time: f64,
    events: Vec<egui::Event>,
    ctrl: bool,
) -> egui::FullOutput {
    let mut input = crate::paint_probe::frame::screen_sized(DESK, 900.0);
    input.time = Some(time);
    input.events = events;
    if ctrl {
        input.modifiers = egui::Modifiers {
            ctrl: true,
            command: true,
            ..egui::Modifiers::default()
        };
    }
    ctx.run(input, |ctx| crate::ui::render(ctx, model))
}

/// **Click at `at`** — move, press, release — with Ctrl held or not.
fn click(ctx: &egui::Context, model: &mut Model, time: f64, at: egui::Pos2, ctrl: bool) {
    frame(ctx, model, time, vec![egui::Event::PointerMoved(at)], ctrl);
    for (pressed, later) in [(true, 0.01), (false, 0.02)] {
        let button = egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        };
        frame(ctx, model, time + later, vec![button], ctrl);
    }
}

/// The other conversation's id and name.
const OTHER: &str = "20260830T051300Z-c3d4";
const NAME: &str = "the other";

#[test]
fn the_trashcan_shows_under_the_pointer_asks_on_a_click_and_fires_at_once_with_ctrl() {
    let ctx = egui::Context::default();
    let mut model = seated();
    model.bar.pinned = true;
    model.convs.push(conv(OTHER, NAME));
    let row_text = format!("{} {NAME}", theme::glyph::AT_REST);
    let rested = tick(&ctx, &mut model, 0.0, None);
    assert!(trash(&rested).is_none(), "no pointer, no trashcan");
    let row = bar(&rested)
        .into_iter()
        .find(|run| run.text == row_text)
        .expect("the other row")
        .laid;
    let over = Some(row.center());
    tick(&ctx, &mut model, 0.1, over);
    let shown = trash(&tick(&ctx, &mut model, 0.2, over)).expect("under the pointer, the trashcan");
    assert_eq!(shown.ink, theme::INK_FAINT, "faint");
    assert!(
        shown.laid.min.x > row.max.x,
        "at the row's right end: {:?}",
        shown.laid
    );
    assert!(
        (shown.laid.center().y - row.center().y).abs() < theme::ROW / 2.0,
        "on the row's own line"
    );
    let only_this = |output: &egui::FullOutput| {
        bar(output)
            .into_iter()
            .filter(|run| run.text == theme::glyph::TRASH)
            .count()
    };
    assert_eq!(
        only_this(&tick(&ctx, &mut model, 0.3, over)),
        1,
        "the row under the pointer alone"
    );
    let away = Some(egui::pos2(900.0, 300.0));
    tick(&ctx, &mut model, 0.4, away);
    assert!(
        trash(&tick(&ctx, &mut model, 0.5, away)).is_none(),
        "gone off it"
    );
    // **A click asks**, and takes nothing of the row's own: the selection
    // stays where it was.
    let at = shown.laid.center();
    click(&ctx, &mut model, 1.0, at, false);
    assert_eq!(model.deleting.as_deref(), Some(OTHER));
    assert_eq!(
        model.conversation.as_deref(),
        Some("20260830T051200Z-a1b2"),
        "not selected"
    );
    assert!(
        model
            .outbox
            .iter()
            .all(|p| p.envelope["op"] != "delete-agent"),
        "nothing sent yet"
    );
    let asked = seen_of(&frame(&ctx, &mut model, 2.0, Vec::new(), false));
    let says = |text: &str| asked.iter().any(|run| run.text == text);
    assert!(
        says(confirm::HEADING)
            && says(NAME)
            && says(confirm::KEEP)
            && says(confirm::DELETE)
            && says(confirm::HINT),
        "{asked:?}"
    );
    // **Esc keeps**: the question goes and nothing is sent.
    frame(&ctx, &mut model, 2.1, vec![press(egui::Key::Escape)], false);
    assert_eq!(model.deleting, None);
    assert!(
        model
            .outbox
            .iter()
            .all(|p| p.envelope["op"] != "delete-agent"),
        "kept"
    );
    // **Enter deletes**: the envelope `/delete <name>` would send.
    click(&ctx, &mut model, 3.0, at, false);
    assert_eq!(model.deleting.as_deref(), Some(OTHER));
    frame(&ctx, &mut model, 3.1, vec![press(egui::Key::Enter)], false);
    assert_eq!(model.deleting, None);
    let wanted = crate::verbs::delete_agent("home".to_owned(), OTHER.to_owned(), NAME.to_owned());
    assert_eq!(
        model.outbox.last().map(|p| p.envelope.clone()),
        Some(wanted.clone())
    );
    assert!(model.outbox.last().is_some_and(|p| p.act), "an act");
    let before = model.outbox.len();
    // **Ctrl+click deletes at once**, no question asked.
    click(&ctx, &mut model, 4.0, at, true);
    assert_eq!(model.deleting, None, "no confirmation");
    assert_eq!(model.outbox.len(), before + 1);
    assert_eq!(
        model.outbox.last().map(|p| p.envelope.clone()),
        Some(wanted)
    );
    assert!(
        bar(&tick(&ctx, &mut model, 5.0, None))
            .iter()
            .any(|run| run.text == ROW)
    );
}

/// **The confirmation's doors**: `keep` puts it down and `delete` fires it,
/// and the keys are nobody's while no question stands.
#[test]
fn the_doors_answer_and_the_keys_are_free_while_nothing_is_asked() {
    let ctx = egui::Context::default();
    let mut model = seated();
    let id = "20260830T051200Z-a1b2";
    frame(&ctx, &mut model, 0.0, vec![press(egui::Key::Enter)], false);
    assert!(
        model
            .outbox
            .iter()
            .all(|p| p.envelope["op"] != "delete-agent")
    );
    model.begin_deleting(id);
    // **Two frames, because a panel's first frame is laid at a guessed
    // height** and the probe reads what the clip let through.
    let door = |ctx: &egui::Context, model: &mut Model, time: f64, text: &str| {
        frame(ctx, model, time, Vec::new(), false);
        seen_of(&frame(ctx, model, time + 0.01, Vec::new(), false))
            .into_iter()
            .find(|run| run.text == text)
            .unwrap_or_else(|| panic!("{text:?} on the glass"))
            .laid
            .center()
    };
    let keep = door(&ctx, &mut model, 1.0, confirm::KEEP);
    click(&ctx, &mut model, 1.1, keep, false);
    assert_eq!(model.deleting, None);
    assert!(
        model
            .outbox
            .iter()
            .all(|p| p.envelope["op"] != "delete-agent")
    );
    model.begin_deleting(id);
    let delete = door(&ctx, &mut model, 2.0, confirm::DELETE);
    click(&ctx, &mut model, 2.1, delete, false);
    assert_eq!(model.deleting, None);
    assert_eq!(
        model.outbox.last().map(|p| p.envelope.clone()),
        Some(crate::verbs::delete_agent(
            "home".to_owned(),
            id.to_owned(),
            "port the paint probe".to_owned()
        ))
    );
}
