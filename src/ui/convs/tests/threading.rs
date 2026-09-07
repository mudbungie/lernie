//! **A threaded row on the glass** (`docs/STYLE.md` §5): the rule that says
//! what a conversation is doing, the tint that says which one is selected, and
//! the connectors that say what hangs from what.
//!
//! The rules and the tint are fills, so they are read back through the one
//! paint walk like everything else. **The connectors are line segments and the
//! walk carries no lines**, deliberately: adding a second traversal to see them
//! is the one thing `rules/no-hand-rolled-paint-walk.yml` exists to stop. So a
//! connector is asserted where it is decidable without one — the pure
//! [`super::super::continues`] against a truth table, which is the whole of
//! *does a rail belong here*, and the row's own words at the indent the elbow
//! is drawn to, which is the whole of *where does it point*.

use super::super::{continues, render, row::STEP};
use crate::paint_probe::frame::Window;
use crate::reply::convs::{AgentState, ConvRow};
use crate::test_support::window::{conv, seated, seen};
use crate::ui::Model;
use crate::ui::theme::{BRAND, RAISED, RULE, State, accent};

/// Every fill one frame of the list put on the glass.
fn fills(model: &mut Model) -> Vec<(egui::Rect, egui::Color32)> {
    let window = Window::sized(600.0, 400.0);
    crate::paint_probe::fills_of(&window.frame(Vec::new(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, model));
    }))
}

/// Whether a state's rule — [`RULE`] wide and no wider — stands on the glass.
fn ruled(fills: &[(egui::Rect, egui::Color32)], ink: egui::Color32) -> bool {
    fills
        .iter()
        .any(|(rect, painted)| *painted == ink && rect.width() <= RULE + 0.5)
}

/// A list of one row, with `seated`'s selection pointed somewhere else so the
/// brand does not stand in for the state under test.
fn listing(row: ConvRow) -> Model {
    Model {
        convs: vec![row],
        conversation: None,
        ..seated()
    }
}

/// **The eye lands on green** (§2): a conversation asking for the operator
/// wears the attention accent as its rule, and it wears it *instead of* the
/// state's — a row asking while its driver is stopped is a row to answer, not
/// a row to mourn, and two rules on one edge is no rule at all.
#[test]
fn a_row_that_is_asking_wears_the_attention_rule_over_its_state_s() {
    let mut asking = listing(ConvRow {
        attention: 2,
        state: AgentState::Stopped,
        ..conv("id", "asking")
    });
    let painted = fills(&mut asking);
    assert!(
        ruled(&painted, accent(State::Attention)),
        "the attention rule is at the row's edge"
    );
    assert!(
        !ruled(&painted, accent(State::Error)),
        "and the state it outranks is not"
    );
}

/// **A stopped driver will not mend itself**, so its row wears the error
/// accent — and a quiescent one wears `Rest`, which is weak ink and therefore
/// a rule that recedes rather than a seventh colour.
#[test]
fn a_stopped_row_wears_the_error_rule_and_a_settled_one_the_resting_rule() {
    let mut stopped = listing(ConvRow {
        state: AgentState::Stopped,
        ..conv("id", "stopped")
    });
    let painted = fills(&mut stopped);
    assert!(ruled(&painted, accent(State::Error)), "the error rule");
    assert!(
        !ruled(&painted, accent(State::Attention)),
        "nothing is asking"
    );

    let mut settled = listing(conv("id", "settled"));
    assert!(
        ruled(&fills(&mut settled), accent(State::Rest)),
        "a quiescent row still says so, in the ink that recedes"
    );
}

/// **The selected row is raised and wears the brand**, whatever its state
/// says: the selection is where the operator IS, and the asking is said in the
/// row's words. This is also where the hand-painted selection fill went — a
/// reserved `Shape::Noop` set after the run (bl-dc07) — and the assertion is
/// the one that survived it: the tint is on the glass and the name is still
/// readable over it.
#[test]
fn the_selected_row_is_raised_and_wears_the_brand() {
    let mut model = Model {
        convs: vec![ConvRow {
            state: AgentState::Stopped,
            ..conv("chosen", "the chosen one")
        }],
        conversation: Some("chosen".to_owned()),
        ..seated()
    };
    let painted = fills(&mut model);
    assert!(
        painted.iter().any(|(_, ink)| *ink == RAISED),
        "the chosen row is raised"
    );
    assert!(ruled(&painted, BRAND), "and its rule is the brand");
    assert!(
        !ruled(&painted, accent(State::Error)),
        "not the state's, which the words carry instead"
    );
}

/// Rows at the given depths, in the engine's own order.
fn branch(depths: &[u64]) -> Vec<ConvRow> {
    depths
        .iter()
        .enumerate()
        .map(|(n, depth)| ConvRow {
            depth: *depth,
            members: 1,
            ..conv(&format!("id-{n}"), &format!("row {n}"))
        })
        .collect()
}

/// **A rail is a claim that the branch has another member coming**, and the
/// truth table is the whole of it. The engine's order is a root followed by
/// its descendants, deepest last, so the answer is the first row below this
/// one that is not deeper than the level: at the level, the branch carries on;
/// shallower, it ended here and a rail would be drawing a sibling that does
/// not exist.
#[test]
fn a_rail_stands_for_an_ancestor_whose_thread_continues_and_for_no_other() {
    let carries_on = branch(&[0, 1, 2, 1, 0]);
    assert!(
        continues(&carries_on, 2, 1),
        "another member of level 1 follows the depth-2 row"
    );
    assert!(
        !continues(&carries_on, 2, 2),
        "nothing follows it at its own level"
    );

    let ended = branch(&[0, 1, 2, 0]);
    assert!(
        !continues(&ended, 2, 1),
        "the next row is a second root: level 1 ended here"
    );

    let last = branch(&[0, 1, 2]);
    assert!(!continues(&last, 2, 1), "nothing follows at all");
}

/// **And the elbow points at the row's words.** The horizontal half of the
/// connector is drawn to exactly where `theme::paint::row` lays the run, so
/// asserting the run's own left edge asserts where the elbow stops: one
/// [`STEP`] per level of descent, from the same pane edge every row starts at.
#[test]
fn a_member_s_words_stand_one_step_per_level_in_from_its_root_s() {
    let mut model = Model {
        convs: branch(&[0, 1, 2, 1, 0]),
        conversation: None,
        ..seated()
    };
    // The fold is the list's own (`crate::ui::model::subtree`): a descendant
    // is on the glass only under an opened root, so the branch is opened
    // before its indents can be read off it.
    model.toggle_subtree("id-0");
    model.toggle_subtree("id-1");
    let window = Window::sized(600.0, 400.0);
    let runs = seen(&window, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| render(ui, &mut model));
    });
    let at = |n: usize| {
        let word = format!("row {n}");
        runs.iter()
            .find(|run| run.text.starts_with(&word))
            .unwrap_or_else(|| panic!("{word:?} is on the glass"))
            .laid
            .min
            .x
    };
    assert!((at(1) - at(0) - STEP).abs() < 0.5, "one level, one step");
    assert!(
        (at(2) - at(0) - STEP * 2.0).abs() < 0.5,
        "two levels, two steps"
    );
    assert!((at(4) - at(0)).abs() < 0.5, "a second root is not indented");
}
