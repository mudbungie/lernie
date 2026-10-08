//! What the staged proposals say on the config pane: the listing, the whole
//! painted verbatim, and the arming on each verdict.

use super::{
    ARMED, ASKED, CLOSE, GONE, HEADING, NOT_ANSWERED, NOT_ARMED, NOT_WHOLE, NOTHING_STAGED, line,
    render, said,
};
use crate::paint_probe::frame::Window;
use crate::test_support::window::panes::{PROPOSED, WHOLE, proposed};
use crate::test_support::window::{click, configured, pane};
use crate::ui::{Model, Posted, Proposing};
use serde_json::json;

/// Paint the listing alone.
fn painted(model: &mut Model) -> String {
    pane(|ui| render(ui, model))
}

/// Paint the listing on its own and click the control reading `label`.
fn press(model: &mut Model, label: &str) {
    let window = Window::new();
    click(&window, label, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, model));
    });
}

/// The fixture with its id typed back into the arming box.
fn armed() -> Model {
    let mut model = configured();
    *model.verdict_box().expect("the fixture names a row") = PROPOSED.to_owned();
    model
}

/// **The diff is painted verbatim** — every line of the engine's text is a
/// painted line exactly as it was sent, its leading markers and indentation
/// kept and its long line unwrapped. A diff a seat reflowed is a diff nobody
/// can apply.
#[test]
fn the_whole_is_painted_byte_for_byte_and_never_reflowed() {
    let text = painted(&mut configured());
    let lines: Vec<&str> = text.lines().collect();
    for want in WHOLE.lines().filter(|line| !line.is_empty()) {
        assert!(
            lines.contains(&want),
            "{want:?} is not a painted line:\n{text}"
        );
    }
}

/// **The listing's three readings each say their own sentence**: nobody has
/// answered, nothing is staged, and a row — whose standing is the engine's
/// word, including the stale one that moves nothing.
#[test]
fn the_listing_says_what_the_engine_answered_and_nothing_more() {
    let mut model = configured();
    model.proposals = None;
    assert!(painted(&mut model).contains(NOT_ANSWERED));
    model.proposals = Some(crate::reply::proposals::Proposals {
        rows: Vec::new(),
        whole: None,
    });
    assert!(painted(&mut model).contains(NOTHING_STAGED));
    let stale = proposed("other", false);
    model.proposals = Some(crate::reply::proposals::Proposals {
        rows: vec![stale.clone()],
        whole: None,
    });
    let text = painted(&mut model);
    for word in [HEADING, "other", line(&stale).as_str(), GONE] {
        assert!(text.contains(word), "{word:?}:\n{text}");
    }
    assert!(
        line(&stale).starts_with("stale — moves no lineage"),
        "{}",
        line(&stale)
    );
    assert!(line(&proposed(PROPOSED, true)).starts_with("fresh — moves default — 1 file"));
}

/// **Named and not yet answered whole**, the row says it is waiting.
#[test]
fn a_named_row_waits_for_its_whole() {
    let mut model = configured();
    model.proposals.as_mut().expect("fixture").whole = None;
    assert!(painted(&mut model).contains(NOT_WHOLE));
}

/// **Clicking a row names it**, which is the read's depth.
#[test]
fn clicking_a_row_names_it() {
    let mut model = configured();
    model.unname_proposal();
    press(&mut model, PROPOSED);
    assert_eq!(model.proposing(), Some(PROPOSED.to_owned()));
}

/// **The three sentences an arming can be in**, read back as a value.
#[test]
fn the_row_says_which_of_the_three_states_it_is_in() {
    let mut held = Proposing {
        id: PROPOSED.to_owned(),
        ..Proposing::default()
    };
    assert_eq!(said(&held), NOT_ARMED);
    held.typed = PROPOSED.to_owned();
    assert_eq!(said(&held), ARMED);
    held.posted = true;
    assert_eq!(said(&held), ASKED);
}

/// **Each verdict is on the glass unarmed and fires nothing**, and armed it
/// composes its settle — one test per verdict, by the control's own word.
#[test]
fn accept_is_armed_by_the_id_and_only_then_fires() {
    verdict_is_armed("accept");
}

/// The same, for the half that throws a reviewer's work away.
#[test]
fn reject_is_armed_by_the_id_and_only_then_fires() {
    verdict_is_armed("reject");
}

/// One verdict's arming, both ways.
fn verdict_is_armed(verdict: &str) {
    let mut model = configured();
    let text = painted(&mut model);
    assert!(text.lines().any(|line| line == NOT_ARMED), "{text}");
    press(&mut model, verdict);
    assert!(
        model.outbox.is_empty(),
        "{verdict}: a dark control fires nothing"
    );
    let mut model = armed();
    press(&mut model, verdict);
    assert_eq!(
        model.outbox,
        vec![Posted::act(json!({"op": "proposal", "workspace": "home",
            "id": PROPOSED, "verdict": verdict}))]
    );
    assert!(painted(&mut model).lines().any(|line| line == ASKED));
}

/// **The way out is first and settles nothing.**
#[test]
fn the_way_out_is_first_and_composes_nothing() {
    let mut model = armed();
    let text = painted(&mut model);
    let index = |word: &str| text.lines().position(|line| line == word);
    assert!(index(CLOSE) < index("accept"), "{text}");
    assert!(index(CLOSE).is_some(), "{text}");
    press(&mut model, CLOSE);
    assert_eq!(model.named_proposal(), None);
    assert!(model.outbox.is_empty());
}
