//! **The notice bar**: it stands where content would have been and says
//! whose it is, it wraps so its remedy reaches the glass, it wears its
//! state's ink, and it can be put down.
//!
//! Split from [`super`] at the line cap on the seam that file's own doc
//! draws: the layout is one subject and the bar that stands over it is
//! another.

use super::super::{DISMISS, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, painted, seated, seen};
use crate::ui::{Model, Notice};

/// **A refusal and an unreadable answer are both visible, and they read
/// differently** — one is the engine's sentence, the other is a statement about
/// this seat, and only the second is fixed by an upgrade. Neither is a silent
/// drop, which is the reply vocabulary's own policy on the glass.
#[test]
fn a_notice_stands_where_the_content_would_have_been_and_says_whose_it_is() {
    for (notice, expected) in [
        (
            Notice::Refused("unknown workspace \"hoem\"".to_owned()),
            "the engine refused: unknown workspace \"hoem\"",
        ),
        (
            Notice::Unreadable("cannot paint a \"board\" answer".to_owned()),
            "this seat could not read the answer: cannot paint a \"board\" answer",
        ),
    ] {
        let mut model = Model {
            notice: Some(notice),
            ..seated()
        };
        let shown = painted(&mut model);
        assert!(shown.contains(expected), "{expected:?}:\n{shown}");
        assert!(
            shown.contains("home  (named)  2 conversations"),
            "a refusal about one pane does not stop the others:\n{shown}"
        );
    }
}

/// **It is a bar, not a modal**, and it can be put down: an operator who has
/// read a refusal should not have to wait for the next answer to clear it.
#[test]
fn a_notice_can_be_put_down() {
    let mut model = Model {
        notice: Some(Notice::Refused("no".to_owned())),
        ..seated()
    };
    let window = Window::new();
    click(&window, DISMISS, |ctx| render(ctx, &mut model));
    assert_eq!(model.notice, None);
}

/// **The notice wraps rather than being cut at the frame** (bl-3d0f). A
/// horizontal layout lays its label on one line however long it is, and the
/// panel cut it at the window's right edge with no ellipsis to say so — and
/// every refusal this seat paints puts the fact first and the remedy last, so
/// the half that was lost was always the half that says what to do.
///
/// The subject is the first run of a seat on an unprovisioned box: the notice
/// is the only thing on the window carrying an instruction.
#[test]
fn a_long_refusal_wraps_and_its_remedy_reaches_the_glass() {
    let said = format!(
        "no wire provisioned at /home/u/.local/share/lernie/wire: {}",
        crate::channel::material::Whose::Own.remedy()
    );
    let mut model = Model {
        notice: Some(Notice::Unreachable(said.clone())),
        ..seated()
    };
    let window = Window::sized(900.0, 600.0);
    window.text(|ctx| render(ctx, &mut model));
    let bar = seen(&window, |ctx| render(ctx, &mut model))
        .into_iter()
        .find(|run| run.text.starts_with("this seat could not reach it"))
        .expect("the bar is on the glass");
    // **The rects are what testify here, not the glyphs.** A galley's rows
    // carry no newline where the WRAP broke them, so a wrapped run and a run
    // laid past the frame read back as the same string — which is the paint
    // probe's own division of labour: geometry is unaffected, it is the text
    // that lies.
    assert!(
        bar.laid.width() <= 900.0,
        "the run was laid inside the window rather than past it: {:?}",
        bar.laid
    );
    assert!(
        bar.shown.width() >= bar.laid.width() - 0.5,
        "and nothing was clipped off its end: laid {:?}, shown {:?}",
        bar.laid,
        bar.shown
    );
    assert!(
        bar.text.ends_with("the seat mints nothing"),
        "so the remedy's last words are on the glass: {:?}",
        bar.text
    );
}

/// **A notice is said in its state's ink** (`docs/STYLE.md` §5): a failure in
/// the error accent, a receipt in the annotation accent, and neither in a box.
#[test]
fn a_notice_wears_its_state_s_ink() {
    use crate::ui::theme::{State, accent};
    for (notice, ink) in [
        (Notice::Refused("no".to_owned()), accent(State::Error)),
        (
            Notice::Said("the floor stands".to_owned()),
            accent(State::Annotation),
        ),
    ] {
        let mut model = Model {
            notice: Some(notice.clone()),
            ..seated()
        };
        let window = Window::new();
        let run = seen(&window, |ctx| render(ctx, &mut model))
            .into_iter()
            .find(|run| run.text == notice.line())
            .expect("the notice is on the glass");
        assert_eq!(run.ink, ink, "{notice:?}");
    }
}
