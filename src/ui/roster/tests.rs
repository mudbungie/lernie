//! The roster: what a section says, what a row says, the pin order, the row
//! this seat cannot address, and the click that aims the window.

use super::{NO_NAME_HERE, NO_WALLS, NOT_ANSWERED, header, line, ordered, render};
use crate::paint_probe::frame::Window;
use crate::reply::roster::{WorkspaceKind, WsRow};
use crate::test_support::window::{click, own, pane, seated, wall};
use crate::ui::{Aim, Channel, Chunk, Model};

/// **Every empty section says which emptiness it is** (bl-08b6): nothing has
/// answered down it yet, this box cannot dial it and knows why off its own
/// files, or the engine answered and holds no workspace.
///
/// The pane used to have one sentence, for an empty ROSTER — and an empty
/// roster is unreachable, because every box holds its own engine's slot
/// whether or not anything is provisioned in it. So the box the sentence was
/// written for got a section header over a blank.
#[test]
fn every_empty_section_says_which_emptiness_it_is() {
    for (held, expected) in [
        (crate::ui::Held::Unheard, NOT_ANSWERED.to_owned()),
        (
            crate::ui::Held::Unheld("nothing provisioned at /home/u/wire".to_owned()),
            "nothing provisioned at /home/u/wire".to_owned(),
        ),
        (crate::ui::Held::Heard, NO_WALLS.to_owned()),
    ] {
        let mut model = Model {
            roster: vec![Chunk { held, ..own() }],
            ..Model::default()
        };
        model.roster[0].walls.clear();
        let painted = pane(|ui| render(ui, &mut model));
        assert!(painted.contains(&expected), "{expected:?}:\n{painted}");
    }
}

/// A row states what it is called, how it is classified and its rollups — and
/// the two rollups only when they are non-zero, because a roster of `0 waiting`
/// on every row teaches nothing and costs the one that says `3`.
#[test]
fn a_row_states_its_rollups_only_where_there_is_something_to_say() {
    let quiet = line(&wall("home"));
    assert_eq!(quiet, "home  (named)  2 conversations");
    let busy = line(&WsRow {
        attention: 3,
        running: true,
        ..wall("home")
    });
    assert_eq!(busy, "home  (named)  2 conversations  3 waiting  running");
}

/// **Pinned first, in pin order**, then the rest by name. A rank rather than a
/// flag is what makes this a sort the seat can do without reading the engine's
/// own pin list back.
#[test]
fn pinned_rows_are_hoisted_in_pin_order() {
    let pinned = |name: &str, rank: u64| WsRow {
        pinned: Some(rank),
        ..wall(name)
    };
    let walls = vec![
        wall("zed"),
        pinned("second", 1),
        wall("aardvark"),
        pinned("first", 0),
    ];
    let names: Vec<String> = ordered(&walls)
        .into_iter()
        .map(|row| row.workspace)
        .collect();
    assert_eq!(names, vec!["first", "second", "aardvark", "zed"]);
}

/// The header names the local spelling, and the host's beside it where the two
/// differ — a local rename is the remedy for a name collision, and an operator
/// has to be able to see one.
#[test]
fn a_renamed_channel_says_both_names_and_an_unrenamed_one_says_one() {
    let entry = |there: &str| Chunk {
        channel: Channel {
            name: "home".to_owned(),
            named_there: Some(there.to_owned()),
        },
        ..Chunk::default()
    };
    assert_eq!(
        header(&entry("personal")),
        "home (named \"personal\" on its host)"
    );
    assert_eq!(header(&entry("home")), "home");
    assert_eq!(header(&own()), "(this box's own engine)");
}

/// **A row this seat holds no name for is painted, and painted as unreachable.**
/// Dropping it would hide a workspace the operator has; addressing it by the
/// entry's leaf would aim a gesture at a different wall.
#[test]
fn a_row_the_entry_does_not_name_is_shown_and_said_to_be_unreachable() {
    let mut model = Model {
        roster: vec![Chunk {
            channel: Channel {
                name: "home".to_owned(),
                named_there: Some("personal".to_owned()),
            },
            walls: vec![wall("personal"), wall("somebody-elses")],
            ..Chunk::default()
        }],
        ..Model::default()
    };
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("personal"), "{painted}");
    assert!(painted.contains(NO_NAME_HERE), "{painted}");
}

/// The engine's currency notes ride with the rows they are about.
#[test]
fn a_channel_says_how_current_its_answer_is_when_the_engine_did() {
    let mut model = Model {
        roster: vec![Chunk {
            stale: Some("derivation 4m behind".to_owned()),
            growth: Some("one grew 3 steps".to_owned()),
            ..own()
        }],
        ..Model::default()
    };
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("derivation 4m behind"), "{painted}");
    assert!(painted.contains("one grew 3 steps"), "{painted}");
}

/// **Clicking a row aims the window at it**, and clears everything that was
/// about the wall it left: a conversation list from another workspace standing
/// under a new header is the worst kind of stale, because it reads as current.
#[test]
fn a_click_aims_the_window_and_drops_what_was_about_the_last_wall() {
    let mut model = seated();
    model.aim = None;
    let window = Window::new();
    click(&window, "home  (named)  2 conversations", |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    assert_eq!(
        model.aim,
        Some(Aim {
            channel: "(this box's own engine)".to_owned(),
            address: "home".to_owned(),
        })
    );
    assert!(model.convs.is_empty(), "the last wall's list went with it");
    assert_eq!(model.conversation, None);
    assert!(model.transcript.entries.is_empty());
}

/// A classification this build does not know keeps its word on the glass, which
/// is rung 3 painted rather than merely decoded.
#[test]
fn an_unknown_classification_is_painted_as_its_own_word() {
    let mut model = Model {
        roster: vec![Chunk {
            walls: vec![WsRow {
                kind: WorkspaceKind::Unknown("sealed".to_owned()),
                ..wall("archive")
            }],
            ..own()
        }],
        ..Model::default()
    };
    let painted = pane(|ui| render(ui, &mut model));
    assert!(painted.contains("(sealed)"), "{painted}");
}
