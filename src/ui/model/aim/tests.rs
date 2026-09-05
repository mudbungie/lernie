//! The aim: the pair a row is aimed at by, and the readings the panes take of
//! it.

use crate::test_support::window::seated;
use crate::ui::{Aim, Model};

/// The aim is a pair, because two channels may both hold a wall called `home`
/// and a seat that matched on the name alone would highlight the wrong row.
#[test]
fn the_aim_is_a_channel_and_an_address_together() {
    let model = Model {
        aim: Some(Aim {
            channel: "one".to_owned(),
            address: "home".to_owned(),
        }),
        ..seated()
    };
    assert!(model.aimed_at("one", Some(&"home".to_owned())));
    assert!(!model.aimed_at("two", Some(&"home".to_owned())));
    assert!(!model.aimed_at("one", Some(&"other".to_owned())));
    assert!(
        !model.aimed_at("one", None),
        "an unreachable row is never aimed at"
    );
    assert!(!Model::default().aimed_at("one", Some(&"home".to_owned())));
}

/// **A channel this seat no longer holds is one no worker will ask about** —
/// the one aim whose emptiness is permanent, which is why the roster rather
/// than the aim is what answers it.
#[test]
fn a_channel_is_held_only_while_the_roster_carries_it() {
    let model = seated();
    let held = model.roster[0].channel.name.clone();
    assert!(model.holds(&held));
    assert!(!model.holds("a channel this box never had"));
}
