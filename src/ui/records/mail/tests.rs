//! What the mail half says, in each of the states a deposit can be in.

use super::{HEAD, NO_MAIL, NOT_ANSWERED, SAID_NOTHING, UNSTATED, ending, headline, render};
use crate::reply::inbox::{Deposit, Row};
use crate::test_support::window::{deposit, pane, recorded};
use crate::ui::Model;

/// **The answered half paints the deposit whole**: its file, its sender and
/// stamp, how the sending agent ended, and what it says.
#[test]
fn the_answered_half_paints_every_fact_a_deposit_states() {
    let model = recorded();
    let text = pane(|ui| render(ui, &model));
    for word in [
        HEAD,
        "user-001.md — from user at 2026-08-30T05:10Z",
        "ended final-response, at refs/litany/agents/c-1",
        "look at the elision",
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
}

/// **Nobody answered is not an empty inbox**, and an empty inbox is a fact
/// about the conversation.
#[test]
fn the_two_empty_states_are_two_sentences() {
    let waiting = Model {
        records: crate::ui::Records {
            mail: None,
            ..recorded().records
        },
        ..recorded()
    };
    assert!(pane(|ui| render(ui, &waiting)).contains(NOT_ANSWERED));
    let empty = Model {
        records: crate::ui::Records {
            mail: Some(Vec::new()),
            ..recorded().records
        },
        ..recorded()
    };
    assert!(pane(|ui| render(ui, &empty)).contains(NO_MAIL));
}

/// **A hand-edited deposit keeps a header**, each unstated fact said as itself
/// — the engine's own rendering rule, kept so the row stays legible.
#[test]
fn an_unstated_fact_is_said_rather_than_dropped() {
    let bare = Row {
        name: "raw.md".to_owned(),
        raw: "bare".to_owned(),
        deposit: Deposit {
            from: None,
            deposited_at: None,
            epitaph: None,
            terminal_ref: None,
            body: "   ".to_owned(),
        },
    };
    assert_eq!(
        headline(&bare),
        format!("raw.md — from {UNSTATED} at {UNSTATED}")
    );
    assert_eq!(ending(&bare), None, "an ordinary deposit states no ending");
    let model = Model {
        records: crate::ui::Records {
            mail: Some(vec![bare]),
            ..recorded().records
        },
        ..recorded()
    };
    assert!(pane(|ui| render(ui, &model)).contains(SAID_NOTHING));
}

/// An epitaph with no commit beside it is still an ending, said without one.
#[test]
fn an_ending_with_no_commit_is_said_without_it() {
    let mut row = deposit();
    row.deposit.terminal_ref = None;
    assert_eq!(ending(&row).as_deref(), Some("ended final-response"));
}
