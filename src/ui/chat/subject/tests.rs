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

/// The stopped conversation the two beats below read, in the window.
fn showing() -> crate::ui::Model {
    let mut model = seated();
    let mut row = row();
    row.display = "ZucchiniFrost".to_owned();
    row.state = crate::reply::convs::AgentState::Stopped;
    row.refused = false;
    row.spend.tokens.total = 5_089_466;
    model.records.agent = Some(row);
    model
}

/// **On the glass**, which is where the ball was filed from: the word
/// *conversation* was the whole of what the pane said about its subject.
///
/// **The line is read as its parts** since bl-d1ae: the three clauses carry
/// three inks and so are three runs, where [`named`] joins them for a reader
/// outside the window.
#[test]
fn the_pane_says_which_conversation_it_is_showing() {
    let model = showing();
    let painted = pane(|ui| crate::ui::chat::render(ui, &model));
    for clause in ["ZucchiniFrost", "stopped", "5089466 tokens"] {
        assert!(painted.contains(clause), "{painted}");
    }
}

/// **The resting clause wears the state's own accent** (STYLE §2), which is
/// what makes *is this still running* answerable at a glance rather than by
/// reading a sentence. A stopped conversation will not mend itself, so it is
/// said in the error accent.
#[test]
fn the_resting_clause_is_painted_in_the_state_s_accent() {
    let mut model = showing();
    let window = crate::paint_probe::frame::Window::new();
    let runs = crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, &mut model));
    let resting = runs
        .iter()
        .find(|run| run.text == "stopped")
        .expect("the resting clause is on the glass");
    assert_eq!(
        resting.ink,
        crate::ui::theme::accent(crate::ui::theme::State::Error)
    );
    let named = runs
        .iter()
        .find(|run| run.text == "ZucchiniFrost")
        .expect("the name is on the glass");
    assert_eq!(named.ink, crate::ui::theme::INK, "the name is body ink");
    // **And it stands at HEADING size** (bl-f251): the header is the pane's
    // name, so it is taller on the glass than the state word beside it.
    assert!(
        named.laid.height() > resting.laid.height(),
        "{:?} over {:?}",
        named.laid,
        resting.laid
    );
}

/// **Where it hangs is on the header** (bl-f251): a child conversation says
/// what it hangs under, one step weaker, and a root says nothing about it.
#[test]
fn a_child_conversation_says_what_it_hangs_under_and_a_root_does_not() {
    let mut model = showing();
    let window = crate::paint_probe::frame::Window::new();
    let runs = crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, &mut model));
    let under = runs
        .iter()
        .find(|run| run.text == "under 20260830T050000Z-root")
        .expect("the path is on the glass");
    assert_eq!(under.ink, crate::ui::theme::INK_WEAK);
    if let Some(row) = model.records.agent.as_mut() {
        row.ancestors = Vec::new();
    }
    let runs = crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, &mut model));
    assert!(
        !runs.iter().any(|run| run.text.starts_with("under ")),
        "a root hangs under nothing"
    );
}

/// **The model that answered last stands on the line, one step weaker** —
/// where the engine holds no context reading, which is the quiescent case the
/// question is actually asked in.
#[test]
fn the_last_model_stands_on_the_line_in_weak_ink_where_the_engine_holds_no_reading() {
    let mut model = showing();
    model.transcript = answered("house-model-9");
    if let Some(row) = model.records.agent.as_mut() {
        row.context = None;
    }
    let window = crate::paint_probe::frame::Window::new();
    let runs = crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, &mut model));
    let answered_by = runs
        .iter()
        .find(|run| run.text == "house-model-9")
        .expect("the model is on the glass");
    assert_eq!(answered_by.ink, crate::ui::theme::INK_WEAK);
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
    let window = crate::paint_probe::frame::Window::new();
    let said = crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, &mut model))
        .into_iter()
        .find(|run| run.text.contains("no credential for provider row"))
        .expect("the provider's clause is on the glass");
    // **In the error accent, not a note's**: a conversation that died will not
    // mend itself, which is the one state red means.
    assert_eq!(
        said.ink,
        crate::ui::theme::accent(crate::ui::theme::State::Error)
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
