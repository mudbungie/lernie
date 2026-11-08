//! The kept conversations: toggled under the aim, the aim's alone, and
//! dropped with the listing that no longer knows them.

use crate::reply::read;
use crate::test_support::window::{own, seated};
use crate::ui::{Aim, Tab};
use serde_json::json;

const A: &str = "20260830T051200Z-a1b2";
const B: &str = "20260830T052000Z-b2c3";

/// **Keeping twice lets go; with nothing aimed, nothing is kept.**
#[test]
fn keeping_toggles_under_the_aim_and_needs_one() {
    let mut model = seated();
    model.toggle_kept(A);
    model.toggle_kept(B);
    assert_eq!(model.kept(), [A, B]);
    model.toggle_kept(A);
    assert_eq!(model.kept(), [B]);
    model.aim = None;
    model.toggle_kept(A);
    assert!(model.kept().is_empty(), "no aim, nothing to keep under");
    assert_eq!(model.bar.kept.len(), 1);
}

/// **A listing without the root drops it**, under the workspace it answered
/// and no other; an unanswered workspace drops nothing.
#[test]
fn a_listing_without_the_root_lets_it_go() {
    let mut model = seated();
    model.toggle_kept(A);
    model.toggle_kept(B);
    model.bar.kept.push(Tab {
        aim: Aim {
            channel: own().channel.name,
            address: "elsewhere".to_owned(),
        },
        root_id: A.to_owned(),
    });
    let rows = json!([{"root_id": B, "display": B, "name": B, "state": "quiescent",
        "uncertain": false, "preview": "", "age_secs": 0, "attention": 0,
        "members": 1, "depth": 0, "tone": "plain"}]);
    model.absorb(
        &own().channel,
        read(&json!({"ok": true, "kind": "conversations", "rows": rows})),
    );
    assert_eq!(model.kept(), [B]);
    assert_eq!(model.bar.kept.len(), 2, "the other workspace's stands");
    model.answered = None;
    model.prune_kept();
    assert_eq!(
        model.bar.kept.len(),
        2,
        "an unanswered workspace drops nothing"
    );
}
