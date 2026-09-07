//! What the cascade offers, when it is armed, and what it fires — read as
//! values and driven through the real glass.

use super::{ARM, armed, control};
use crate::paint_probe::frame::Window;
use crate::reply::agent::{Agent, Offer};
use crate::test_support::window::{click, own_row, pane, recorded};
use crate::ui::Model;

/// The word the control wears, which is the offers sentence's own.
fn word() -> &'static str {
    crate::ui::records::header::word(Offer::Children)
}

/// **The control is painted where the engine offers the cascade**, with the
/// box that arms it — whose hint is the sentence saying what would.
#[test]
fn the_offered_cascade_paints_its_box_and_its_word() {
    let mut model = recorded();
    let row = own_row();
    let text = pane(|ui| control(ui, &mut model, &row));
    for said in [ARM, word()] {
        assert!(text.contains(said), "{said:?}:\n{text}");
    }
}

/// **A conversation the engine offers no cascade on carries no control at
/// all** — absent, not disabled: what is missing is the subject, not a
/// parameter.
#[test]
fn a_row_with_no_children_offer_carries_no_control() {
    let mut model = recorded();
    let lone = Agent {
        offers: vec![Offer::Nudge, Offer::Stop],
        ..own_row()
    };
    let text = pane(|ui| control(ui, &mut model, &lone));
    assert!(!text.contains(word()), "{text}");
    assert!(!text.contains(ARM), "{text}");
}

/// **The arming is the display name back**, surrounding whitespace forgiven
/// and nothing else — the engine's own rule for the arming `delete-agent`
/// takes.
#[test]
fn the_arming_is_the_name_back_with_whitespace_forgiven() {
    let row = own_row();
    assert!(!armed(&row, ""));
    assert!(!armed(&row, "port the paint prob"));
    assert!(armed(&row, "port the paint probe"));
    assert!(armed(&row, "  port the paint probe \n"));
}

/// **The whole pane paints it beside the offers line**, so the sentence and
/// the control that answers it are read in one glance — and the click through
/// the real glass fires the cascade, not the bare stop.
#[test]
fn the_armed_control_fires_the_stop_that_takes_the_children() {
    let window = Window::new();
    let mut model = recorded();
    click(&window, word(), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(model.outbox.is_empty(), "an unarmed control fires nothing");

    model.cascade = own_row().display;
    click(&window, word(), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(model.outbox.len(), 1, "the armed control fires once");
    let envelope = &model.outbox[0].envelope;
    assert_eq!(envelope["op"], crate::verbs::STOP.word);
    assert_eq!(envelope["agent"], "20260830T051200Z-a1b2");
    assert_eq!(
        envelope["children"], true,
        "the cascade, and not the bare form the composer fires"
    );
    // **The arming is never spent**, so the engine's refusal costs no retype.
    assert_eq!(model.cascade, own_row().display);
}

/// **The arming goes with the records it armed**: a name typed for one
/// conversation must not leave the next one's control looking armed.
#[test]
fn moving_the_subject_takes_the_arming_with_it() {
    let mut model = Model {
        cascade: own_row().display,
        ..recorded()
    };
    model.select("another");
    assert_eq!(model.cascade, "");
}

/// **A cascade with no wall and one with no conversation compose nothing** —
/// the two states the pane cannot paint this control in, made unreachable
/// rather than merely unlikely (`crate::ui::model::records`).
#[test]
fn a_cascade_with_no_subject_composes_nothing() {
    let mut nowhere = Model {
        cascade: own_row().display,
        aim: None,
        ..recorded()
    };
    nowhere.post_cascade();
    assert!(nowhere.outbox.is_empty(), "nothing is aimed at");

    let mut unselected = Model {
        cascade: own_row().display,
        conversation: None,
        ..recorded()
    };
    unselected.post_cascade();
    assert!(unselected.outbox.is_empty(), "nothing is selected");
}

/// **A row nobody has answered about arms nothing**, which is the third gate:
/// the arming is compared against the engine's own row, and there is none.
#[test]
fn an_unanswered_row_arms_nothing() {
    let mut model = Model {
        cascade: own_row().display,
        records: crate::ui::Records {
            agent: None,
            ..recorded().records
        },
        ..recorded()
    };
    model.post_cascade();
    assert!(model.outbox.is_empty());
}
