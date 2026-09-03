//! The two window-level panes between frames: what opening each asks, what an
//! answer replaces, what the needle enables, and what the roster refresh
//! composes.

use super::{Hits, Lookup, Pages};
use crate::reply::search::Found;
use crate::test_support::window::{helped, hit, own, seated};
use crate::ui::{Channel, Model};

/// A second channel, to prove an answer replaces its own section and no other.
fn other() -> Channel {
    Channel {
        name: "elsewhere".to_owned(),
        named_there: None,
        dials: None,
    }
}

/// **Opening the commands pane asks for the table**, because the read is
/// posted rather than standing: a verb table cannot change under an operator
/// and re-asking it on every beat would spend a round trip per channel forever.
#[test]
fn opening_the_commands_pane_composes_the_ask_and_closing_keeps_the_rows() {
    let mut model = seated();
    model.begin_commands();
    assert!(model.commanding());
    assert_eq!(model.outbox, vec![crate::verbs::window::help()]);
    model.paged(&own().channel, vec![helped("scan", "control")]);
    model.close_lookup();
    assert!(!model.commanding());
    assert_eq!(model.pages.len(), 1, "the rows survive the close");
}

/// **An answer replaces its own channel's section and leaves every other
/// standing** — REMOTE §8.2's rule, the shape the roster already keeps.
#[test]
fn a_channels_answer_replaces_only_its_own_section() {
    let mut model = seated();
    model.paged(&own().channel, vec![helped("scan", "control")]);
    model.paged(&other(), vec![helped("ops", "machine")]);
    model.paged(&own().channel, vec![helped("nudge", "control")]);
    assert_eq!(
        model.pages,
        vec![
            Pages {
                channel: own().channel,
                rows: vec![helped("nudge", "control")],
            },
            Pages {
                channel: other(),
                rows: vec![helped("ops", "machine")],
            },
        ]
    );
}

/// The hits keep the same per-channel reading.
#[test]
fn a_channels_hits_replace_only_its_own_section() {
    let answered = |needle: &str| Found {
        needle: needle.to_owned(),
        rows: vec![hit("conversation")],
        unreadable: Vec::new(),
    };
    let mut model = seated();
    model.hit(&own().channel, answered("one"));
    model.hit(&other(), answered("two"));
    model.hit(&own().channel, answered("three"));
    assert_eq!(
        model.found,
        vec![
            Hits {
                channel: own().channel,
                found: answered("three"),
            },
            Hits {
                channel: other(),
                found: answered("two"),
            },
        ]
    );
}

/// **Opening the find pane asks nothing**, because there is no needle yet and
/// a search for nothing is a scan of every store on the box for nothing.
#[test]
fn opening_the_find_pane_asks_nothing() {
    let mut model = seated();
    model.begin_finding();
    assert!(model.finding());
    assert!(model.outbox.is_empty());
    model.close_lookup();
    assert!(!model.finding());
}

/// **Whitespace is not a needle**, and the enablement says so before the act
/// does: a search for it would match every field on the box.
#[test]
fn the_needle_enables_the_act_and_whitespace_is_not_one() {
    let mut model = seated();
    assert!(!model.needled());
    model.post_search();
    assert!(model.outbox.is_empty(), "an empty needle composes nothing");
    model.needle = "   ".to_owned();
    assert!(!model.needled());
    model.post_search();
    assert!(model.outbox.is_empty(), "whitespace composes nothing");
    model.needle = "  gate ".to_owned();
    assert!(model.needled());
    model.post_search();
    assert_eq!(model.outbox, vec![crate::verbs::search("gate".to_owned())]);
    assert_eq!(model.needle, "  gate ", "the box is not spent on firing");
}

/// **The refresh composes the roster read and clears nothing**: each answer
/// replaces its own channel's chunk when it lands, and a channel that cannot
/// be reached says so under its own header.
#[test]
fn the_refresh_asks_every_channel_again_and_drops_nothing() {
    let mut model = seated();
    model.refresh_roster();
    assert_eq!(model.outbox, vec![crate::verbs::workspaces()]);
    assert_eq!(model.roster.len(), 1, "what stands keeps standing");
}

/// **Neither pane is retired by an aim or a selection**, because neither is
/// about one — the decision queue's own rule, one noun over.
#[test]
fn aiming_and_selecting_leave_both_panes_standing() {
    let mut model = Model {
        lookup: Some(Lookup::Commands),
        ..seated()
    };
    model.aim_at("(this box's own engine)", "home");
    model.select("c-9");
    assert!(model.commanding(), "neither is about an aim or a selection");
}

/// **Both answers come in through the one door**, stamped with the channel the
/// leg opened — driven through `Model::absorb` rather than by calling the
/// filing doors, because the door is where a kind is dropped or filed.
#[test]
fn both_kinds_land_through_the_one_door() {
    let mut model = seated();
    model.absorb(
        &own().channel,
        crate::reply::read(&serde_json::json!({
            "kind": "help", "ok": true,
            "rows": [{"verb": "scan", "usage": "/scan", "summary": "sweep",
                      "detail": "one sweep", "surface": "control"}]})),
    );
    assert_eq!(model.pages.len(), 1);
    assert_eq!(model.pages[0].rows[0].verb, "scan");
    model.absorb(
        &own().channel,
        crate::reply::read(&serde_json::json!({
            "kind": "search", "ok": true, "needle": "gate",
            "rows": [], "unreadable": []})),
    );
    assert_eq!(model.found.len(), 1);
    assert_eq!(model.found[0].found.needle, "gate");
}

/// **Escape puts whichever of the two is standing down**, on one arm because
/// one field holds both.
#[test]
fn escape_closes_whichever_of_the_two_is_standing() {
    for open in [Lookup::Commands, Lookup::Finding] {
        let mut model = Model {
            lookup: Some(open),
            ..seated()
        };
        model.escape();
        assert_eq!(model.lookup, None, "{open:?}");
    }
}
