//! The fleet pane: the control that opens it, the five acts and which of them
//! are held down until their box has something in it, and the two emptinesses.

use super::{
    ATTEMPTS, CHANGES, CLOSE, DISARM, DISBAND, FEWER, HEADING, MODEL_HINT, MORE, NOT_ANSWERED,
    NOTHING, OPEN, PROJECT_HINT, RUN, SCAN, WATCH,
};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, fleeting, painted, seated};
use crate::ui::Model;

/// **The control hangs off the aimed wall's row and opens the pane**, which is
/// the only thing that says that button is wired to that door.
#[test]
fn the_wall_s_control_opens_the_pane() {
    let mut model = seated();
    let window = Window::new();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.fleet.is_some(), "the control opened nothing");
    let glass = painted(&mut model);
    assert!(glass.contains(HEADING), "{glass}");
    assert!(!glass.contains(OPEN), "{glass}");
}

/// **The close puts it down.**
#[test]
fn the_close_puts_the_pane_down() {
    let mut model = fleeting();
    let window = Window::new();
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.fleet.is_none());
}

/// **Two of the five controls are held down until their box has something in
/// it** — §4.20's enablement rule, where the parameter is missing rather than
/// the subject, so the control is on the glass and says what it wants.
#[test]
fn the_two_acts_that_carry_a_word_are_dead_until_it_is_typed() {
    let mut model = seated();
    model.begin_fleet();
    let glass = painted(&mut model);
    for said in [RUN, DISBAND, WATCH, DISARM, SCAN, PROJECT_HINT, MODEL_HINT] {
        assert!(glass.contains(said), "{said:?} is on no row: {glass}");
    }
    let window = Window::new();
    click(&window, RUN, |ctx| crate::ui::render(ctx, &mut model));
    click(&window, WATCH, |ctx| crate::ui::render(ctx, &mut model));
    assert!(
        model.outbox.is_empty(),
        "a control with no word to carry fired"
    );
}

/// **The three that need no word fire at once**, each composing the envelope
/// its own op takes against the aimed wall.
#[test]
fn the_three_wordless_acts_compose_their_own_gestures() {
    for (word, op) in [(DISBAND, "disband"), (DISARM, "disarm"), (SCAN, "scan")] {
        let mut model = fleeting();
        let window = Window::new();
        click(&window, word, |ctx| crate::ui::render(ctx, &mut model));
        let posted = model.outbox.first().expect("a gesture");
        assert_eq!(posted.envelope["op"], op);
        assert_eq!(posted.envelope["workspace"], "home");
        assert!(posted.act, "{op} is an act");
    }
}

/// **And the two that need one fire once it is there**, carrying the wall, the
/// word and — for the loop — the cap the operator set.
#[test]
fn the_two_worded_acts_carry_the_words_the_pane_holds() {
    let mut model = fleeting();
    let window = Window::new();
    click(&window, RUN, |ctx| crate::ui::render(ctx, &mut model));
    let posted = model.outbox.first().expect("a gesture").clone();
    assert_eq!(posted.envelope["op"], "fleet");
    assert_eq!(posted.envelope["project"], "lernie");
    assert_eq!(posted.envelope["cap"], 4);
    let mut watching = fleeting();
    click(&window, WATCH, |ctx| crate::ui::render(ctx, &mut watching));
    let armed = watching.outbox.first().expect("a gesture");
    assert_eq!(armed.envelope["op"], "arm");
    assert_eq!(armed.envelope["model"], "claude-haiku-4-5");
}

/// **The cap is raised and lowered on the glass**, because it is a number and
/// a number is not a word an operator types into the verb table.
#[test]
fn the_cap_moves_under_its_own_two_controls_and_never_below_one() {
    let mut model = fleeting();
    let window = Window::new();
    click(&window, MORE, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.fleet.as_ref().expect("open").cap, 5);
    click(&window, FEWER, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.fleet.as_ref().expect("open").cap, 4);
    // **The floor is one, and it is the pane's own.** A cap of zero is a loop
    // that spawns nothing and still reaps, which upstream refuses to spell as
    // a cap at all — `disband` is that.
    let mut floored = seated();
    floored.begin_fleet();
    click(&window, FEWER, |ctx| crate::ui::render(ctx, &mut floored));
    assert_eq!(floored.fleet.as_ref().expect("open").cap, 1);
}

/// **The two emptinesses are two sentences, and each listing has its own** —
/// nobody has answered, and the wall answered and has none.
#[test]
fn each_listing_says_which_emptiness_it_is_in() {
    let mut waiting = seated();
    waiting.begin_fleet();
    let glass = painted(&mut waiting);
    assert!(glass.contains(NOT_ANSWERED), "{glass}");
    assert!(glass.contains(ATTEMPTS), "{glass}");
    assert!(glass.contains(CHANGES), "{glass}");

    let mut empty = Model {
        attempts: Some(Vec::new()),
        work: Some(Vec::new()),
        ..waiting
    };
    let glass = painted(&mut empty);
    assert!(glass.contains(NOTHING), "{glass}");
    assert!(!glass.contains(NOT_ANSWERED), "{glass}");
}

/// **Every sentence the pane can say reaches the glass** — the receipt in its
/// op's name, an attempt's lines, a diff row and both churn shapes.
#[test]
fn every_line_the_pane_paints_reaches_the_glass() {
    let mut model = fleeting();
    let glass = painted(&mut model);
    for said in [
        "fleet: it is standing",
        "ship it",
        "judge-one — candidate B reads cleaner",
        "accepted at ccc",
        "work/bl-3 → main",
        "no such ref: work/bl-2",
        "src/a.rs  +3 −1",
        "assets/x.png  binary",
    ] {
        assert!(glass.contains(said), "{said:?} is on no row: {glass}");
    }
}
