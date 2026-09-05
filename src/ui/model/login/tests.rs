//! The login pane between frames: the aim that gates it, the two questions it
//! holds, and what each control composes.

use super::Login;
use crate::test_support::window::{seated, signing};
use crate::ui::{Model, Posted};
use serde_json::json;

/// **The aim is the gate.** Every gesture the pane composes carries a
/// workspace, and a workspace is what an aim is — so a seat aimed at nothing
/// cannot open it at all.
#[test]
fn the_pane_opens_only_where_the_window_is_aimed_at_a_wall() {
    let mut nowhere = Model::default();
    nowhere.begin_login();
    assert_eq!(nowhere.login, None);
    let mut model = seated();
    model.begin_login();
    assert_eq!(model.login, Some(Login::default()));
    assert!(model.covered(), "it covers the conversation");
    model.close_login();
    assert_eq!(model.login, None);
}

/// **Starting a sign-in follows that row and drops the last run's lines.** A
/// second sign-in replaces the first upstream, so keeping the old fold would
/// paint one run's output under another's name.
#[test]
fn signing_in_follows_the_row_composes_the_act_and_drops_the_old_fold() {
    let mut model = signing();
    model.post_signin("otherhouse");
    assert_eq!(model.following().as_deref(), Some("otherhouse"));
    assert_eq!(model.signin, None, "the old run's lines go with it");
    assert_eq!(
        model.outbox,
        vec![Posted::act(
            json!({"op": "login", "workspace": "home", "provider": "otherhouse"})
        )]
    );
}

/// **Asking what a row offers is a READ**, and it drops whatever the last row
/// answered: the reply carries no provider name, so a stale listing under a new
/// question would be unattributable.
#[test]
fn asking_what_a_row_offers_composes_a_read_and_drops_the_last_answer() {
    let mut model = signing();
    model.post_offering("housevendor");
    assert_eq!(
        model.login.as_ref().and_then(|pane| pane.asking.clone()),
        Some("housevendor".to_owned())
    );
    assert_eq!(model.offered, None);
    assert_eq!(
        model.outbox,
        vec![Posted::read(
            json!({"op": "models", "workspace": "home", "provider": "housevendor"})
        )]
    );
}

/// **Neither act reaches anything with the pane shut**, which is what makes a
/// keyboard incapable of firing a control that is not on the glass.
#[test]
fn neither_act_composes_anything_with_the_pane_shut() {
    let mut model = seated();
    model.post_signin("housevendor");
    model.post_offering("housevendor");
    assert!(model.outbox.is_empty());
    assert_eq!(model.following(), None);
}

/// **The aim is the gate at composition too**, not only at opening — the
/// reading that makes a pane open over no wall unreachable rather than merely
/// unlikely.
#[test]
fn an_aimless_pane_composes_nothing_even_when_it_is_open() {
    let mut model = Model {
        login: Some(Login::default()),
        ..Model::default()
    };
    model.post_signin("housevendor");
    model.post_offering("housevendor");
    assert!(model.outbox.is_empty());
    assert_eq!(model.following().as_deref(), Some("housevendor"));
}

/// **The pane and its three answers go with the wall.** Aiming elsewhere
/// retires all four, because the act signs a credential into one wall's store.
#[test]
fn aiming_at_another_wall_retires_the_pane_and_every_answer_on_it() {
    let mut model = signing();
    model.aim_at("(this box's own engine)", "elsewhere");
    assert_eq!(model.login, None);
    assert_eq!(model.providers, None);
    assert_eq!(model.offered, None);
    assert_eq!(model.signin, None);
}

/// **Escape closes it**, on the ladder's own rung and without unmaking or
/// signing anything.
#[test]
fn escape_puts_the_pane_down() {
    let mut model = signing();
    model.escape();
    assert_eq!(model.login, None);
    assert!(model.outbox.is_empty());
}

/// **Where the engine is, is read off the channel stamp and never an address.**
/// This box's own engine carries no name on a host; an entry does; and a
/// channel this seat no longer holds says nothing at all.
#[test]
fn a_wall_held_elsewhere_is_read_off_the_channel_and_not_off_an_address() {
    let mut model = seated();
    assert!(!model.elsewhere(), "this box's own engine is not elsewhere");
    model.roster[0].channel.named_there = Some("theirs".to_owned());
    assert!(model.elsewhere());
    model.roster.clear();
    assert!(!model.elsewhere(), "a channel this seat has lost");
    model.aim = None;
    assert!(!model.elsewhere(), "nothing aimed at is nothing to say");
}

/// **The three answers are filed whether or not the pane is open**, on the
/// roles read's own terms: a frame that arrives after it closed is the last one
/// in flight rather than a thing to drop.
#[test]
fn the_panes_three_answers_come_in_through_the_one_door() {
    let mut model = seated();
    let channel = model.roster[0].channel.clone();
    for (frame, check) in [
        (
            json!({"ok": true, "kind": "providers", "rows": [
                {"name": "housevendor", "fact": "credential present",
                 "blocked": null, "effort": true, "priority": false}]}),
            "providers",
        ),
        (
            json!({"ok": true, "kind": "models", "rows": ["house-model-1"]}),
            "models",
        ),
        (
            json!({"ok": true, "kind": "login",
                   "lines": [{"text": "open it", "err": true}]}),
            "login",
        ),
    ] {
        model.absorb(&channel, crate::reply::read(&frame));
        assert_eq!(model.notice, None, "{check} is content, never a notice");
    }
    assert_eq!(
        model.providers.as_deref().map(<[_]>::len),
        Some(1),
        "the table"
    );
    assert_eq!(
        model.offered,
        Some(vec!["house-model-1".to_owned()]),
        "what the row offers"
    );
    assert_eq!(
        model.signin.expect("the run").lines.len(),
        1,
        "the run's lines"
    );
}
