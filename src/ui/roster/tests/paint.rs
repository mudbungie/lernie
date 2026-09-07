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

use super::super::{NO_NAME_HERE, acts, line, render};
use crate::paint_probe::{fills_of, frame::Window};
use crate::reply::roster::WsRow;
use crate::test_support::window::{click, own, seated, seen, wall};
use crate::ui::theme::{BRAND, INK_FAINT, INK_WEAK, RAISED, RULE, State, accent};
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

/// The ink `waiting on you…` reached the glass in, on this model.
fn queue_ink(model: &mut Model) -> Option<egui::Color32> {
    let window = Window::new();
    seen(&window, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| acts::render(ui, model));
    })
    .into_iter()
    .find(|run| run.text == crate::ui::queue::OPEN)
    .map(|run| run.ink)
}

/// **The one control most worth pressing is the one that is coloured** (§2),
/// and a window with nothing waiting has nothing green on it — the same fact
/// read the other way.
#[test]
fn the_queue_entry_is_green_while_something_waits_and_weak_otherwise() {
    let mut quiet = Model::default();
    assert_eq!(queue_ink(&mut quiet), Some(INK_WEAK));
    let mut asked = Model {
        waiting: vec![crate::ui::Asking {
            channel: own().channel,
            rows: vec![crate::test_support::window::waiting("home", "c-1")],
        }],
        ..Model::default()
    };
    assert_eq!(queue_ink(&mut asked), Some(accent(State::Attention)));
    // **A section that answered and holds no row is not a wait** — the
    // emptiness the queue's own `Vec` exists to keep per channel.
    let mut answered = Model {
        waiting: vec![crate::ui::Asking {
            channel: own().channel,
            rows: Vec::new(),
        }],
        ..Model::default()
    };
    assert_eq!(queue_ink(&mut answered), Some(INK_WEAK));
    // **A wall's own rollup lights it too**, before the queue pane has ever
    // been opened: the roster is answered from the first beat, the queue only
    // while its pane stands.
    let mut rolled = Model {
        roster: vec![crate::ui::Chunk {
            walls: vec![crate::reply::roster::WsRow {
                attention: 2,
                ..wall("home")
            }],
            ..own()
        }],
        ..Model::default()
    };
    assert_eq!(queue_ink(&mut rolled), Some(accent(State::Attention)));
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

/// **The window's acts are a compact strip, and so is the aimed wall's band**
/// (bl-f251, item 4): at the roster's own width the six verbs stand on at
/// most two lines and the wall's eight on at most three, every one of them
/// on the glass whole — where the first pass had six full-width rows.
#[test]
fn the_two_bands_are_compact_strips_at_the_rosters_width() {
    let mut model = seated();
    let window = Window::sized(crate::ui::shell::policy::ROSTER, 600.0);
    let runs = seen(&window, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    let lines = |words: &[&str]| -> usize {
        let mut tops: Vec<f32> = words
            .iter()
            .map(|word| {
                let run = runs
                    .iter()
                    .find(|run| run.text == *word)
                    .unwrap_or_else(|| panic!("{word:?} is on the glass whole"));
                run.laid.min.y
            })
            .collect();
        tops.sort_by(f32::total_cmp);
        tops.dedup_by(|a, b| (*a - *b).abs() < 1.0);
        tops.len()
    };
    let window_acts = [
        acts::REFRESH,
        crate::ui::queue::OPEN,
        crate::ui::commands::OPEN,
        crate::ui::trail::OPEN,
        crate::ui::board::OPEN,
        crate::ui::find::OPEN,
    ];
    assert!(lines(&window_acts) <= 2, "the window's strip");
    let wall_acts = [
        super::super::PIN,
        crate::ui::enroll::OPEN,
        crate::ui::tuning::OPEN,
        crate::ui::login::OPEN,
        crate::ui::clients::OPEN,
        crate::ui::config::OPEN,
        crate::ui::fleet::OPEN,
        crate::ui::unmake::OPEN,
    ];
    assert!(lines(&wall_acts) <= 3, "the wall's strip");
}
