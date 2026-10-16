//! The live dock: what it counts, its one line, and Ctrl+T.

use super::{line, row_words, running};
use crate::paint_probe::frame::Window;
use crate::reply::convs::AgentState;
use crate::test_support::window::{click, conv, painted, seated};
use crate::ui::{Model, theme};

/// A seated model with one conversation at work and one at rest.
fn busy() -> Model {
    let mut model = seated();
    model.convs[0].state = AgentState::InFlight;
    model.convs.push(conv("c-2", "a quiet one"));
    model
}

/// The one chord, pressed on the whole window.
fn ctrl_t(window: &Window, model: &mut Model) {
    window.frame(
        vec![egui::Event::Key {
            key: egui::Key::T,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        }],
        |ctx| crate::ui::render(ctx, model),
    );
}

/// **It counts the conversations at work and no others**, and its line
/// names the first.
#[test]
fn it_counts_what_runs_and_names_the_first() {
    assert!(running(&seated()).is_empty());
    let rows = running(&busy());
    assert_eq!(rows.len(), 1);
    assert_eq!(
        line(&rows, false),
        format!(
            "{} 1 running · port the paint probe · Ctrl+T",
            theme::glyph::SHUT
        )
    );
    assert!(line(&rows, true).starts_with(theme::glyph::OPEN));
    assert_eq!(row_words(&rows[0]), "port the paint probe  in-flight  42s");
    assert_eq!(
        line(&[], false),
        format!("{} 0 running ·  · Ctrl+T", theme::glyph::SHUT)
    );
}

/// **Nothing stands where nothing runs; folded, one line stands; Ctrl+T
/// unfolds it to a row apiece, and a click on a row selects.**
#[test]
fn ctrl_t_unfolds_it_and_a_row_selects() {
    let mut quiet = seated();
    assert!(!painted(&mut quiet).contains("running"));
    let mut model = busy();
    let window = Window::new();
    window.text(|ctx| crate::ui::render(ctx, &mut model));
    let folded = window.text(|ctx| crate::ui::render(ctx, &mut model));
    assert!(folded.contains("1 running"), "{folded}");
    assert!(!folded.contains("in-flight  42s"), "{folded}");
    ctrl_t(&window, &mut model);
    let open = window.text(|ctx| crate::ui::render(ctx, &mut model));
    assert!(open.contains("in-flight  42s"), "{open}");
    model.conversation = Some("c-2".to_owned());
    window.text(|ctx| crate::ui::render(ctx, &mut model));
    click(&window, &row_words(&running(&model)[0]), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.conversation.as_deref(), Some("20260830T051200Z-a1b2"));
    click(&window, &line(&running(&model), true), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    let shut = window.text(|ctx| crate::ui::render(ctx, &mut model));
    assert!(
        !shut.contains("in-flight  42s"),
        "the head folds it:\n{shut}"
    );
}

/// **The `dock` knob is the fold's first frame** (DESIGN §4.46 item 4): a
/// window that has not toggled it stands as the knob says, and Ctrl+T
/// toggles from there.
#[test]
fn the_knob_is_the_fold_until_it_is_toggled() {
    let ctx = egui::Context::default();
    assert!(!super::unfolded(&ctx, false));
    assert!(super::unfolded(&ctx, true), "the knob on");
    super::toggle(&ctx, true);
    assert!(!super::unfolded(&ctx, true), "toggled from the knob");
    assert!(!super::unfolded(&ctx, false), "and the knob no longer read");
}
