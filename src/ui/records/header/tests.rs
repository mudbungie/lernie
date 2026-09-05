//! What the header says about a conversation, in every state each of its lines
//! can be in — read as values beside the paint.

use super::{
    ABSENT, DISPLAY_ONLY, HEAD, NOT_ANSWERED, NOTHING_OFFERED, about, contextual, costing, descent,
    doing, flighted, marked, named, offered, parked, render, resting, seated as seats_of, spent,
};
use crate::reply::agent::Agent;
use crate::test_support::window::{own_row, pane, recorded};
use crate::ui::Model;

/// **The answered header paints the whole row** — the name, the rest, why it
/// is resting there, what is in flight, what is parked, what it cost and how
/// full it is.
#[test]
fn the_answered_header_paints_the_conversation_whole() {
    let model = recorded();
    let text = pane(|ui| render(ui, &model));
    for word in [
        HEAD,
        "port the paint probe — in-flight — the provider refused the latest turn — at aaaaaaa",
        DISPLAY_ONLY,
        "no credential for provider row \"work\"",
        "tools — Bash · 5s",
        "held at Bash (toolu_1) — unconfined",
        "under 20260830T050000Z-root",
        "marked notified, held",
        "pennant waiting",
        "120 tokens — $4.00, over 3 conversations",
        "context 2% — 4000 of 200000 for claude-x",
        "the engine offers stop, stop with its children",
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    let nobody = Model {
        records: crate::ui::Records {
            agent: None,
            ..recorded().records
        },
        ..recorded()
    };
    let waiting = pane(|ui| render(ui, &nobody));
    assert!(waiting.contains(NOT_ANSWERED), "{waiting}");
}

/// **The quiet row says none of it**, and each silence is a line that is not
/// painted rather than a line that says nothing.
#[test]
fn a_quiet_row_paints_none_of_the_optional_lines() {
    let quiet = Agent {
        ancestors: Vec::new(),
        display_only: false,
        refused: false,
        failure: None,
        marks: Vec::new(),
        flight: None,
        held: None,
        seats: Vec::new(),
        strip: None,
        context: None,
        ..own_row()
    };
    assert_eq!(resting(&quiet), "in-flight");
    assert_eq!(descent(&quiet), None);
    assert_eq!(marked(&quiet), None);
    assert_eq!(seats_of(&quiet), None);
    assert_eq!(flighted(&quiet), None);
    assert_eq!(parked(&quiet), None);
    assert_eq!(about(&quiet), None, "a row that states none of the three");
    assert_eq!(doing(&quiet), offered(&quiet), "and none of the two");
    assert_eq!(
        costing(&quiet),
        spent(&quiet.spend),
        "a context nothing measured says nothing beside the spend"
    );
}

/// **A class in flight with no strip beside it is still said** — the two are
/// separate facts upstream, and the strip is the fuller of them.
#[test]
fn a_flight_class_with_no_strip_is_said_on_its_own() {
    let bare = Agent {
        strip: None,
        ..own_row()
    };
    assert_eq!(flighted(&bare).as_deref(), Some("in flight: tools"));
}

/// **A conversation the snapshot does not carry has no tip and no offers**,
/// and both are said as sentences rather than as an empty end of a line.
#[test]
fn an_absent_conversation_says_so_rather_than_trailing_off() {
    let gone = Agent {
        tip: String::new(),
        present: false,
        ..own_row()
    };
    assert_eq!(
        named(&gone),
        "port the paint probe — in-flight — the provider refused the latest turn — at no branch tip"
    );
    assert_eq!(offered(&gone), ABSENT);
    let stuck = Agent {
        offers: Vec::new(),
        ..own_row()
    };
    assert_eq!(offered(&stuck), NOTHING_OFFERED);
    let quiet = Agent {
        offers: vec![crate::reply::agent::Offer::Nudge],
        ..own_row()
    };
    assert_eq!(offered(&quiet), "the engine offers nudge");
}

/// **The money is only said where there is a price table**, and the
/// attribution sentence only where the engine wrote one.
#[test]
fn an_unpriced_figure_says_only_its_counters() {
    let mut figure = own_row().spend;
    figure.usd = None;
    figure.attribution.label = None;
    assert_eq!(spent(&figure), "120 tokens");
    let mut priced = own_row().spend;
    priced.attribution.label = None;
    assert_eq!(spent(&priced), "120 tokens — $4.00");
}

/// The percent is the engine's, painted as it came — including one that has
/// outgrown its window, which upstream deliberately does not clamp.
#[test]
fn the_context_percent_is_the_engines_own_unclamped_figure() {
    let over = crate::reply::agent::Fullness {
        model: "claude-x".to_owned(),
        prompt_tokens: 280_000,
        window: 200_000,
        percent: 140,
    };
    assert_eq!(
        contextual(&over),
        "context 140% — 280000 of 200000 for claude-x"
    );
}
