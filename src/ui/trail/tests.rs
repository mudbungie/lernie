//! The trail pane: the three emptinesses, every sentence a row can carry, and
//! the control that opens it.

use super::{CLOSE, HEADING, NOT_ANSWERED, NOTHING, OPEN, headline, output, provenance, standing};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, own, painted, seated, trailed, trailing};
use crate::ui::{Model, Trail};

/// **The control hangs off the roster and opens the pane**, which is the only
/// thing that says that button is wired to that door. It is offered from an
/// unaimed seat, because the pane takes no subject.
#[test]
fn the_roster_s_control_opens_the_pane() {
    let mut model = Model::default();
    let window = Window::new();
    click(&window, OPEN, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.trailing(), "the control opened nothing");
    let glass = painted(&mut model);
    assert!(glass.contains(HEADING), "{glass}");
    // And the strip stands down while a pane covers the conversation.
    assert!(!glass.contains(OPEN), "{glass}");
}

/// **The three emptinesses are three sentences.** Nobody has answered, the
/// engines answered and hold nothing, and rows — the conversation list's own
/// doctrine, on a pane whose subject is every channel.
#[test]
fn each_emptiness_says_which_one_it_is() {
    let mut unheard = Model {
        lookup: Some(crate::ui::Lookup::Trailing),
        ..seated()
    };
    let glass = painted(&mut unheard);
    assert!(glass.contains(NOT_ANSWERED), "{glass}");

    let mut quiet = Model {
        trails: vec![Trail {
            channel: own().channel,
            rows: Vec::new(),
        }],
        ..unheard.clone()
    };
    let glass = painted(&mut quiet);
    assert!(glass.contains(NOTHING), "{glass}");
    assert!(!glass.contains(NOT_ANSWERED), "{glass}");

    let mut answered = trailing();
    let glass = painted(&mut answered);
    assert!(glass.contains("bl close bl-1"), "{glass}");
    assert!(!glass.contains(NOTHING), "{glass}");
}

/// **A section that answered nothing is silent, not a header over a blank** —
/// the queue's rule, on the pane that shares its shape.
#[test]
fn a_channel_holding_nothing_gets_no_header() {
    let mut model = trailing();
    let quiet = crate::ui::Channel {
        name: "elsewhere".to_owned(),
        ..own().channel
    };
    model.trails.push(Trail {
        channel: quiet.clone(),
        rows: Vec::new(),
    });
    let glass = painted(&mut model);
    assert!(!glass.contains("elsewhere"), "{glass}");
}

/// **The engine's own words reach the glass** — the label and the standing,
/// never a classification this seat made (REMOTE §9.17).
#[test]
fn the_row_paints_the_engine_s_own_reading_of_how_it_ended() {
    let mut model = trailing();
    let glass = painted(&mut model);
    for said in [
        "bl close bl-1",
        "exit 1",
        "live",
        "the gate said no",
        "detached — handed off, no exit to observe",
        "quarantined",
    ] {
        assert!(
            glass.contains(said),
            "{said:?} is not on the glass:\n{glass}"
        );
    }
}

/// **A clean run wears no badge**, because a badge on every ordinary run would
/// bury the rows this pane exists for — and every other standing wears its own
/// word, including one this build has never seen.
#[test]
fn a_clean_row_says_nothing_and_every_other_standing_says_itself() {
    assert_eq!(
        standing(&trailed("bl list", crate::reply::ops::CLEAN)),
        None
    );
    assert_eq!(
        standing(&trailed("bl close x", "live")),
        Some("live".to_owned())
    );
    assert_eq!(
        standing(&trailed("bz login", "quarantined")),
        Some("quarantined".to_owned())
    );
}

/// The headline is the command and the engine's label, and the exit integer
/// rides in the detail — carried because it is the next thing an operator asks
/// for, never because the pane reads it.
#[test]
fn the_headline_is_the_command_and_the_label_and_the_detail_carries_the_rest() {
    let row = trailed("bl close x", "live");
    assert_eq!(headline(&row), "bl close x  [exit 1]");
    let said = provenance(&row);
    for part in ["1700", "balls", "/ws/home", "exit 1"] {
        assert!(said.contains(part), "{said}");
    }
}

/// **What it said is one line, the complaint first**, and a child that printed
/// nothing says nothing rather than an empty row.
#[test]
fn a_row_that_printed_nothing_gets_no_line_and_one_that_did_leads_with_why() {
    assert_eq!(output(&trailed("bl list", "clean")), None);
    let both = crate::reply::ops::OpRow {
        stdout: "listed".to_owned(),
        stderr: "the gate said no".to_owned(),
        ..trailed("bl close x", "live")
    };
    assert_eq!(
        output(&both),
        Some("the gate said no\nlisted".to_owned()),
        "the complaint leads"
    );
}

/// The pane's close is on the glass and wired to the door beside it.
#[test]
fn the_pane_s_close_stands_it_down() {
    let mut model = trailing();
    let window = Window::new();
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.trailing());
    // The rows outlive it — the next open is about the same trail.
    assert_eq!(model.trails.len(), 1);
}
