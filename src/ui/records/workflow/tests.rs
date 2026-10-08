//! The workflow mark's three renderings, and what each of its two acts
//! composes.

use super::{CLEAR, marking, post, row};
use crate::paint_probe::frame::Window;
use crate::reply::governing::Mark;
use crate::test_support::window::{click, mark, pane, recorded};
use crate::ui::Model;
use serde_json::json;

/// Paint the row on its own over `model`, with `mark` as the governing read's.
fn painted(model: &mut Model, mark: Option<&Mark>) -> String {
    pane(|ui| row(ui, model, mark))
}

/// Click the seat reading `label` on the row painted over `model`.
fn press(model: &mut Model, label: &str) {
    let held = model
        .records
        .governing
        .clone()
        .and_then(|config| config.workflow_mark);
    let window = Window::new();
    click(&window, label, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| row(ui, model, held.as_ref()));
    });
}

/// **`null` renders as the absence it is**: no mark sentence and no clear —
/// only the lineages a mark could name.
#[test]
fn no_mark_paints_no_sentence_and_offers_no_clear() {
    let mut model = recorded();
    let text = painted(&mut model, None);
    assert!(!text.contains("workflow.yaml"), "{text}");
    assert!(!text.contains(CLEAR), "{text}");
    assert!(text.contains(&marking("strict")), "{text}");
}

/// **A mark on a lineage's head names the lineage**, and offers its clear.
#[test]
fn a_mark_on_a_head_names_its_lineage() {
    let mut model = recorded();
    let text = painted(&mut model, Some(&mark(Some("strict"))));
    for word in [
        "workflow.yaml marked to config/strict at dddddddd, on r-0",
        CLEAR,
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
}

/// **A null lineage with an oid is the pinned older commit**, painted as one
/// and never declined.
#[test]
fn a_mark_its_lineage_moved_past_is_painted_as_the_pinned_commit() {
    let mut model = recorded();
    let text = painted(&mut model, Some(&mark(None)));
    for word in [
        "workflow.yaml pinned at dddddddd, a commit its lineage has moved past, on r-0",
        CLEAR,
    ] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
}

/// **Each act composes the envelope its verb row builds**: a lineage's button
/// carries that lineage's name, and the clear carries none.
#[test]
fn each_act_composes_the_envelope_its_verb_row_builds() {
    for (label, expected) in [
        (
            marking("strict"),
            json!({"op": "workflow", "workspace": "home",
                   "agent": "20260830T051200Z-a1b2", "config": "strict"}),
        ),
        (
            CLEAR.to_owned(),
            json!({"op": "clear-workflow", "workspace": "home",
                   "agent": "20260830T051200Z-a1b2"}),
        ),
    ] {
        let mut model = recorded();
        press(&mut model, &label);
        assert_eq!(
            model.outbox,
            vec![crate::ui::Posted::act(expected)],
            "{label}"
        );
    }
}

/// **With no lineages answered there is nothing to offer**, and nothing is.
#[test]
fn unanswered_lineages_offer_no_mark() {
    let mut model = Model {
        lineages: None,
        ..recorded()
    };
    assert!(!painted(&mut model, None).contains("workflow from"));
}

/// Nothing aimed at or selected posts nothing — the gate is the posting, not
/// the paint.
#[test]
fn nothing_selected_posts_nothing() {
    let mut model = Model {
        conversation: None,
        ..recorded()
    };
    post(&mut model, &crate::verbs::clear_workflow);
    assert!(model.outbox.is_empty());
}
