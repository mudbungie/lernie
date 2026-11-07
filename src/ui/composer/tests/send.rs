//! **The send door** (DESIGN §4.48 item 4): the pointer's spelling of the
//! Enter the field reads, in each mode of the box — and absent on a slash
//! line, where the menu's rows are the pointer's doors.

use super::super::render;
use crate::paint_probe::frame::Window;
use crate::test_support::window::seated;
use crate::ui::{Model, theme};

/// The door's glyph on the glass, where it stands: the prompt's glyph in the
/// ground's ink, on the round seat.
fn door(window: &Window, body: &mut dyn FnMut(&egui::Context)) -> Option<egui::Pos2> {
    let output = window.frame(Vec::new(), body);
    crate::paint_probe::seen_of(&output)
        .into_iter()
        .find(|run| run.text == theme::glyph::PROMPT && run.ink == theme::GROUND)
        .map(|run| run.laid.center())
}

/// **Click the door on a box holding `draft`**, answering the model after.
fn sent(mut model: Model, draft: &str) -> Model {
    model.draft = draft.to_owned();
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    };
    let at = door(&window, &mut body).expect("the send door stands");
    crate::paint_probe::frame::click(&window, at, &mut body);
    window.frame(Vec::new(), &mut body);
    model
}

/// **In the deposit it sends what Enter sends**; in the start it stages
/// what Enter stages; in the founding it founds — each clearing the box.
#[test]
fn the_send_door_fires_what_enter_fires_in_every_mode() {
    let deposit = sent(seated(), "ship it");
    assert_eq!(deposit.outbox.len(), 1, "{:?}", deposit.outbox);
    assert!(format!("{:?}", deposit.outbox).contains("ship it"));
    assert_eq!(deposit.draft, "");
    let fresh = Model {
        conversation: None,
        ..seated()
    };
    let start = sent(fresh.clone(), "port the probe");
    assert!(start.start.is_some(), "staged");
    let mut founding = fresh;
    founding.begin_founding(crate::seat::OWN);
    let found = sent(founding, "lab port the probe");
    assert!(found.start.is_some(), "founded and staged");
}

/// **An empty box's door sends nothing**, and a slash line has no door.
#[test]
fn an_empty_box_sends_nothing_and_a_slash_line_has_no_door() {
    let quiet = sent(seated(), "");
    assert!(quiet.outbox.is_empty(), "{:?}", quiet.outbox);
    let mut model = seated();
    model.draft = "/he".to_owned();
    let window = Window::new();
    let mut body = |ctx: &egui::Context| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    };
    assert_eq!(door(&window, &mut body), None);
}

/// **Every control stays inside the box however narrow it is**: beside the
/// field at the desk; at a width too narrow for that, on a line of their own
/// under it, the next starting a line where one would run past the edge.
#[test]
fn the_trailing_end_stays_inside_the_box_at_every_width() {
    for wide in [900.0, 300.0, 130.0] {
        let mut model = crate::test_support::window::selecting();
        let window = Window::sized(1000.0, 400.0);
        let mut body = |ctx: &egui::Context| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.allocate_ui(egui::vec2(wide, 300.0), |ui| render(ui, &mut model));
            });
        };
        window.frame(Vec::new(), &mut body);
        let output = window.frame(Vec::new(), &mut body);
        let frame = crate::paint_probe::strokes_of(&output)
            .into_iter()
            .map(|(rect, _)| rect)
            .max_by(|a, b| a.area().total_cmp(&b.area()))
            .expect("the box's hairline");
        let seen = crate::paint_probe::seen_of(&output);
        let field = seen
            .iter()
            .find(|run| run.text.starts_with("say it"))
            .expect("hint");
        for run in &seen {
            assert!(
                frame.contains_rect(run.laid),
                "{wide}: {:?} outside {frame:?}",
                run.text
            );
        }
        let door = seen
            .iter()
            .find(|run| run.ink == theme::GROUND)
            .expect("the door");
        let beside = (door.laid.center().y - field.laid.center().y).abs() < theme::ROW;
        assert_eq!(beside, wide > 500.0, "{wide}: {door:?} vs {field:?}");
        let pills: Vec<f32> = seen
            .iter()
            .filter(|run| run.text.ends_with(theme::glyph::DROP))
            .map(|run| run.laid.min.y)
            .collect();
        assert_eq!(pills.len(), 2, "{wide}: {seen:?}");
        assert_eq!(pills[0] < pills[1], wide < 200.0, "{wide}: {pills:?}");
    }
}
