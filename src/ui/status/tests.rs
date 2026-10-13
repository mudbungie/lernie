//! The status rule: its state slot, its facts, its name, and what fits.

use super::{ENGINES, LEAD, NO_MODEL, READY, SEP, facts, fitting, name, state, waiting};
use crate::reply::convs::AgentState;
use crate::test_support::window::{painted, seated, wall};
use crate::ui::{Model, Notice, theme};

/// **The state slot is the selected conversation's word and age, in its ink**,
/// `ready` where nothing is selected, and the notice while there is one.
#[test]
fn the_state_slot_follows_the_selection_and_the_notice() {
    let quiet = state(&Model::default());
    assert_eq!(quiet.text, READY);
    assert_eq!(quiet.ink, theme::INK_WEAK);
    let talking = state(&seated());
    assert!(talking.text.contains("quiescent"), "{}", talking.text);
    assert!(talking.text.contains("42s"), "{}", talking.text);
    assert_eq!(talking.ink, theme::state_ink(&AgentState::Quiescent));
    let mut heard = seated();
    heard.notice = Some(Notice::Refused("no".to_owned()));
    let said = state(&heard);
    assert!(said.text.contains("refused"), "{}", said.text);
    assert_eq!(said.ink, theme::accent(theme::State::Error));
    heard.notice = Some(Notice::Said("the act landed".to_owned()));
    let noted = state(&heard);
    assert!(noted.text.contains("the act landed"), "{}", noted.text);
    assert_eq!(noted.ink, theme::accent(theme::State::Annotation));
}

/// **The facts**: how many engines always, how many are waiting only while
/// any is — in the attention accent, the one green on a resting window.
#[test]
fn the_facts_count_the_engines_and_colour_the_waiting() {
    let quiet: Vec<String> = facts(&seated()).into_iter().map(|run| run.text).collect();
    assert_eq!(quiet, vec![NO_MODEL.to_owned(), "1 engine".to_owned()]);
    assert_eq!(waiting(&seated()), 0);
    let mut asked = seated();
    asked.roster[0].walls.push(crate::reply::roster::WsRow {
        attention: 2,
        ..wall("annex")
    });
    assert_eq!(waiting(&asked), 2);
    let green = facts(&asked)
        .into_iter()
        .find(|run| run.text == "2 waiting")
        .expect("the count is a fact");
    assert_eq!(green.ink, theme::accent(theme::State::Attention));
    let none: Vec<String> = facts(&Model::default())
        .into_iter()
        .map(|run| run.text)
        .collect();
    assert_eq!(none, vec!["0 engines".to_owned()]);
}

/// **The right end**: the conversation, else the wall, else the window.
#[test]
fn the_name_is_the_conversation_the_wall_or_the_window() {
    assert_eq!(name(&seated()).text, "port the paint probe");
    let mut starting = seated();
    starting.conversation = None;
    assert_eq!(name(&starting).text, "home");
    assert_eq!(name(&Model::default()).text, ENGINES);
}

/// **Facts drop from the right as the width runs out, the name last**: none
/// fit in no room, all fit in plenty.
#[test]
fn facts_drop_from_the_right_as_the_width_runs_out() {
    let model = seated();
    let window = crate::paint_probe::frame::Window::new();
    let mut shown = (0, 0);
    window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (s, f, n) = (state(&model), facts(&model), name(&model));
            shown = (
                fitting(ui, &s, &f, &n.text, 10.0),
                fitting(ui, &s, &f, &n.text, 10_000.0),
            );
        });
    });
    assert_eq!(shown, (0, facts(&model).len()));
}

/// **The rule reaches the glass whole** on a seated window: the lead, the
/// state, a separator, the engines, the name.
#[test]
fn the_rule_reaches_the_glass() {
    let mut model = seated();
    let text = painted(&mut model);
    for word in [LEAD, SEP, "1 engine", "port the paint probe"] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
}

/// **The spend is the selected conversation's, or nothing** (bl-3b46): the
/// row the records read filed stays in hand after `+ new` clears the
/// selection, and a row that answered another conversation is not this
/// one's — neither is painted on the rule.
#[test]
fn the_spend_belongs_to_the_selection() {
    let own = crate::test_support::window::panes::records::own_row();
    let spend = crate::ui::records::header::costing(&own);
    let texts =
        |model: &Model| -> Vec<String> { facts(model).into_iter().map(|run| run.text).collect() };
    let mut model = seated();
    model.records.agent = Some(own.clone());
    assert!(texts(&model).contains(&spend), "{:?}", texts(&model));
    model.conversation = None;
    assert!(!texts(&model).contains(&spend), "{:?}", texts(&model));
    let mut other = seated();
    other.records.agent = Some(crate::reply::agent::Agent {
        agent: "20260101T000000Z-zzzz".to_owned(),
        ..own
    });
    assert!(!texts(&other).contains(&spend), "{:?}", texts(&other));
}

/// The rule's two doors.
mod doors;

/// **`/plan` stands on the rule while it is on** (bl-6f3f), first among the
/// facts and in the brand — the operator's own arming — and not otherwise.
#[test]
fn the_plan_stands_on_the_rule_while_it_is_on() {
    let mut model = seated();
    model.conversation = None;
    assert!(!facts(&model).iter().any(|run| run.text == super::PLAN));
    model.plan = true;
    let first = facts(&model).into_iter().next().expect("a fact");
    assert_eq!(first.text, super::PLAN);
    assert_eq!(first.ink, theme::BRAND);
    let glass = painted(&mut model);
    assert!(glass.lines().any(|line| line == super::PLAN), "{glass}");
}
