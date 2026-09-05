//! One wall's row: the line it wears, and the pin control whose word and op
//! follow the row's own rank.

use super::{PIN, UNPIN};
use crate::paint_probe::frame::Window;
use crate::reply::roster::WsRow;
use crate::test_support::window::{click, pane, pinned, seated, wall};
use crate::ui::{Model, Posted};
use serde_json::json;

/// **The control names the act it fires**, because the wire's two ops are
/// assertions rather than a toggle: an unpinned row is offered the pin and a
/// pinned one the unpin, and neither word is on the other's row.
#[test]
fn the_pin_control_follows_the_rows_own_rank() {
    let mut unpinned = seated();
    let painted = pane(|ui| super::super::render(ui, &mut unpinned));
    assert!(painted.contains(PIN), "{painted}");
    assert!(!painted.contains(UNPIN), "{painted}");
    let mut model = pinned();
    let painted = pane(|ui| super::super::render(ui, &mut model));
    assert!(painted.contains(UNPIN), "{painted}");
}

/// **Each click composes the assertion the word names**, and the click is what
/// proves the tag on it names the op it fires.
#[test]
fn clicking_it_composes_the_assertion_and_never_a_flip() {
    for (fixture, label, expected) in [
        (seated(), PIN, json!({"op": "pin", "workspace": "home"})),
        (pinned(), UNPIN, json!({"op": "unpin", "workspace": "home"})),
    ] {
        let mut model = fixture;
        let window = Window::new();
        click(&window, label, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                super::super::render(ui, &mut model);
            });
        });
        assert_eq!(model.outbox, vec![Posted::act(expected)], "{label}");
    }
}

/// **The machines control opens the pane on the aimed wall**, which is what
/// stands its one read up — the read has no control of its own, exactly as the
/// tuning pane's and the login pane's do not.
#[test]
fn the_machines_control_opens_the_clients_pane() {
    let mut model = seated();
    let window = Window::new();
    click(&window, crate::ui::clients::OPEN, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::super::render(ui, &mut model);
        });
    });
    assert!(model.showing(crate::ui::Listing::Clients));
    assert!(model.outbox.is_empty(), "opening a pane composes nothing");
}

/// **The config control opens the pane on the aimed wall**, which is what
/// stands its lineage read up — the read has no control of its own.
#[test]
fn the_config_control_opens_the_config_pane() {
    let mut model = seated();
    let window = Window::new();
    click(&window, crate::ui::config::OPEN, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            super::super::render(ui, &mut model);
        });
    });
    assert!(model.configuring.is_some());
    assert!(model.outbox.is_empty(), "opening a pane composes nothing");
}

/// **The aim is the gate**, exactly as it is for every other per-wall act:
/// nothing aimed at is nothing to assert about.
#[test]
fn an_aimless_seat_composes_no_pin() {
    let mut model = Model::default();
    model.post_pin(true);
    assert!(model.outbox.is_empty());
}

/// **It hangs off the aimed row and off no other**, and stands down while a
/// pane covers the conversation — the rule all six per-wall controls keep.
#[test]
fn it_is_offered_on_the_aimed_row_alone_and_stands_down_under_a_pane() {
    let mut elsewhere = Model {
        aim: None,
        ..seated()
    };
    let painted = pane(|ui| super::super::render(ui, &mut elsewhere));
    assert!(!painted.contains(PIN), "no row is aimed at:\n{painted}");
    let mut covered = seated();
    covered.begin_tuning();
    let painted = pane(|ui| super::super::render(ui, &mut covered));
    assert!(!painted.contains(PIN), "a pane is standing:\n{painted}");
}

/// A row this seat cannot address is painted as what it is, and carries no
/// control at all.
#[test]
fn a_row_with_no_entry_naming_it_wears_the_line_and_no_control() {
    let quiet = super::line(&wall("home"));
    assert!(quiet.contains("home"), "{quiet}");
    let busy = super::line(&WsRow {
        attention: 3,
        running: true,
        ..wall("home")
    });
    assert!(
        busy.contains("3 waiting") && busy.contains("running"),
        "{busy}"
    );
}
