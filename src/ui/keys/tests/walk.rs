//! **The one cursor track** (DESIGN §4.39): where each arrow lands, and what
//! landing there means — a server's row stood on, a workspace aimed, a
//! conversation selected — plus the sideways keys, which name a column in the
//! narrow shape and nothing at all in the broad one.
//!
//! Split from [`super`] at the 300-line cap on the seam the module has: that
//! file is what stands the keys DOWN and the one binding that is not a walk,
//! and this is the walk.

use super::{stocked, typed};
use crate::test_support::window::painted;
use crate::ui::{Aim, Model, Stand};

/// The server row `name`, as the walk stands on it.
fn server(name: &str) -> Stand {
    Stand::Server(name.to_owned())
}

/// `+ new` under the own server's `home`, and `+ new workspace` under it
/// (bl-f991): the two begins the walk stands on before the next server.
fn begins() -> [Option<Stand>; 2] {
    let own = "(this box's own server)".to_owned();
    let home = Aim {
        channel: own.clone(),
        address: "home".to_owned(),
    };
    [Some(Stand::Begin(home)), Some(Stand::Found(own))]
}

/// **The roster's walk crosses two kinds of row** (DESIGN §4.39): landing on
/// a workspace aims it, exactly as a click does, and landing on a server's row
/// only STANDS there — a walk that opened every server it moved through would
/// be a walk nobody could use to reach the one below.
#[test]
fn the_arrows_walk_the_server_rows_and_the_open_server_s_workspaces() {
    let mut model = Model {
        aim: None,
        switching: true,
        ..stocked()
    };
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.standing,
        Some(server("(this box's own server)")),
        "a list nobody has entered opens at its first row, which is a server"
    );
    assert_eq!(model.aim, None, "and standing there selects nothing");
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("home".to_owned()),
        "the open server's workspace is the next stop, and landing on it aims it"
    );
    assert_eq!(model.standing, None, "an aim is where the cursor is");
    for begin in begins() {
        typed(&mut model, &[egui::Key::ArrowDown]);
        assert_eq!(model.standing, begin, "the begins stand last under it");
    }
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.standing,
        Some(server("elsewhere")),
        "and the closed server's own row is next, never its workspaces"
    );
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.standing,
        Some(server("elsewhere")),
        "the end saturates rather than wrapping"
    );
    typed(&mut model, &[egui::Key::ArrowUp, egui::Key::ArrowUp]);
    assert_eq!(model.standing, begins()[0]);
}

/// **Enter opens the server the walk is standing on**, through the same door
/// the row's own click calls — which is what keeps the binding from being a
/// second surface. It is a second keypress and not the walk's own act, because
/// the walk has to be able to pass a closed server without opening it.
#[test]
fn enter_on_the_row_the_walk_stands_on_opens_that_server() {
    let mut model = Model {
        aim: None,
        switching: true,
        ..stocked()
    };
    typed(
        &mut model,
        &[
            egui::Key::ArrowDown,
            egui::Key::ArrowDown,
            egui::Key::ArrowDown,
            egui::Key::ArrowDown,
            egui::Key::ArrowDown,
            egui::Key::Enter,
        ],
    );
    assert_eq!(model.servers.open.as_deref(), Some("elsewhere"));
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.clone()),
        Some("elsewhere".to_owned()),
        "opening aims the server's first addressable workspace"
    );
}

/// **The conversations are stops on the SAME track** (DESIGN §4.39): the walk
/// goes server row, workspace, that workspace's conversations, next server row — one
/// list in paint order — and landing on a conversation selects it, which is
/// the selection every read follows.
#[test]
fn the_walk_runs_on_through_the_aimed_workspace_s_conversations() {
    let mut model = Model {
        conversation: None,
        switching: true,
        ..stocked()
    };
    // The cursor is on the aimed workspace, so one step down is its first
    // conversation.
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(model.conversation.as_deref(), Some("a"));
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(model.conversation.as_deref(), Some("b"));
    // Past the last conversation the track carries on through the begins
    // into the next server's own row, because it is one list and not two.
    for begin in begins() {
        typed(&mut model, &[egui::Key::ArrowDown]);
        assert_eq!(model.standing, begin);
    }
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(model.standing, Some(server("elsewhere")));
    assert_eq!(
        model.conversation.as_deref(),
        Some("b"),
        "standing on a server selects nothing"
    );
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.standing,
        Some(server("elsewhere")),
        "the end saturates rather than wrapping"
    );
}

/// **A row the pointer can aim at is a row a key can aim at.** The pending row
/// a start puts in the list is in the one list both walk, so an arrow leaves it
/// for the first conversation the server actually answered — which is also how
/// an operator escapes a claim whose driver never wrote its branch.
#[test]
fn the_arrows_walk_the_started_conversation_s_row_like_any_other() {
    let mut model = Model {
        conversation: Some("brisk-otter".to_owned()),
        start: Some(crate::ui::model::Start {
            address: "home".to_owned(),
            goal: "port it".to_owned(),
            phase: crate::ui::model::Phase::Started("brisk-otter".to_owned()),
            spread: None,
        }),
        switching: true,
        ..stocked()
    };
    let shown = painted(&mut model);
    assert!(shown.contains("brisk-otter"), "{shown}");
    typed(&mut model, &[egui::Key::ArrowDown]);
    assert_eq!(
        model.conversation.as_deref(),
        Some("a"),
        "one step down out of the pending row and into the list"
    );
}

/// **The walk reaches the two begins, and Enter fires each as its click
/// does** (bl-f991): on `+ new` the switcher closes on the workspace's start, and
/// on `+ new workspace` it closes on the founding's box — while the begin the
/// cursor stands on wears the brand rule and is brought into view.
#[test]
fn enter_on_a_begin_the_walk_stands_on_fires_it() {
    let at = |n: usize| {
        let mut model = Model {
            conversation: None,
            switching: true,
            ..stocked()
        };
        // Past the aimed workspace's two conversations, `a` and `b`.
        typed(&mut model, &vec![egui::Key::ArrowDown; n + 2]);
        assert_eq!(model.standing, begins()[n - 1]);
        let shown = typed(&mut model, &[egui::Key::Enter]);
        assert!(!model.switching, "the begin closes the switcher");
        (model, shown)
    };
    let (begun, shown) = at(1);
    assert_eq!(begun.conversation, None, "a start is no conversation");
    assert_eq!(begun.standing, None, "an aim is where the cursor is");
    assert!(shown.contains("home"), "{shown}");
    let (founding, shown) = at(2);
    assert_eq!(
        founding.founding.as_deref(),
        Some("(this box's own server)")
    );
    assert_eq!(founding.standing, None);
    assert!(shown.contains(crate::ui::composer::found::HINT), "{shown}");
}
