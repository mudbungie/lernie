//! The header over the transcript: the four facts, and the absence.

use super::{NOT_ANSWERED, costing, named};
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript, Usage};
use crate::test_support::window::{pane, seated};
use crate::ui::Model;

/// A transcript whose last MODEL turn was answered by `model`, with a
/// delivered message after it — because the last entry of a real transcript is
/// usually not a model turn, and the walk has to read back past one.
fn answered(model: &str) -> Transcript {
    Transcript {
        entries: vec![
            Entry {
                name: "003-model.json".to_owned(),
                raw: "{}".to_owned(),
                kind: EntryKind::Model {
                    model_id: model.to_owned(),
                    blocks: vec![Block::Text("done".to_owned())],
                    usage: Usage::new(),
                },
            },
            crate::test_support::window::said("op", "and again"),
        ],
    }
}

/// The conversation the ball measured, in the shape the wire hands it over.
fn row() -> crate::reply::agent::Agent {
    crate::test_support::window::own_row()
}

#[test]
fn the_line_names_the_conversation_and_says_how_it_is_resting() {
    let mut row = row();
    row.display = "ZucchiniFrost".to_owned();
    row.state = crate::reply::convs::AgentState::Stopped;
    row.refused = false;
    assert_eq!(
        named(&row, &Transcript::default()),
        "ZucchiniFrost — stopped"
    );
    // **And a provider refusal rides in the resting clause**, which is the
    // fact that tells an operator's own stop apart from one the far end made.
    row.refused = true;
    assert_eq!(
        named(&row, &Transcript::default()),
        "ZucchiniFrost — stopped — the provider refused the latest turn"
    );
}

/// **A quiescent conversation is the one an operator asks the model question
/// about**, and it has no context reading for the engine to answer with — so
/// the transcript's own last turn does.
#[test]
fn a_conversation_with_no_context_reading_says_which_model_answered_last() {
    let mut row = row();
    row.display = "ZucchiniFrost".to_owned();
    row.state = crate::reply::convs::AgentState::Quiescent;
    row.refused = false;
    row.context = None;
    assert_eq!(
        named(&row, &answered("claude-sonnet-5")),
        "ZucchiniFrost — quiescent — claude-sonnet-5"
    );
    // **And it stands down where the engine has a reading of its own**, which
    // the costing line already names — never joined beside it.
    let held = self::row();
    assert!(!named(&held, &answered("claude-sonnet-5")).contains("claude-sonnet-5"));
    // Nothing to say where the transcript holds no model turn at all.
    assert_eq!(
        named(&row, &Transcript::default()),
        "ZucchiniFrost — quiescent"
    );
}

#[test]
fn the_costing_line_carries_the_spend_and_the_model() {
    let mut row = row();
    row.spend.tokens.total = 5_089_466;
    let said = costing(&row);
    assert!(said.contains("5089466 tokens"), "{said}");
    assert!(
        said.contains(&row.context.expect("the fixture is in flight").model),
        "{said}"
    );
}

/// **On the glass**, which is where the ball was filed from: the word
/// *conversation* was the whole of what the pane said about its subject.
#[test]
fn the_pane_says_which_conversation_it_is_showing() {
    let mut model = seated();
    let mut row = row();
    row.display = "ZucchiniFrost".to_owned();
    row.state = crate::reply::convs::AgentState::Stopped;
    row.refused = false;
    row.spend.tokens.total = 5_089_466;
    model.records.agent = Some(row);
    let painted = pane(|ui| crate::ui::chat::render(ui, &model));
    assert!(painted.contains("ZucchiniFrost — stopped"), "{painted}");
    assert!(painted.contains("5089466 tokens"), "{painted}");
}

/// **A conversation that died on a bad model id must not look like one that
/// finished**: the provider's own clause is the only thing that tells them
/// apart, and it is on the header.
#[test]
fn a_failed_conversation_says_so_where_it_is_being_read() {
    let mut model = seated();
    let mut row = row();
    row.failure = Some("no credential for provider row \"work\"".to_owned());
    model.records.agent = Some(row);
    let painted = pane(|ui| crate::ui::chat::render(ui, &model));
    assert!(
        painted.contains("no credential for provider row"),
        "{painted}"
    );
}

/// **Nobody answered yet is not the same claim as a conversation with nothing
/// to say**, which is this seat's rule everywhere else.
#[test]
fn a_conversation_nobody_has_been_answered_about_says_so() {
    let model = Model {
        records: crate::ui::Records::default(),
        ..seated()
    };
    let painted = pane(|ui| crate::ui::chat::render(ui, &model));
    assert!(painted.contains(NOT_ANSWERED), "{painted}");
}
