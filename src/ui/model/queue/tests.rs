//! The queue between frames: what opens and closes it, how a fan's answers
//! land, how a row resolves to an address, and the two acts a row spends.

use crate::test_support::window::{queued, seated, waiting};
use crate::ui::{Aim, Asking, Channel, Model};

/// The channel every fixture row comes down.
fn own() -> Channel {
    crate::test_support::window::own().channel
}

/// **It opens with no subject**, which is the one thing that separates it from
/// the other three covering panes — and closing it keeps the rows, because the
/// next open is about the same queue.
#[test]
fn it_opens_on_an_unaimed_seat_and_keeps_its_rows_when_it_shuts() {
    let mut model = Model::default();
    model.begin_queue();
    assert!(model.queue && model.covered());
    let mut model = queued();
    model.close_queue();
    assert!(!model.queue);
    assert_eq!(model.waiting.len(), 1, "the rows outlive the pane");
}

/// **Its subject is everything, so nothing on the glass retires it.** Aiming
/// and selecting retire the tuning and records panes because those are about a
/// wall and a conversation; this one is about neither.
#[test]
fn no_aim_and_no_selection_can_retire_it() {
    let mut model = queued();
    model.aim_at("(this box's own engine)", "home");
    model.select("c-2");
    assert!(model.queue, "the queue is about no focus");
    assert_eq!(model.waiting.len(), 1);
}

/// **A fan's answer replaces its own channel's section and no other** — the
/// roster's rule, one noun over: a box serving three engines does not lose the
/// two that are fine.
#[test]
fn one_channel_s_answer_leaves_every_other_standing() {
    let elsewhere = Channel {
        name: "elsewhere".to_owned(),
        named_there: None,
        dials: None,
    };
    let mut model = Model::default();
    model.absorb(
        &own(),
        crate::reply::Read::Answer(crate::reply::Reply::Attention(vec![waiting("home", "c-1")])),
    );
    model.absorb(
        &elsewhere,
        crate::reply::Read::Answer(crate::reply::Reply::Attention(vec![waiting("far", "c-9")])),
    );
    assert_eq!(model.waiting.len(), 2);
    // The same channel answering again replaces its own section in place.
    model.absorb(
        &own(),
        crate::reply::Read::Answer(crate::reply::Reply::Attention(Vec::new())),
    );
    assert_eq!(model.waiting.len(), 2);
    assert!(model.waiting[0].rows.is_empty());
    assert_eq!(model.waiting[1].rows.len(), 1, "the other channel stands");
}

/// **A row is addressed off the roster**, so a stamp cannot aim a gesture: the
/// wall a queue row names is resolved where every other aim is resolved, and a
/// wall this seat holds no name for answers nothing.
#[test]
fn a_row_resolves_against_the_roster_and_not_against_its_section() {
    let model = queued();
    assert_eq!(
        model.wall("home"),
        Some(Aim {
            channel: "(this box's own engine)".to_owned(),
            address: "home".to_owned(),
        })
    );
    assert_eq!(model.wall("elsewhere"), None);
    // A section stamped with a channel that renames does not change the answer:
    // the roster is the one authority, and the stamp is a display fact.
    let mut renamed = model;
    renamed.waiting = vec![Asking {
        channel: Channel {
            name: "wrong".to_owned(),
            named_there: Some("theirs".to_owned()),
            dials: None,
        },
        rows: vec![waiting("home", "c-1")],
    }];
    assert!(renamed.wall("home").is_some());
}

/// **The answer is composed against the resolved address**, and a row this
/// seat cannot address composes nothing at all rather than a gesture aimed at a
/// name the engine will refuse.
#[test]
fn seen_is_composed_only_where_the_wall_resolves() {
    let mut model = queued();
    model.post_seen(&waiting("elsewhere", "c-3"));
    assert!(model.outbox.is_empty(), "no address, no gesture");
    model.post_seen(&waiting("home", "c-1"));
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::act(crate::verbs::seen(
            "home".to_owned(),
            "c-1".to_owned()
        ))]
    );
}

/// **Going to a row spends the two doors a pointer already spends** and stands
/// the pane down; an unaddressable row goes nowhere and says so by staying.
#[test]
fn going_to_a_row_aims_selects_and_closes() {
    let mut model = queued();
    model.go_to(&waiting("elsewhere", "c-3"));
    assert!(model.queue, "an unaddressable row leaves the pane standing");
    model.go_to(&waiting("home", "c-1"));
    assert!(!model.queue);
    assert_eq!(
        model.aim,
        Some(Aim {
            channel: "(this box's own engine)".to_owned(),
            address: "home".to_owned(),
        })
    );
    assert_eq!(model.conversation.as_deref(), Some("c-1"));
    assert!(model.outbox.is_empty(), "a view crosses no wire");
}

/// **Escape closes it**, on the ladder's own rung: after every pane that holds
/// more, and before the notice.
#[test]
fn escape_closes_the_queue_before_reaching_the_notice() {
    let mut model = queued();
    model.notice = Some(crate::ui::Notice::Refused("no".to_owned()));
    model.escape();
    assert!(!model.queue, "the pane went down");
    assert!(model.notice.is_some(), "the notice did not");
    model.escape();
    assert_eq!(model.notice, None, "the next escape reaches it");
}

/// The raise carries nothing, so what it does to the window is put the last
/// notice down — the receipt says the gesture landed and the next queue says
/// what it changed.
#[test]
fn the_raise_clears_the_notice_and_files_nothing() {
    let mut model = Model {
        notice: Some(crate::ui::Notice::Refused("earlier".to_owned())),
        ..seated()
    };
    model.absorb(
        &own(),
        crate::reply::Read::Answer(crate::reply::Reply::Flagged),
    );
    assert!(model.notice.is_none());
    assert!(model.waiting.is_empty());
}
