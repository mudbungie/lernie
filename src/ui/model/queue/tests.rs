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
    assert!(model.showing(crate::ui::Listing::Queue) && model.covered());
    let mut model = queued();
    model.close_queue();
    assert!(!model.showing(crate::ui::Listing::Queue));
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
    assert!(
        model.showing(crate::ui::Listing::Queue),
        "the queue is about no focus"
    );
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

/// **A row is addressed off the roster, within its own channel's chunk**:
/// the wall a queue row names is resolved where every other aim is resolved,
/// and a wall this seat holds no name for ON THAT CHANNEL answers nothing —
/// a channel the roster does not hold included.
#[test]
fn a_row_resolves_against_its_own_channel_s_roster() {
    let model = queued();
    assert_eq!(
        model.wall(&own(), "home"),
        Some(Aim {
            channel: "(this box's own engine)".to_owned(),
            address: "home".to_owned(),
        })
    );
    assert_eq!(model.wall(&own(), "elsewhere"), None);
    assert_eq!(model.wall(&side().channel, "home"), None);
}

/// A second engine whose bootstrap wall is ALSO called `home` on its host,
/// under an entry this box names `side` — the arrangement every two-engine seat has (bl-666f).
fn side() -> crate::ui::Chunk {
    let mut chunk = crate::test_support::window::own();
    chunk.channel = Channel {
        name: "side".to_owned(),
        named_there: Some("home".to_owned()),
        dials: None,
    };
    chunk
}

/// **Two engines each holding `home`: a row aims at its OWN engine's**
/// (bl-666f). Every door — the address, *go to it*, `seen`, `answer` and the
/// ask box — resolves within the channel the row's section came down, so the
/// second engine's row never reaches the first's wall.
#[test]
fn two_engines_holding_the_same_wall_name_each_keep_their_rows() {
    let mut model = queued();
    model.roster.push(side());
    let theirs = side().channel;
    let row = waiting("home", "c-401");
    let aimed = model.wall(&theirs, "home").expect("their home");
    assert_eq!(aimed.channel, "side");
    assert_ne!(aimed, model.wall(&own(), "home").expect("our home"));
    model.post_seen(&theirs, &row);
    model.post_answer(&theirs, &row, "pass", crate::verbs::capability::CALL);
    for posted in &model.outbox {
        assert_eq!(posted.envelope["workspace"], aimed.address.as_str());
    }
    model.outbox.clear();
    model.go_to(&theirs, &row);
    assert_eq!(model.aim, Some(aimed.clone()));
    assert_eq!(model.conversation.as_deref(), Some("c-401"));
    // The ask box is the same row only on the same channel: the selected
    // conversation's id on the OTHER engine's `home` is not this one.
    model.waiting = vec![Asking {
        channel: theirs.clone(),
        rows: vec![row.clone()],
    }];
    assert_eq!(crate::ui::ask::row(&model), Some((theirs, row)));
    model.aim = model.wall(&own(), "home");
    assert_eq!(crate::ui::ask::row(&model), None);
}

/// **The answer is composed against the resolved address**, and a row this
/// seat cannot address composes nothing at all rather than a gesture aimed at a
/// name the engine will refuse.
#[test]
fn seen_is_composed_only_where_the_wall_resolves() {
    let mut model = queued();
    model.post_seen(&own(), &waiting("elsewhere", "c-3"));
    assert!(model.outbox.is_empty(), "no address, no gesture");
    model.post_seen(&own(), &waiting("home", "c-1"));
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
    model.go_to(&own(), &waiting("elsewhere", "c-3"));
    assert!(
        model.showing(crate::ui::Listing::Queue),
        "an unaddressable row leaves the pane standing"
    );
    model.go_to(&own(), &waiting("home", "c-1"));
    assert!(!model.showing(crate::ui::Listing::Queue));
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
    assert!(
        !model.showing(crate::ui::Listing::Queue),
        "the pane went down"
    );
    assert!(model.notice.is_some(), "the notice did not");
    model.escape();
    assert_eq!(model.notice, None, "the next escape reaches it");
}

/// The raise carries nothing, so what it does to the window is say it landed
/// (bl-9760) — the next queue says what it changed, and the receipt files no
/// row of its own.
#[test]
fn the_raise_says_it_landed_and_files_nothing() {
    let mut model = Model {
        notice: Some(crate::ui::Notice::Refused("earlier".to_owned())),
        ..seated()
    };
    model.absorb(
        &own(),
        crate::reply::Read::Answer(crate::reply::Reply::Flagged),
    );
    assert_eq!(
        model.notice,
        Some(crate::ui::Notice::Said(
            "flagged — it is on the queue now".to_owned()
        ))
    );
    assert!(model.waiting.is_empty());
}
