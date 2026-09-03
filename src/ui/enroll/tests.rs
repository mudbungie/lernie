//! **The enrollment pane, before the engine answers**: the control that opens
//! it, the form, and the gesture it composes.
//!
//! Split from [`shown`] on the seam the pane itself has — the form and the
//! picture are never on the glass together, and they fail for different
//! reasons: one is about what an operator can say, the other about what is done
//! with a secret.
//!
//! **Nothing here asserts a pixel.** A QR symbol drawn at the wrong scale
//! carries the same bytes and a symbol with one wrong module does not, so the
//! thing that is right or wrong is the matrix — which `crate::qr`'s own suite
//! pins against an independent encoder — and what is left for this pane is
//! *where* a module goes and *what words* stand around it.

use fixtures::{file, opened};

use super::{CLOSE, HEADING, KEPT, MINTING, NAME_HINT, OPEN, SEND};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, painted, seated};
use crate::ui::{Enrolling, Grade, Model};

/// **The control hangs off the aimed row and off no other** — an enrollment
/// mints the pair `(client, workspace)`, and the workspace is exactly what an
/// aim is.
#[test]
fn the_roster_offers_the_control_only_on_the_aimed_wall() {
    let mut aimed = seated();
    assert!(painted(&mut aimed).contains(OPEN));
    let mut adrift = Model {
        aim: None,
        ..seated()
    };
    assert!(
        !painted(&mut adrift).contains(OPEN),
        "offered with nothing aimed at"
    );
}

/// **The control on the glass opens the pane**, which is the only thing that
/// says that button is wired to that door.
#[test]
fn the_roster_s_control_opens_the_pane() {
    let mut model = seated();
    let window = Window::new();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.enroll.is_some(), "the control opened nothing");
    // And it stops being offered while one is open — a second would replace the
    // first, name, grade and material together.
    assert!(!painted(&mut model).contains(OPEN));
}

/// Opening it needs an aim, and does nothing without one — the same gate the
/// control paints, read from the model's side.
#[test]
fn an_enrollment_cannot_be_opened_with_nothing_aimed_at() {
    let mut adrift = Model::default();
    adrift.begin_enrollment();
    assert!(adrift.enroll.is_none());
}

/// **The form asks for the two things only an operator can supply**, and names
/// the wall it is enrolling into so the act is never aimed by memory.
#[test]
fn the_form_asks_for_a_name_and_a_grade_and_names_the_wall() {
    let mut model = opened();
    let glass = painted(&mut model);
    for word in [HEADING, NAME_HINT, SEND, CLOSE, "home"] {
        assert!(
            glass.contains(word),
            "{word:?} is not on the glass:\n{glass}"
        );
    }
    for grade in Grade::both() {
        assert!(glass.contains(&grade.word()), "{grade:?} has no control");
    }
}

/// **The pane covers the conversation while it is open**, which is the one
/// place this window covers anything: what it holds is a private key on a
/// screen, and a conversation legible behind it would invite the one thing the
/// material must not have.
#[test]
fn the_pane_stands_where_the_conversation_would() {
    let mut chatting = seated();
    let before = painted(&mut chatting);
    assert!(
        before.contains("port it"),
        "the conversation is painted:\n{before}"
    );
    let mut model = opened();
    let during = painted(&mut model);
    assert!(
        !during.contains("port it"),
        "the conversation is still legible:\n{during}"
    );
}

/// A name is the one thing the operator has to supply, so an empty one composes
/// nothing — spending a round trip to learn what this end already knows.
#[test]
fn an_unnamed_enrollment_composes_no_gesture() {
    let mut model = opened();
    model.post_enrollment();
    assert!(model.outbox.is_empty());
    if let Some(enrolling) = model.enroll.as_mut() {
        enrolling.name = "  ".to_owned();
    }
    model.post_enrollment();
    assert!(model.outbox.is_empty(), "whitespace is not a name");
}

/// **The gesture is the verb table's own row**, trimmed and addressed at the
/// aimed wall — so a click and `lernie enroll` compose one object.
#[test]
fn the_composed_gesture_is_the_row_the_command_line_spends() {
    let mut model = opened();
    if let Some(enrolling) = model.enroll.as_mut() {
        enrolling.name = "  phone-1 ".to_owned();
        enrolling.grade = Grade::Foot;
    }
    model.post_enrollment();
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(crate::verbs::enroll(
            "home".to_owned(),
            "phone-1".to_owned(),
            "foot".to_owned()
        ))]
    );
}

/// The default grade is `operator`, because the box an operator enrols first is
/// their own next seat.
#[test]
fn the_default_grade_is_the_one_an_operator_enrolls_first() {
    assert_eq!(Grade::default().word(), "operator");
    assert_eq!(Grade::Foot.word(), "foot");
}

/// **A mint in flight is said, and it survives the drain.** The gesture a frame
/// composes is taken by the poster within the beat, so a pane that read the
/// outbox back would say "minting" for exactly one frame and then say nothing —
/// and an operator with no answer to *has this been asked* clicks again, which
/// is a second box.
#[test]
fn a_mint_in_flight_is_said_and_outlives_the_outbox() {
    let mut model = opened();
    if let Some(enrolling) = model.enroll.as_mut() {
        enrolling.name = "phone-1".to_owned();
    }
    assert!(
        !painted(&mut model).contains(MINTING),
        "said before it was asked"
    );
    model.post_enrollment();
    model.outbox.clear();
    assert!(model.enroll.as_ref().is_some_and(Enrolling::minting));
    let glass = painted(&mut model);
    assert!(glass.contains(MINTING), "{glass}");
    // And the answer ends it.
    file(&mut model);
    assert!(!model.enroll.as_ref().is_some_and(Enrolling::minting));
    assert!(!painted(&mut model).contains(MINTING));
}

/// **Every control on the pane works from the glass**, which is the only thing
/// that says the buttons are wired to the doors beside them: a door called by
/// nothing is a door that compiles.
#[test]
fn the_pane_s_controls_fire_from_the_glass() {
    let mut model = opened();
    if let Some(enrolling) = model.enroll.as_mut() {
        enrolling.name = "phone-1".to_owned();
    }
    let window = Window::new();
    // The grade control, then the mint.
    click(&window, &Grade::Foot.word(), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.enroll.as_ref().map(|e| e.grade), Some(Grade::Foot));
    click(&window, SEND, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.outbox.len(), 1, "the mint composed nothing");
    assert!(model.enroll.as_ref().is_some_and(Enrolling::minting));
    // A second click cannot mint a second box, because the control is closed.
    model.post_enrollment();
    assert_eq!(model.outbox.len(), 1, "a second click minted again");
    assert!(painted(&mut model).contains(MINTING));
}

/// The form's own close forgets an enrollment that never got an answer.
#[test]
fn the_form_s_close_drops_an_unanswered_enrollment() {
    let mut model = opened();
    let window = Window::new();
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.enroll, None);
}

/// The fixtures both halves share.
mod fixtures;
/// What makes it a modal: Escape closes it, and nothing live paints under it.
mod modal;
/// The half after the engine answers: the picture, the forgetting, the geometry.
mod shown;
