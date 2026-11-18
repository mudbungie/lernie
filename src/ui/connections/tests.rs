//! The connections page: what each connection says, and the two clicks.

use super::{ABOUT, CONFIRM, KEEP, NO_ADDRESS, OWN, WARNING, facts, remove, render};
use crate::test_support::window::{own, pane, seated, workspace};
use crate::ui::{Channel, Chunk, Held, Lookup, Model, Page};
use egui_kittest::kittest::Queryable;

fn entry(name: &str, there: &str, dials: Option<&str>, held: Held) -> Chunk {
    Chunk {
        channel: Channel {
            name: name.to_owned(),
            named_there: Some(there.to_owned()),
            dials: dials.map(str::to_owned),
        },
        held,
        workspaces: vec![workspace("home"), workspace("ops")],
        ..Chunk::default()
    }
}

fn held() -> Model {
    let mut model = seated();
    model.roster[0].channel.dials = Some("127.0.0.1:7737".to_owned());
    model.roster.extend([
        entry("lab", "lab", Some("127.0.0.1:7737"), Held::Heard),
        entry("zoo", "home", Some("engine.example:7737"), Held::Unheard),
        entry("bare", "bare", None, Held::Unheld("no key".to_owned())),
    ]);
    model.lookup = Some(Lookup::Settings(Page::Connections));
    model
}

/// **Every connection says the whole of itself**: its host-side name, what it
/// dials, which other connection dials the same, where its files are and
/// what it last said.
#[test]
fn a_connection_states_every_fact_this_box_holds_about_it() {
    let model = held();
    let said = |at: usize| facts(&model.roster[at], &model.roster);
    assert_eq!(
        said(0),
        [
            "dials 127.0.0.1:7737",
            "the same address as lab",
            "files: wire under this box's data root",
            "answered with 1 workspace(s): home",
        ]
    );
    assert_eq!(
        said(1),
        [
            "named \"lab\" on its host",
            "dials 127.0.0.1:7737",
            "the same address as (this box's own server)",
            "files: wire/workspaces/lab under this box's data root",
            "answered with 2 workspace(s): home, ops",
        ]
    );
    assert_eq!(said(2)[0], "named \"home\" on its host");
    assert_eq!(said(2)[3], crate::ui::roster::NOT_ANSWERED);
    let bare = said(3);
    assert_eq!(bare[1], NO_ADDRESS);
    assert_eq!(bare.len(), 4, "no address is nobody's twin");
    assert!(bare[3].ends_with("no key"), "{bare:?}");
}

/// **The page lists every connection, offers removal on entries alone**, and
/// says why this box's own server has none.
#[test]
fn the_page_lists_every_connection_and_offers_removal_on_entries_alone() {
    let mut model = held();
    let said = pane(|ui| render(ui, &mut model));
    for words in [ABOUT, OWN, "lab", "zoo", &remove("lab"), &remove("zoo")] {
        assert!(said.contains(words), "{words:?}:\n{said}");
    }
    assert!(!said.contains(&remove("(this box's own server)")), "{said}");
    assert!(!said.contains(WARNING), "nothing armed:\n{said}");
}

/// **The first click arms and says what goes; the way out disarms; the
/// second click asks** — and the removal is handed over, not performed.
#[test]
fn two_clicks_ask_for_a_removal_and_the_way_out_asks_for_none() {
    let mut harness = crate::snapshot::seat(held(), 1400.0, 900.0);
    harness.get_by_label(&remove("lab")).click();
    harness.run();
    assert!(harness.query_by_label(WARNING).is_some());
    assert!(harness.query_by_label(&remove("lab")).is_none());
    harness.get_by_label(KEEP).click();
    harness.run();
    assert_eq!(harness.state().servers.forgetting.armed, None);
    harness.get_by_label(&remove("lab")).click();
    harness.run();
    harness.get_by_label(CONFIRM).click();
    harness.run();
    assert_eq!(harness.state().servers.forgetting.asked, ["lab"]);
    assert_eq!(harness.state().roster.len(), 4, "the frame removes nothing");
}

/// A removal that was refused says why at the head of the page.
#[test]
fn a_refused_removal_is_said_on_the_page() {
    let mut model = held();
    model.servers.forgetting.refused = Some("wire/workspaces/lab was not removed".to_owned());
    let said = pane(|ui| render(ui, &mut model));
    assert!(said.contains("was not removed"), "{said}");
}

/// The own server's chunk in the fixture is this box's own.
#[test]
fn the_fixture_s_first_connection_is_the_own_server() {
    assert_eq!(own().channel.named_there, None);
}
