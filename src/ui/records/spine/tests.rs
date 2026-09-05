//! What the spine half says, in every state each of its three parts can be in
//! — and the sentences computed beside the paint, read as values.

use super::{
    GOVERNING_HEAD, NO_CARDS, NO_FILES, NO_NOTCHES, NOT_ANSWERED_GOVERNING, NOT_ANSWERED_SPINE,
    ROLE_SAID, SPINE_HEAD, card, forking, headline, render, seated as placed,
};
use crate::paint_probe::frame::Window;
use crate::reply::governing::{Governance, Governing};
use crate::reply::rail::{Notch, Rail};
use crate::test_support::window::{click, notch, painted, pane, recorded, seated};
use crate::ui::{Forking, Model};

/// **The answered half paints the whole spine**: the governing sentence and
/// its tree, both notches, where the chat seats one, and the card off it.
#[test]
fn the_answered_half_paints_the_governing_commit_the_notches_and_the_cards() {
    let mut model = recorded();
    let text = pane(|ui| render(ui, &mut model));
    for word in [
        GOVERNING_HEAD,
        "policy follows config/default, now at bbbbbbbb",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "workflow.yaml",
        SPINE_HEAD,
        ROLE_SAID,
        "001  abcdef1 — 120 tokens",
        "above 003-claude.json — it had read 2 entries",
        "002  — — 120 tokens",
        "fork from abcdef1",
        "Cobalt — from here — live — 9 tokens",
        "working",
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
}

/// **A notch with no commit carries no fork control**, because `fork`'s `from`
/// is a ref and that notch names none.
#[test]
fn only_an_operable_notch_offers_a_fork() {
    let bare = Notch {
        commit: None,
        seat: None,
        ..notch("002")
    };
    assert!(!bare.operable());
    let mut model = Model {
        records: crate::ui::Records {
            rail: Some(Rail {
                notches: vec![bare.clone()],
                cards: Vec::new(),
            }),
            ..recorded().records
        },
        ..recorded()
    };
    let text = pane(|ui| render(ui, &mut model));
    assert!(!text.contains("fork from"), "{text}");
    // The label names the commit it carries, so two notches never offer one
    // word and an operator reads what they are about to fork off.
    assert_eq!(forking(&notch("001")), "fork from abcdef1");
    assert_eq!(forking(&bare), "fork from —");
}

/// **Every empty state is its own sentence.** Nobody answered is not the same
/// claim as a spine with no commit on it, nor as a conversation nobody forked
/// from, nor as a config commit whose tree is empty.
#[test]
fn every_empty_state_is_its_own_sentence() {
    let mut unanswered = Model {
        records: crate::ui::Records {
            rail: None,
            governing: None,
            ..recorded().records
        },
        ..recorded()
    };
    let text = pane(|ui| render(ui, &mut unanswered));
    assert!(text.contains(NOT_ANSWERED_SPINE), "{text}");
    assert!(text.contains(NOT_ANSWERED_GOVERNING), "{text}");

    let mut empty = Model {
        records: crate::ui::Records {
            rail: Some(Rail {
                notches: Vec::new(),
                cards: Vec::new(),
            }),
            governing: Some(Governing {
                oid: "c".to_owned(),
                short_oid: "c".to_owned(),
                governance: Governance::Held { diverged: 2 },
                files: Vec::new(),
            }),
            ..recorded().records
        },
        ..recorded()
    };
    let text = pane(|ui| render(ui, &mut empty));
    for word in [
        NO_NOTCHES,
        NO_CARDS,
        NO_FILES,
        "policy held at c — 2 diverged config lineages",
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
}

/// **The lines are pure functions of the row**, so the suite reads the sentence
/// rather than the layout — and a notch the chat gave no seat says nothing
/// rather than saying it was seated nowhere.
#[test]
fn the_lines_are_values_and_an_unseated_notch_has_none() {
    assert_eq!(headline(&notch("001")), "001  abcdef1 — 120 tokens");
    assert_eq!(
        placed(&notch("001")).as_deref(),
        Some("above 003-claude.json — it had read 2 entries")
    );
    let unseated = Notch {
        seat: None,
        ..notch("003")
    };
    assert_eq!(placed(&unseated), None);
    let rail = recorded()
        .records
        .rail
        .expect("the fixture answers a spine");
    assert_eq!(card(&rail.cards[0]), "Cobalt — from here — live — 9 tokens");
}

/// **The control is disabled until both words are there, and firing it spends
/// the notch it hangs on** — driven through the real glass, so the click lands
/// on the seat the label names.
#[test]
fn the_fork_control_waits_for_the_draft_and_then_carries_its_own_commit() {
    let window = Window::new();
    let mut model = recorded();
    click(&window, &forking(&notch("001")), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(model.outbox.is_empty(), "a blank draft fires nothing");

    model.forking = Forking {
        role: "worker".to_owned(),
        goal: "try it".to_owned(),
    };
    click(&window, &forking(&notch("001")), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.outbox.len(), 1, "the armed control fires once");
    assert_eq!(model.outbox[0].envelope["from"], "abcdef1234567890");
}

/// The half stands down with the pane, exactly as the two above it do.
#[test]
fn a_shut_pane_paints_no_spine() {
    let mut model = Model {
        listing: None,
        ..recorded()
    };
    let text = painted(&mut model);
    assert!(!text.contains(SPINE_HEAD), "{text}");
    let mut nothing = seated();
    assert!(!painted(&mut nothing).contains(GOVERNING_HEAD));
}
