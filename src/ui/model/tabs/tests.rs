//! The tabs: opened by selecting and by nothing else, one per conversation
//! per workspace, closed onto a neighbour or the workspace, and dropped with the
//! listing or the selection the server put down.

use crate::reply::read;
use crate::test_support::window::{own, seated};
use crate::ui::model::{Phase, Start};
use crate::ui::{Model, Tab};
use serde_json::json;

const A: &str = "20260830T051200Z-a1b2";
const B: &str = "20260830T052000Z-b2c3";
const C: &str = "20260830T053000Z-c3d4";

/// A listing of root ids under the aimed workspace.
fn listing(roots: &[&str]) -> crate::reply::Read {
    let rows: Vec<_> = roots
        .iter()
        .map(|root| {
            json!({"root_id": root, "display": root, "name": root, "state": "quiescent",
             "uncertain": false, "preview": "", "age_secs": 0, "attention": 0,
             "members": 1, "depth": 0, "tone": "plain"})
        })
        .collect();
    read(&json!({"ok": true, "kind": "conversations", "rows": rows}))
}

/// The seated workspace with `roots` selected in turn, the last one standing.
fn opened(roots: &[&str]) -> Model {
    let mut model = seated();
    for root in roots {
        model.select(root);
    }
    model
}

/// **Selecting opens one tab, and selecting it again opens none.**
#[test]
fn a_select_opens_one_tab_and_never_a_second() {
    let mut model = opened(&[A, B, A]);
    assert_eq!(model.strip(), [A, B]);
    model.aim = None;
    model.select(C);
    assert_eq!(model.tabs.len(), 2, "no workspace, no tab");
}

/// **The strip is the aimed workspace's**: another workspace's tabs wait under it.
#[test]
fn the_strip_paints_the_aimed_workspaces_tabs_only() {
    let mut model = opened(&[A]);
    model.aim_at(&own().channel.name, "elsewhere");
    assert!(model.strip().is_empty());
    model.select(B);
    assert_eq!(model.strip(), [B]);
    model.aim_at(&own().channel.name, "home");
    assert_eq!(model.strip(), [A], "and comes back with its workspace");
}

/// **Closing the selected tab selects the one to its left**, or to its right
/// at the left end.
#[test]
fn closing_the_selected_tab_selects_its_neighbour() {
    let mut model = opened(&[A, B, C]);
    model.close_tab(C);
    assert_eq!(model.conversation.as_deref(), Some(B));
    model.select(A);
    model.close_tab(A);
    assert_eq!(
        model.conversation.as_deref(),
        Some(B),
        "the right, at the end"
    );
    assert_eq!(model.strip(), [B]);
}

/// **Closing the last puts the selection down to the workspace**, and closing one
/// that is not selected leaves the selection where it was.
#[test]
fn closing_the_last_tab_puts_the_selection_down() {
    let mut model = opened(&[B, A]);
    model.close_tab(B);
    assert_eq!(model.conversation.as_deref(), Some(A));
    model.close_tab("never-opened");
    assert_eq!(model.strip(), [A]);
    let aim = model.aim.clone();
    model.close_tab(A);
    assert_eq!(model.conversation, None);
    assert!(model.transcript.entries.is_empty());
    assert_eq!(model.aim, aim, "back to the workspace it was on");
    assert!(model.strip().is_empty());
}

/// **A tab the listing no longer knows leaves with it** — under the workspace the
/// listing answered, and no other.
#[test]
fn a_listing_without_the_root_drops_its_tab() {
    let mut model = opened(&[A, B]);
    model.tabs.push(Tab {
        aim: crate::ui::Aim {
            channel: own().channel.name,
            address: "elsewhere".to_owned(),
        },
        root_id: C.to_owned(),
    });
    model.absorb(&own().channel, listing(&[B]));
    assert_eq!(model.strip(), [B]);
    assert_eq!(model.tabs.len(), 2, "the other workspace's tab stands");
    model.answered = None;
    model.prune_tabs();
    assert_eq!(model.tabs.len(), 2, "an unanswered workspace drops nothing");
}

/// **A start's tab is opened under its claimed name, survives the listing
/// that does not know it yet, and takes the minted root when it does.**
#[test]
fn a_start_opens_its_tab_and_keeps_it_through_resolving() {
    let mut model = seated();
    model.conversation = None;
    model.start = Some(Start {
        address: "home".to_owned(),
        goal: "port it".to_owned(),
        phase: Phase::Firing,
        spread: None,
    });
    model.absorb(
        &own().channel,
        read(&json!({"ok": true, "kind": "started", "conversation": "brisk-otter"})),
    );
    assert_eq!(model.strip(), ["brisk-otter"]);
    model.absorb(&own().channel, listing(&[A]));
    assert_eq!(model.strip(), ["brisk-otter"], "the claim is not dropped");
    model.select(C);
    model.absorb(
        &own().channel,
        read(&json!({"ok": true, "kind": "conversations", "rows": [
            {"root_id": B, "display": "brisk-otter", "name": "brisk-otter",
             "state": "quiescent", "uncertain": false, "preview": "",
             "age_secs": 0, "attention": 0, "members": 1, "depth": 0, "tone": "plain"},
            {"root_id": C, "display": "c", "name": "c", "state": "quiescent",
             "uncertain": false, "preview": "", "age_secs": 0, "attention": 0,
             "members": 1, "depth": 0, "tone": "plain"}]})),
    );
    assert_eq!(model.strip(), [B, C], "renamed in place, not reopened");
}

/// **A renamed claim whose root is already open folds into that tab.**
#[test]
fn a_rename_onto_an_open_tab_is_one_tab() {
    let mut model = opened(&[A, "brisk-otter"]);
    model.rename_tab("brisk-otter", A);
    assert_eq!(model.strip(), [A]);
}

/// **A selection the server put down is a tab put down** (bl-6dfd).
#[test]
fn a_refused_selection_takes_its_tab() {
    let mut model = opened(&[A, B]);
    model.about(
        &own().channel,
        crate::verbs::AGENT.word,
        read(&json!({"ok": false, "error": "unknown conversation"})),
    );
    assert_eq!(model.conversation, None);
    assert_eq!(model.strip(), [A]);
    model.about(
        &own().channel,
        crate::verbs::AGENT.word,
        read(&json!({"ok": false, "error": "unknown conversation"})),
    );
    assert_eq!(model.strip(), [A], "nothing selected, nothing shut");
}
