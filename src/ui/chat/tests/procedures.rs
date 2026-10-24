//! **A procedure run on the glass** (§4.47; bl-1d79): the marker the chat
//! paints above the entry a `reply/rail` card's notch seats.

use super::super::render;
use crate::test_support::window::{pane, said, seated};

/// **A procedure run stands above the entry its notch seats** (§4.47): the
/// titler's one line between the operator's message and what it fired.
#[test]
fn a_procedure_run_is_a_marker_above_the_entry_it_fired_at() {
    use crate::reply::rail::{Card, Notch, Rail, Seat};
    let mut model = seated();
    model.transcript.entries.push(said("child", "done"));
    model.records.rail = Some(Rail {
        notches: vec![Notch {
            seq: "001".to_owned(),
            commit: Some("abcdef1234567890".to_owned()),
            budget: 0,
            cost: None,
            seat: Some(Seat {
                row: "001-child.md".to_owned(),
                cut: 1,
            }),
        }],
        cards: vec![Card {
            agent: "20260830T051200Z-a1b2-t-1".to_owned(),
            name: "Tern".to_owned(),
            fork: "from here".to_owned(),
            state: crate::reply::convs::AgentState::Quiescent,
            tokens: 7,
            tail: Some("Port the probe".to_owned()),
            notch: 0,
            procedure: Some("user_message".to_owned()),
            role: Some("titler".to_owned()),
            landed: Some("title".to_owned()),
        }],
    });
    let painted = pane(|ui| {
        render(ui, &model);
    });
    let marker = "titler \u{b7} user_message \u{b7} landed title: \u{201c}Port the probe\u{201d} \u{b7} 7 tokens";
    let (at_marker, at_done) = (painted.find(marker), painted.find("done"));
    assert!(at_marker.is_some(), "{painted}");
    assert!(
        at_marker < at_done,
        "above the entry it fired at:\n{painted}"
    );
    assert!(
        painted.find("port it") < at_marker,
        "and below the message that fired it"
    );
}
