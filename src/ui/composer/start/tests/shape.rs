//! **One composer, in both of its modes** (DESIGN §4.39): that the start is
//! laid at the deposit's own rows with its act inside the field, that the
//! control still carries both of the start's ops, and that the row under it
//! carries the start's one parameter and none of a conversation's acts.
//!
//! Split from [`super`] because the subject is different: that file reads what
//! the start COMPOSES, and this one reads what it LOOKS like — off the glass
//! and off the accessibility tree, which are the two instruments §4.38's own
//! assertions use.

use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, seated};
use crate::ui::composer::start::START;
use crate::ui::composer::{self, INTERRUPT, MORE, NUDGE, SEND, STOP, offers};
use crate::ui::theme::{self, glyph};
use crate::ui::{Model, records};
use serde_json::json;

/// A model aimed at a wall with nothing selected on it — the start's own case.
fn unselected() -> Model {
    Model {
        conversation: None,
        ..seated()
    }
}

/// **The height of the field the act `word` stands inside**, off the glass.
///
/// The fill is found by containing the act's own laid rect, which is what
/// proves the act is INSIDE the field rather than beside it, and by being
/// wider than a control — the act's own button is a fill containing those same
/// words (`theme::paint::field`'s own assertion reads it the same way).
fn field_height(window: &Window, body: &mut impl FnMut(&egui::Context), word: &str) -> f32 {
    let output = window.frame(Vec::new(), body);
    let act = crate::paint_probe::seen_of(&output)
        .into_iter()
        .find(|run| run.text == word)
        .expect("the act is on the glass");
    crate::paint_probe::fills_of(&output)
        .into_iter()
        .find(|(rect, _)| rect.contains_rect(act.laid) && rect.width() > 300.0)
        .expect("the act stands inside the field")
        .0
        .height()
}

/// **The start is the deposit's own field, at the same rows** (§4.39). Until
/// bl-b3c3 this mode painted a one-line box and a button beside it, so the
/// pane's focal element shrank to a bar whenever nothing was selected — the
/// two were one control that did not look like one.
///
/// Asserted as a height off the glass rather than as a picture, which is how
/// every one of §4.38's composer assertions is stated.
#[test]
fn the_start_field_stands_as_tall_as_the_deposit_s_with_its_act_inside_it() {
    let said = {
        let window = Window::sized(400.0, 400.0);
        let mut model = seated();
        field_height(
            &window,
            &mut |ctx: &egui::Context| {
                egui::CentralPanel::default().show(ctx, |ui| composer::render(ui, &mut model));
            },
            SEND,
        )
    };
    let begun = {
        let window = Window::sized(400.0, 400.0);
        let mut model = unselected();
        field_height(
            &window,
            &mut |ctx: &egui::Context| {
                egui::CentralPanel::default().show(ctx, |ui| composer::render(ui, &mut model));
            },
            START,
        )
    };
    assert!(
        (said - begun).abs() < f32::EPSILON,
        "one field in both modes: the deposit stands {said} and the start {begun}"
    );
}

/// **The parity ledger does not move** (§4.16). Both halves of the start still
/// ride the one control the walk can see — the click composes `prepare` and
/// the frame that absorbs its receipt composes `prompt` from no widget at all
/// — so the tokens are read back off the real accessibility tree, exactly as
/// `crate::snapshot::parity` reads them.
#[test]
fn the_start_control_still_carries_both_of_its_acts() {
    let harness = crate::snapshot::seat(unselected(), 1400.0, 900.0);
    let inventory = crate::snapshot::parity::inventory(&harness);
    for op in [crate::verbs::PREPARE, crate::verbs::PROMPT] {
        assert!(inventory.contains(op), "{op:?} is tagged: {inventory:?}");
    }
}

/// **The row under the start carries the start's one parameter and nothing
/// else** (§4.39). `records…`, `interrupt`, `stop`, `nudge` and the `…` strip
/// are acts on a conversation's turn or on a conversation as an object, and
/// there is no conversation — so they are absent, not greyed, which is the
/// offers row's own rule.
#[test]
fn the_row_in_start_mode_carries_the_role_and_none_of_the_conversation_s_acts() {
    let mut model = unselected();
    let painted = pane(|ui| composer::render(ui, &mut model));
    let plan = theme::worded(glyph::PLAN, offers::PLAN);
    assert!(
        painted.lines().any(|line| line == plan),
        "{plan:?}:\n{painted}"
    );
    for gone in [
        records::OPEN.to_owned(),
        MORE.to_owned(),
        theme::worded(glyph::INTERRUPT, INTERRUPT),
        theme::worded(glyph::STOP, STOP),
        theme::worded(glyph::NUDGE, NUDGE),
    ] {
        assert!(
            !painted.lines().any(|line| line == gone),
            "{gone:?} is an act on a conversation and there is none:\n{painted}"
        );
    }
}

/// **The seat is what states the born-on role** (REMOTE §9.21): pressed, the
/// fire this window composes from the staging receipt carries `planner`;
/// unpressed, the body is byte-identical to what the engine answered.
#[test]
fn the_plan_seat_writes_the_born_on_role_onto_the_fire_and_nothing_else_does() {
    for (planning, born) in [(false, None), (true, Some(json!("planner")))] {
        let mut model = unselected();
        model.draft = "do the thing".to_owned();
        let window = Window::sized(400.0, 400.0);
        let mut body = |ctx: &egui::Context| {
            egui::CentralPanel::default().show(ctx, |ui| composer::render(ui, &mut model));
        };
        if planning {
            let plan = theme::worded(glyph::PLAN, offers::PLAN);
            click(&window, &plan, &mut body);
        }
        click(&window, START, &mut body);
        assert_eq!(
            model.plan, planning,
            "the seat holds what it was pressed to"
        );
        model.absorb(
            &crate::test_support::window::own().channel,
            crate::reply::read(&json!({"ok": true, "kind": "prepared",
                                       "prepared": {"workspace": "home", "goal": ""}})),
        );
        let fire = model
            .outbox
            .iter()
            .find(|said| said.envelope["op"] == json!("prompt"))
            .expect("the fire is composed from the receipt");
        assert_eq!(
            fire.envelope["prepared"].get(crate::verbs::start::ROLE),
            born.as_ref(),
            "planning: {planning}"
        );
    }
}
