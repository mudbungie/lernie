//! The bubble a row gets, decided from the turn's shape and never by counting
//! on the glass (DESIGN §4.45 item 7).

use super::{born, delivered, settle};
use crate::ui::chat::{Kind, Row};
use crate::ui::theme::{Bubble, Speaker};

fn prose(who: &str, weight: Speaker) -> Row {
    Row::plain(who.to_owned(), "words".to_owned(), weight)
}

/// **Born by what it is**: the operator's turn on the operator's, a step on
/// the trail's, other prose on the final rung until the turn is read.
#[test]
fn a_row_is_born_on_the_bubble_its_kind_names() {
    assert_eq!(born(Speaker::Operator, Kind::Said), Bubble::Operator);
    assert_eq!(born(Speaker::Model, Kind::Said), Bubble::Final);
    assert_eq!(born(Speaker::Peer, Kind::Said), Bubble::Final);
    assert_eq!(born(Speaker::Model, Kind::Thinking), Bubble::Trail);
    assert_eq!(born(Speaker::Model, Kind::Call), Bubble::Trail);
    assert_eq!(delivered("user", None), Speaker::Operator);
    assert_eq!(
        delivered("child", Some(&"ended".to_owned())),
        Speaker::Ended
    );
    assert_eq!(delivered("judge", None), Speaker::Peer);
}

/// **Prose a trail follows within its turn is intermediate; the turn's last
/// prose is final** — and the next operator's row begins a new turn, so a
/// trail there does not demote the prose before it.
#[test]
fn prose_a_trail_follows_is_intermediate_and_the_last_prose_is_final() {
    let mut rows = vec![
        prose("op", Speaker::Operator),
        prose("model-a", Speaker::Model),
        Row::call("read", "", "tu-1"),
        prose("model-a", Speaker::Model),
        prose("op", Speaker::Operator),
        Row::call("read", "", "tu-2"),
    ];
    settle(&mut rows, false);
    let bubbles: Vec<Bubble> = rows.iter().map(|row| row.bubble).collect();
    assert_eq!(
        bubbles,
        vec![
            Bubble::Operator,
            Bubble::Intermediate,
            Bubble::Trail,
            Bubble::Final,
            Bubble::Operator,
            Bubble::Trail,
        ]
    );
    assert!(rows.iter().all(|row| !row.live), "nothing runs");
}

/// **While the turn runs, its prose is intermediate** — what follows is not
/// known yet — and every row of it is live, where the settled turn before it
/// is neither. A transcript with no operator row is one turn.
#[test]
fn the_running_turn_s_prose_is_intermediate_and_its_rows_are_live() {
    let mut rows = vec![
        prose("op", Speaker::Operator),
        prose("model-a", Speaker::Model),
        prose("op", Speaker::Operator),
        Row::call("read", "", "tu-1"),
        prose("«live»", Speaker::Model),
    ];
    settle(&mut rows, true);
    assert_eq!(rows[1].bubble, Bubble::Final);
    assert!(!rows[1].live);
    assert_eq!(rows[4].bubble, Bubble::Intermediate);
    assert!(rows[2].live && rows[3].live && rows[4].live, "{rows:?}");
    let mut alone = vec![prose("model-a", Speaker::Model)];
    settle(&mut alone, true);
    assert_eq!(alone[0].bubble, Bubble::Intermediate);
    let mut none: Vec<Row> = Vec::new();
    settle(&mut none, true);
    assert!(none.is_empty());
}
