//! The staged proposals between frames: how an answer is filed, the depth the
//! read stands at, and the arming on each verdict.

use crate::reply::read;
use crate::state::{Open, Standing};
use crate::test_support::window::panes::{PROPOSED, proposed};
use crate::test_support::window::{configured, own, seated};
use crate::ui::{Model, Posted};
use serde_json::{Value, json};

/// The fixture's named row, armed.
fn armed() -> Model {
    let mut model = configured();
    *model.verdict_box().expect("the fixture names a row") = format!(" {PROPOSED} ");
    model
}

/// The envelope a verdict on the fixture's row composes.
fn settle(verdict: &str) -> Value {
    json!({"op": "proposal", "workspace": "home", "id": PROPOSED, "verdict": verdict})
}

/// **An empty listing is filed as an answer**, not dropped: nothing staged is
/// a fact the pane says, which is different from nobody having answered.
#[test]
fn an_empty_listing_is_filed_as_the_answer_it_is() {
    let mut model = seated();
    model.absorb(
        &own().channel,
        read(&json!({"ok": true, "kind": "proposals", "rows": []})),
    );
    let held = model.proposals.expect("filed");
    assert!(held.rows.is_empty());
    assert_eq!(held.whole, None);
}

/// **A populated listing is filed whole** — the rows and, at the deeper depth,
/// the named proposal's message and diff, as one answer.
#[test]
fn a_populated_listing_is_filed_with_its_whole() {
    let mut model = seated();
    let row = json!({"id": PROPOSED, "lineages": ["default"], "parent": "9f2c1ab4",
        "fresh": true, "diffstat": "1 file changed", "subject": "stop retrying"});
    model.absorb(
        &own().channel,
        read(&json!({"ok": true, "kind": "proposals", "rows": [row], "whole": "diff"})),
    );
    let held = model.proposals.expect("filed");
    assert_eq!(held.rows.len(), 1);
    assert_eq!(held.rows[0].id, PROPOSED);
    assert_eq!(held.whole.as_deref(), Some("diff"));
}

/// **Naming a row deepens the standing read to it**, and the depth is the
/// named id only while the listing still carries it — a settled proposal
/// leaves the listing, and the read goes back to bare.
#[test]
fn the_read_stands_at_the_named_depth_only_while_the_row_is_staged() {
    let mut model = configured();
    let standing = Standing::of(&model);
    assert_eq!(standing.named(), Some(PROPOSED.to_owned()));
    assert_eq!(
        standing.open,
        Some(Open::Config(standing.at(), Some(PROPOSED.to_owned())))
    );
    model.proposals.as_mut().expect("fixture").rows.clear();
    assert_eq!(model.proposing(), None, "gone from the listing");
    assert_eq!(Standing::of(&model).named(), None);
    assert_eq!(Standing::of(&seated()).named(), None, "no pane, no depth");
}

/// **Naming another row drops the last one's whole**, keeping the rows: the
/// answer carries no id, so a diff under a new name would be unattributable.
/// The way out does the same and settles nothing.
#[test]
fn naming_and_unnaming_drop_the_whole_and_keep_the_rows() {
    let mut model = configured();
    model.name_proposal("other");
    assert_eq!(
        model.named_proposal().map(|held| held.id),
        Some("other".to_owned())
    );
    let held = model.proposals.clone().expect("rows kept");
    assert_eq!((held.rows.len(), held.whole), (1, None));
    let mut model = configured();
    model.unname_proposal();
    assert_eq!(model.named_proposal(), None);
    assert_eq!(
        model.proposals.as_ref().and_then(|held| held.whole.clone()),
        None
    );
    assert!(model.outbox.is_empty(), "the way out composes nothing");
}

/// **With no pane open, naming and unnaming touch nothing.**
#[test]
fn with_no_pane_the_row_acts_do_nothing() {
    let mut model = seated();
    model.name_proposal(PROPOSED);
    model.unname_proposal();
    model.settle_proposal("accept");
    assert_eq!((model.named_proposal(), model.verdict_box()), (None, None));
    assert!(model.outbox.is_empty());
}

/// **Each verdict is dark until the id is typed back**, and armed it composes
/// the envelope the verb row builds — keeping the arming, because a stale
/// accept refuses and a retype would be a toll on the engine's *no*.
#[test]
fn each_verdict_fires_only_armed_and_keeps_the_arming() {
    for verdict in crate::verbs::proposals::VERDICTS {
        let mut model = configured();
        model.settle_proposal(verdict);
        assert!(model.outbox.is_empty(), "{verdict}: unarmed fires nothing");
        let mut model = armed();
        model.settle_proposal(verdict);
        assert_eq!(model.outbox, vec![Posted::act(settle(verdict))]);
        let held = model.named_proposal().expect("still named");
        assert!(held.posted && held.armed(), "{verdict}: {held:?}");
    }
}

/// **A row the listing no longer carries, or a pane with no aim, settles
/// nothing** — there is no subject left to say yes or no to.
#[test]
fn a_gone_row_or_a_missing_aim_settles_nothing() {
    let mut gone = armed();
    gone.proposals.as_mut().expect("fixture").rows = vec![proposed("other", false)];
    gone.settle_proposal("accept");
    assert!(gone.outbox.is_empty());
    let mut unaimed = armed();
    unaimed.aim = None;
    unaimed.settle_proposal("reject");
    assert!(unaimed.outbox.is_empty());
}

/// **A new aim takes the listing with the pane**, on the config answers' own
/// terms: a proposal is one wall's candidate.
#[test]
fn a_new_aim_retires_the_listing() {
    let mut model = configured();
    model.aim_at("(this box's own engine)", "elsewhere");
    assert_eq!(model.proposals, None);
}
