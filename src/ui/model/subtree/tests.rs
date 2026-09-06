//! The fold as a value: what a closed subtree hides, what an open one shows,
//! and that the one list is what both surfaces read.

use crate::reply::convs::ConvRow;
use crate::test_support::window::{conv, seated};
use crate::ui::Model;

/// The session the ball measured, in the shape the wire answers it: a root
/// with four compactors under it, and a second root beside them.
fn session() -> Vec<ConvRow> {
    let mut rows = Vec::new();
    let mut root = conv("c-root", "ShorelineGuppy");
    root.members = 5;
    rows.push(root);
    for name in [
        "CardboardFoothill",
        "CrispGorge",
        "DuneBreeze",
        "TortillaSaucepan",
    ] {
        let mut child = conv(name, name);
        child.depth = 1;
        child.preview = "You are the compactor for branch `20260906T034041Z`.".to_owned();
        rows.push(child);
    }
    rows.push(conv("c-other", "WhiskFrost"));
    rows
}

fn named(model: &Model) -> Vec<String> {
    model.rows().into_iter().map(|row| row.display).collect()
}

#[test]
fn a_conversation_s_machinery_is_under_it_and_not_beside_it() {
    let model = Model {
        convs: session(),
        ..seated()
    };
    assert_eq!(named(&model), vec!["ShorelineGuppy", "WhiskFrost"]);
}

#[test]
fn opening_one_subtree_shows_it_and_leaves_the_others_alone() {
    let mut model = Model {
        convs: session(),
        ..seated()
    };
    model.toggle_subtree("c-root");
    assert!(model.subtree_open("c-root"));
    assert_eq!(
        named(&model),
        vec![
            "ShorelineGuppy",
            "CardboardFoothill",
            "CrispGorge",
            "DuneBreeze",
            "TortillaSaucepan",
            "WhiskFrost",
        ]
    );
    model.toggle_subtree("c-root");
    assert!(!model.subtree_open("c-root"));
    assert_eq!(named(&model), vec!["ShorelineGuppy", "WhiskFrost"]);
}

/// **A subtree under a subtree is its own gesture.** Opening a root shows its
/// children; what hangs under one of THOSE is under it, on the same rule.
#[test]
fn a_deeper_subtree_is_folded_under_the_child_that_holds_it() {
    let mut rows = session();
    let mut deep = conv("c-deep", "MetronomeHumble");
    deep.depth = 2;
    rows.insert(2, deep);
    let mut model = Model {
        convs: rows,
        ..seated()
    };
    model.toggle_subtree("c-root");
    assert!(!named(&model).contains(&"MetronomeHumble".to_owned()));
    model.toggle_subtree("CardboardFoothill");
    assert!(named(&model).contains(&"MetronomeHumble".to_owned()));
}
