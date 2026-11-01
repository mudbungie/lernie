//! **What a row's hover sentence says** (bl-d9ce). The glyph and its ink are
//! legend-less, so the help is the legend: the state in words, the failure
//! clause where the row is red, then what the click does.

use super::super::help;
use crate::reply::convs::{AgentState, ConvRow};
use crate::test_support::window::conv;

/// The clause every sentence ends with: the row is a door, and the help says
/// where it goes whatever the state says.
const CLICK: &str = " — click shows this conversation";

/// **One state, one word** — and asking outranks the state, as the glyph's
/// `?` does, because the row that wants a person is the one to say so.
#[test]
fn the_sentence_leads_with_the_state_in_words() {
    for (state, said) in [
        (AgentState::Live, "working"),
        (AgentState::InFlight, "working"),
        (AgentState::Quiescent, "at rest"),
        (AgentState::Stopped, "stopped"),
        (AgentState::Unknown("parked".to_owned()), "parked"),
    ] {
        let row = ConvRow {
            state,
            ..conv("id", "port the probe")
        };
        assert_eq!(help(&row), format!("{said}{CLICK}"));
    }
    let asking = ConvRow {
        state: AgentState::Live,
        attention: 2,
        ..conv("id", "port the probe")
    };
    assert_eq!(help(&asking), format!("asking, 2 waiting{CLICK}"));
}

/// **A state nothing observed says so in words**, the `?` the glyph wears
/// spelled out, before anything else the sentence has to say.
#[test]
fn an_unobserved_state_is_qualified() {
    let row = ConvRow {
        uncertain: true,
        ..conv("id", "port the probe")
    };
    assert_eq!(
        help(&row),
        format!("at rest, as far as anyone has looked{CLICK}")
    );
}

/// **A red row's words are under the pointer too.** The switcher paints the
/// failure clause beneath the row and the bar never had room for it; the
/// sentence carries it verbatim, after the state and before the click.
#[test]
fn a_failure_clause_stands_between_the_state_and_the_click() {
    let row = ConvRow {
        state: AgentState::Stopped,
        uncertain: true,
        failure: Some("no credential for provider row \"work\"".to_owned()),
        ..conv("id", "a refusing conversation")
    };
    assert_eq!(
        help(&row),
        format!(
            "stopped, as far as anyone has looked · no credential for provider row \"work\"{CLICK}"
        )
    );
}
