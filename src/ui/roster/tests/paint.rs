//! **What the roster is PAINTED in** (`docs/STYLE.md` §2): the rule a row
//! wears for its state, the raise and the brand on the aimed one, the band
//! entry that goes green while something is waiting, and the faint row no
//! gesture can address.
//!
//! Split from [`super`] on the seam the two questions already have: that file
//! reads the WORDS a row says and this one reads the colour it says them in.
//! A rule at a row's left edge is a fill and not a glyph, so it is legible
//! only through [`crate::paint_probe::fills_of`] — and every assertion here
//! is on a finished frame, never on the string that went in.

use super::super::{NO_NAME_HERE, line, render};
use crate::paint_probe::{fills_of, frame::Window};
use crate::reply::roster::WsRow;
use crate::test_support::window::{click, own, seated, seen, wall};
use crate::ui::theme::{BRAND, INK_FAINT, RAISED, RULE, State, accent};
use crate::ui::{Channel, Chunk, Model};

/// Every fill one idle frame of the pane put on the glass.
fn fills(window: &Window, body: impl FnMut(&egui::Context)) -> Vec<(egui::Rect, egui::Color32)> {
    fills_of(&window.frame(Vec::new(), body))
}

/// The pane in a panel of its own, which is what a fill's rect is measured in.
fn shown(window: &Window, model: &mut Model) -> Vec<(egui::Rect, egui::Color32)> {
    fills(window, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, model));
    })
}

/// Whether a rule of `ink` — [`RULE`] wide and no wider — stands anywhere.
fn ruled(fills: &[(egui::Rect, egui::Color32)], ink: egui::Color32) -> bool {
    fills
        .iter()
        .any(|(rect, fill)| *fill == ink && rect.width() <= RULE + 0.5)
}

/// **The eye lands on green** (§2): a wall that is asking wears the attention
/// rule at its left edge and one that is merely working wears the working
/// one, so a roster of five asking walls is five green rules in a column.
#[test]
fn an_asking_wall_wears_the_attention_rule_and_a_running_one_the_working_rule() {
    let mut model = Model {
        roster: vec![Chunk {
            walls: vec![
                WsRow {
                    attention: 3,
                    ..wall("asking")
                },
                WsRow {
                    running: true,
                    ..wall("busy")
                },
            ],
            ..own()
        }],
        ..Model::default()
    };
    let window = Window::new();
    let painted = shown(&window, &mut model);
    for state in [State::Attention, State::Working] {
        assert!(ruled(&painted, accent(state)), "{state:?} stands as a rule");
    }
}

/// **The aimed row is where the operator IS**, so it is raised and wears the
/// brand — whatever its own state would have said.
#[test]
fn the_aimed_row_is_raised_and_wears_the_brand_rule() {
    let mut model = seated();
    let window = Window::new();
    let painted = shown(&window, &mut model);
    assert!(
        painted.iter().any(|(_, fill)| *fill == RAISED),
        "the aimed row is raised"
    );
    assert!(ruled(&painted, BRAND), "and wears the brand rule");
}

/// **A row no entry names is faint and is not a target**: it is on the glass,
/// because dropping it would hide a workspace the operator has, and a click
/// on it aims nothing, because there is no envelope this seat could write.
#[test]
fn a_row_no_entry_names_is_faint_and_takes_no_click() {
    let mut model = Model {
        roster: vec![Chunk {
            channel: Channel {
                name: "home".to_owned(),
                named_there: Some("personal".to_owned()),
                dials: None,
            },
            walls: vec![wall("somebody-elses")],
            ..Chunk::default()
        }],
        ..Model::default()
    };
    let label = format!("{}  — {NO_NAME_HERE}", line(&wall("somebody-elses")));
    let window = Window::new();
    let ink = seen(&window, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    })
    .into_iter()
    .find(|run| run.text == label)
    .map(|run| run.ink);
    assert_eq!(ink, Some(INK_FAINT), "{label}");
    click(&window, &label, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert_eq!(model.aim, None, "a click on it aims nothing");
}

/// **The state survives the column's own width** (bl-cecd). A row is one run
/// elided at the roster's width, so what stands last is what a narrow roster
/// cuts — and a busy wall at 280 points used to lose `running` to the `…`
/// while the rule at its edge was the only place it was said. Read off the
/// glyphs, because the string that went in is whole whatever the glass shows.
#[test]
fn a_busy_wall_at_280_points_still_says_its_state() {
    let mut model = Model {
        roster: vec![Chunk {
            walls: vec![WsRow {
                attention: 5,
                agents: 6,
                running: true,
                ..wall("home")
            }],
            ..own()
        }],
        ..Model::default()
    };
    let window = Window::sized(800.0, 600.0);
    let glyphs: Vec<String> = seen(&window, |ctx| {
        egui::SidePanel::left("roster")
            .exact_width(280.0)
            .show(ctx, |ui| render(ui, &mut model));
    })
    .into_iter()
    .map(|seen| seen.text)
    .collect();
    let row = glyphs
        .iter()
        .find(|text| text.starts_with("home  "))
        .unwrap_or_else(|| panic!("the wall's row is on the glass: {glyphs:?}"));
    assert!(
        row.contains("5 waiting  running"),
        "the state is cut: {row:?}"
    );
}
