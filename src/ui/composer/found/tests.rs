//! The founding line (bl-673e): what it asks, what Enter composes, and what
//! it refuses to compose.

use super::{HINT, render};
use crate::paint_probe::frame::{Window, press};
use crate::test_support::window::{pane, seated};
use crate::ui::{Aim, Model};
use serde_json::json;

/// A seat whose box was just sent to found a workspace.
fn founding() -> Model {
    let mut model = Model {
        conversation: None,
        ..seated()
    };
    model.begin_founding(crate::seat::OWN);
    model
}

/// Type nothing, press Enter on whatever the draft holds.
fn entered(model: &mut Model) {
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, model));
    };
    window.frame(Vec::new(), &mut body);
    window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
    window.frame(vec![press(egui::Key::Enter)], &mut body);
}

/// **The hint says the name is typed first**, and the box takes the caret.
#[test]
fn the_box_asks_for_a_name_then_a_goal() {
    let mut model = founding();
    let painted = pane(|ui| crate::ui::composer::render(ui, &mut model));
    assert!(painted.contains(HINT), "{painted}");
    assert_eq!(model.fill, None, "the caret request was taken");
}

/// **`<name> <goal>` founds the workspace by starting on it**: the posted
/// envelope is the start's own `prepare` naming the new workspace and the goal
/// is held for the fire — and **nothing is aimed until the `prepare` is
/// answered** (bl-693d): the workspace's reads asked ahead of the act that
/// registers this client were refused into the bar.
#[test]
fn enter_stages_a_start_on_the_named_workspace() {
    let mut model = founding();
    let before = model.aim.clone();
    model.draft = "lab  fix the build".to_owned();
    entered(&mut model);
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(
            json!({"op": "prepare", "workspace": "lab", "payload": {"rung": "bare"}})
        )]
    );
    assert_eq!(
        model.aim, before,
        "nothing is asked of a workspace not founded yet"
    );
    assert!(model.founding.is_some());
    assert_eq!(
        model.start.as_ref().map(|start| start.goal.clone()),
        Some("fix the build".to_owned())
    );
    let own = model.roster[0].channel.clone();
    model.receipt(&own, "message", crate::reply::Read::Refusal("x".to_owned()));
    assert!(model.founding.is_some(), "only the prepare's answer aims");
    model.receipt(
        &own,
        "prepare",
        crate::reply::Read::Refusal("no key".to_owned()),
    );
    assert_eq!(
        model.aim,
        Some(Aim {
            channel: crate::seat::OWN.to_owned(),
            address: "lab".to_owned(),
        })
    );
    assert_eq!(model.founding, None, "the aim put the founding down");
}

/// **A `prepare` that never left this box aims too**: the founding is put
/// down with the goal back in the box, under the workspace it named.
#[test]
fn an_unsent_founding_aims_and_gives_the_goal_back() {
    let mut model = founding();
    model.draft = "lab fix the build".to_owned();
    entered(&mut model);
    let gone = crate::channel::Reach::Unsent("no route".to_owned());
    model.acted("prepare", &gone, None);
    assert_eq!(model.founding, None);
    assert_eq!(model.aim.map(|aim| aim.address), Some("lab".to_owned()));
    assert_eq!(model.draft, "fix the build");
}

/// **A name with no goal composes nothing**, the start's own rule, and the
/// draft stands for the rest of the line.
#[test]
fn a_name_alone_composes_nothing() {
    let mut model = founding();
    model.draft = "lab ".to_owned();
    entered(&mut model);
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
    assert!(model.founding.is_some());
    assert_eq!(model.draft, "lab ");
}

/// **A slash line is the menu's**, here as in every mode of the box.
#[test]
fn a_slash_line_founds_nothing() {
    let mut model = founding();
    model.draft = "/help me".to_owned();
    entered(&mut model);
    assert!(model.outbox.is_empty(), "{:?}", model.outbox);
}

/// **Esc puts the founding down**, and so does choosing anything.
#[test]
fn esc_and_a_selection_put_it_down() {
    let mut model = founding();
    model.escape();
    assert_eq!(model.founding, None);
    let mut model = founding();
    model.select("home/x");
    assert_eq!(model.founding, None);
}

/// **The parity ledger reads the start's two acts off the founding box**:
/// it is the start, naming a workspace nobody has made yet.
#[test]
fn the_founding_box_carries_the_start_s_acts() {
    let harness = crate::snapshot::seat(founding(), 1400.0, 900.0);
    let inventory = crate::snapshot::parity::inventory(&harness);
    for op in [crate::verbs::PREPARE, crate::verbs::PROMPT] {
        assert!(inventory.contains(op), "{op:?} is tagged: {inventory:?}");
    }
}
