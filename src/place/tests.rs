//! The place: the round trip, every way a file can fail to be one, and the
//! growth rule.

use super::{Place, at, read, write};
use crate::test_support::Scratch;
use crate::ui::{Aim, Dragged, Engines};

/// The wall a window was aimed at.
fn aimed() -> Aim {
    Aim {
        channel: "(this box's own engine)".to_owned(),
        address: "home".to_owned(),
    }
}

/// A place aimed at that wall and nothing else remembered.
fn pointed() -> Place {
    Place {
        aim: Some(aimed()),
        engines: Engines::default(),
        dragged: Dragged::default(),
    }
}

/// **The round trip, both ways round.** Aimed at nothing is a place too — an
/// operator who left the roster comes back to it — so a run that wrote no aim
/// reads back as no aim rather than as the last one.
#[test]
fn where_the_window_was_pointed_comes_back_and_so_does_nowhere() {
    let scratch = Scratch::new();
    write(scratch.path(), &pointed()).expect("written");
    assert_eq!(read(scratch.path()), pointed());
    write(scratch.path(), &Place::default()).expect("written");
    assert_eq!(read(scratch.path()), Place::default());
}

/// **The accordion round-trips with the aim** (DESIGN §4.39): which engine is
/// open, what each engine's last opening ranks as, and the wall last aimed
/// under each — so a seat comes back to the arrangement it was left in.
#[test]
fn the_accordion_comes_back_with_the_aim() {
    let scratch = Scratch::new();
    let arranged = Place {
        aim: Some(aimed()),
        engines: Engines {
            open: Some("lab".to_owned()),
            opened: [("lab".to_owned(), 7), ("home".to_owned(), 3)]
                .into_iter()
                .collect(),
            aimed: [("lab".to_owned(), "bench".to_owned())]
                .into_iter()
                .collect(),
        },
        dragged: Dragged::default(),
    };
    write(scratch.path(), &arranged).expect("written");
    assert_eq!(read(scratch.path()), arranged);
}

/// **The edges an operator dragged come back with the aim** (DESIGN §4.39,
/// bl-46e5, bl-b9a3): one width and a row count, in the same file and on the
/// same terms — so a seat comes back laid out the way it was left.
#[test]
fn every_edge_the_operator_dragged_comes_back_beside_the_aim() {
    let scratch = Scratch::new();
    let laid_out = Place {
        dragged: Dragged {
            list: Some(360.5),
            rows: Some(7),
        },
        ..pointed()
    };
    write(scratch.path(), &laid_out).expect("written");
    assert_eq!(read(scratch.path()), laid_out);
}

/// **A file that names no edge is a file with no drag** — which is the absence
/// the width policy already answers, and not a zero it would have to refuse.
#[test]
fn a_file_that_names_no_edge_reads_back_as_no_drag_at_all() {
    let scratch = Scratch::new();
    std::fs::write(
        at(scratch.path()),
        r#"{"aim": {"channel": "(this box's own engine)", "address": "home"}}"#,
    )
    .expect("write");
    assert_eq!(read(scratch.path()), pointed());
}

/// **The root is made, not required.** A first run has no state directory at
/// all, and a window that could not keep its place because nobody had created a
/// directory for it would be a window that never kept one.
#[test]
fn a_state_root_that_is_not_there_yet_is_made() {
    let scratch = Scratch::new();
    let root = scratch.join("never/made/before");
    write(&root, &pointed()).expect("written");
    assert_eq!(read(&root), pointed());
}

/// **Every way this can fail is one answer, and it is the answer a first run
/// gets.** A forgotten selection is a keypress; a startup error is an outage,
/// and per-seat UI state may never become the second.
#[test]
fn nothing_a_file_can_be_wrong_about_refuses() {
    let scratch = Scratch::new();
    assert_eq!(read(scratch.path()), Place::default(), "no file at all");
    for body in [
        "",
        "not json",
        "[]",
        "{}",
        r#"{"aim": null}"#,
        r#"{"aim": {}}"#,
        r#"{"aim": {"channel": "own"}}"#,
        r#"{"aim": {"address": "home"}}"#,
        r#"{"aim": {"channel": 7, "address": "home"}}"#,
        r#"{"open": 7, "opened": [], "aimed": "lab"}"#,
        r#"{"list_width": "wide"}"#,
        r#"{"list_width": null}"#,
        // **The keys the fold retired** (bl-b9a3): a file two builds old
        // names the two widths the window used to have, and this build reads
        // it as a seat that has dragged nothing — an unknown key is ignored,
        // which costs exactly one drag.
        r#"{"roster_width": 360.5, "convs_width": 240.0}"#,
        r#"{"composer_rows": 900}"#,
        r#"{"composer_rows": -1}"#,
        r#"{"composer_rows": 3.5}"#,
    ] {
        std::fs::write(at(scratch.path()), body).expect("write");
        assert_eq!(read(scratch.path()), Place::default(), "{body}");
    }
}

/// **A table drops the row it cannot read and keeps the rest** — rung 3 per
/// row rather than per key, so one entry a newer build wrote costs one entry
/// and never the arrangement.
#[test]
fn a_row_this_build_cannot_read_costs_one_row_and_not_the_table() {
    let scratch = Scratch::new();
    std::fs::write(
        at(scratch.path()),
        r#"{"opened": {"lab": 7, "home": "whenever"},
            "aimed": {"lab": "bench", "home": 3}}"#,
    )
    .expect("write");
    let held = read(scratch.path());
    assert_eq!(
        held.engines.opened,
        [("lab".to_owned(), 7)].into_iter().collect()
    );
    assert_eq!(
        held.engines.aimed,
        [("lab".to_owned(), "bench".to_owned())]
            .into_iter()
            .collect()
    );
}

/// **A key this build does not know is ignored and one it wants is absence** —
/// the reply vocabulary's own rungs 3 and 4, applied to this box's own file. It
/// is what lets the next fact REMOTE §7 names be a key beside these rather than
/// a format, and what lets an older build read a newer build's file without
/// losing the half it does understand.
#[test]
fn an_unknown_key_is_ignored_so_the_next_fact_is_a_key_and_not_a_format() {
    let scratch = Scratch::new();
    std::fs::write(
        at(scratch.path()),
        r#"{"aim": {"channel": "(this box's own engine)", "address": "home",
                    "scrolled_to": 42}, "draft": "not read yet"}"#,
    )
    .expect("write");
    assert_eq!(read(scratch.path()), pointed());
}

/// **The place a model IS**, as a projection: the aim and the accordion are the
/// model's own fields, so there is nothing to keep in step.
#[test]
fn the_place_is_a_projection_of_the_model_and_not_a_stored_copy() {
    let mut model = crate::test_support::window::seated();
    model.engines.open = Some("(this box's own engine)".to_owned());
    let held = Place::of(&model);
    assert_eq!(held.aim, model.aim);
    assert_eq!(held.engines, model.engines);
    assert_eq!(held.dragged, model.dragged);
}

/// **A write answers its refusal rather than swallowing it**: by the time it
/// runs there is no window left to paint one in, and the only alternative is
/// losing the operator's place in silence.
#[test]
fn a_root_that_cannot_be_made_says_so_and_names_itself() {
    let scratch = Scratch::new();
    let blocked = scratch.join("a-file");
    std::fs::write(&blocked, b"not a directory").expect("write");
    let refusal = write(&blocked.join("under"), &Place::default()).expect_err("refused");
    assert!(refusal.contains("under"), "{refusal}");
}
