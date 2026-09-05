//! The authoring block: what it says it is about, which boxes it offers, the
//! refusals beside its dark controls, and the five gestures its clicks compose.

use super::{
    ACT, AMEND, ARM_HINT, CLAIM, DELIVER, DONE, FILE, FILE_IT, NEW, NOTE_HINT, PROJECT_HINT,
    RELEASE, UNAMENDABLE, UNARMED, UNFILEABLE,
};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{boarded, click, painted};
use crate::ui::Model;

/// The pane with the block open on the wall's one held ball.
fn amending() -> Model {
    let mut model = boarded();
    let ball = model
        .holding
        .clone()
        .and_then(|rows| rows.first().cloned())
        .expect("the boarded fixture holds one ball");
    model.begin_amending(&ball);
    model
}

/// **The wall section's control opens the block on a new ball**, and the block
/// says what it is about before it offers anything (§4.20).
#[test]
fn the_wall_s_control_opens_a_block_on_a_ball_that_does_not_exist_yet() {
    let mut model = boarded();
    let window = Window::new();
    click(&window, FILE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.authoring.is_some(), "the control opened nothing");
    let glass = painted(&mut model);
    assert!(glass.contains(NEW), "{glass}");
    assert!(glass.contains(PROJECT_HINT), "{glass}");
    // A ball that does not exist has no journal and nothing to arm.
    assert!(!glass.contains(NOTE_HINT), "{glass}");
    assert!(!glass.contains(ARM_HINT), "{glass}");
}

/// **A row's control opens it on that ball**, and the subject names the id,
/// the project its verbs run in and the name they stamp.
#[test]
fn a_held_row_s_control_opens_the_block_on_that_ball() {
    let mut model = boarded();
    let window = Window::new();
    click(&window, ACT, |ctx| crate::ui::render(ctx, &mut model));
    let glass = painted(&mut model);
    assert!(glass.contains("bl-1 in lernie as home"), "{glass}");
    assert!(glass.contains(NOTE_HINT), "{glass}");
    // The project is the ball's, so it is not typed.
    assert!(!glass.contains(PROJECT_HINT), "{glass}");
}

/// **The way out comes first and it changes nothing.**
#[test]
fn the_way_out_puts_the_block_down_and_leaves_the_pane() {
    let mut model = amending();
    let window = Window::new();
    click(&window, DONE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.authoring.is_none());
    assert!(model.boarding(), "the pane went down with the block");
    assert!(model.outbox.is_empty(), "the way out sent something");
}

/// **Escape is that same control reached without a pointer**, and it puts the
/// block down rather than the pane — a thing inside a pane is not the pane.
#[test]
fn escape_puts_the_block_down_before_the_pane() {
    let mut model = amending();
    model.escape();
    assert!(model.authoring.is_none());
    assert!(model.boarding());
    model.escape();
    assert!(!model.boarding());
}

/// **Every dark control says what would make it live** — a greyed control says
/// a thing is not live and nothing about what would make it live (§4.20).
#[test]
fn each_refusal_is_spelled_beside_its_dark_control() {
    let mut model = boarded();
    model.begin_filing();
    let glass = painted(&mut model);
    assert!(
        glass.contains(FILE_IT) && glass.contains(UNFILEABLE),
        "{glass}"
    );

    let mut model = amending();
    let glass = painted(&mut model);
    assert!(
        glass.contains(AMEND) && glass.contains(UNAMENDABLE),
        "{glass}"
    );
    assert!(
        glass.contains(DELIVER) && glass.contains(UNARMED),
        "{glass}"
    );
    // The release is offered whenever there is a ball, so it says nothing.
    assert!(glass.contains(RELEASE), "{glass}");
}

/// **A filled block files a ball down the channel the wall is on**, which is
/// the fan the whole surface was arranged to avoid.
#[test]
fn filing_composes_a_create_down_one_channel() {
    let mut model = boarded();
    model.begin_filing();
    if let Some(block) = model.authoring.as_mut() {
        block.project = "lernie".to_owned();
        block.title = "a title".to_owned();
    }
    let window = Window::new();
    click(&window, FILE_IT, |ctx| crate::ui::render(ctx, &mut model));
    let posted = model.outbox.first().expect("one gesture");
    assert_eq!(posted.envelope["op"], "create");
    assert!(posted.channel.is_some(), "a create must never fan");
}

/// **The three acts on a held ball each compose their own verb**, and the
/// armed one composes only once the box holds the ball's id.
#[test]
fn the_held_ball_s_three_acts_each_compose_their_own_verb() {
    for (word, op, arm) in [
        (AMEND, "update", ""),
        (RELEASE, "release", ""),
        (DELIVER, "close", "bl-1"),
    ] {
        let mut model = amending();
        if let Some(block) = model.authoring.as_mut() {
            block.note = "a note".to_owned();
            block.arm = arm.to_owned();
        }
        let window = Window::new();
        click(&window, word, |ctx| crate::ui::render(ctx, &mut model));
        let posted = model.outbox.first().expect("one gesture");
        assert_eq!(posted.envelope["op"], op, "{word}");
        assert!(posted.channel.is_some(), "{word} must never fan");
    }
}

/// **The claim hangs on the board row and fires down that row's channel.**
#[test]
fn the_board_row_s_claim_composes_an_assign() {
    let mut model = boarded();
    let window = Window::new();
    click(&window, CLAIM, |ctx| crate::ui::render(ctx, &mut model));
    let posted = model.outbox.first().expect("one gesture");
    assert_eq!(posted.envelope["op"], "assign");
    // **The gated row is claimable and the claimed one is not**, which is the
    // row's own claimant rather than the column's word: gated is upstream's
    // *"a ball you could claim but could not deliver"*.
    assert_eq!(posted.envelope["id"], "bl-2");
    assert!(posted.channel.is_some(), "an assign must never fan");
}

/// **A block with nowhere to send is painted nowhere**, which is the honest
/// reading of an aim on a channel this box no longer holds.
#[test]
fn a_block_whose_channel_has_gone_paints_nothing() {
    let mut model = amending();
    model.roster.clear();
    let glass = painted(&mut model);
    assert!(!glass.contains(DELIVER), "{glass}");
}
