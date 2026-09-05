//! What the clients pane says: the three empty states, the two lifetimes on a
//! row, and the consent said on every tool.

use super::{
    CLOSE, HEADING, NO_SUBJECT, NONE_REGISTERED, NOT_ANSWERED, OFFERS_NOTHING, SUBJECT, render,
};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, machine, machines, pane, seated};
use crate::ui::Model;

/// A closed pane paints nothing and says so, which is what lets the shell put
/// the conversation back where it was.
#[test]
fn a_shut_pane_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood, "a shut pane reports that it painted nothing");
    assert!(!painted.contains(HEADING), "{painted}");
}

/// **Three states, and the first two are different sentences.** A wall nobody
/// has been answered about is not a wall that holds no registration — and the
/// second says whose act would change it, because it is not one from here.
#[test]
fn an_unanswered_wall_and_a_wall_with_no_machine_say_different_things() {
    for (rows, expected) in [
        (None, NOT_ANSWERED.to_owned()),
        (Some(Vec::new()), NONE_REGISTERED.to_owned()),
        (Some(vec![machine("laptop", true)]), "laptop".to_owned()),
    ] {
        let mut model = Model {
            listing: Some(crate::ui::Listing::Clients),
            machines: rows,
            ..seated()
        };
        let painted = pane(|ui| {
            render(ui, &mut model);
        });
        assert!(painted.contains(&expected), "{expected:?}:\n{painted}");
    }
}

/// **Both lifetimes are on the row, and both are said.** Presence is true only
/// at the moment the engine answered; the advertised set stands whether or not
/// the machine is connected — so a row reads *not connected* beside a full set
/// as the ordinary thing.
#[test]
fn a_row_says_whether_it_was_connected_and_what_it_offers_regardless() {
    let mut model = machines();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    for word in [
        "laptop",
        crate::reply::clients::HERE,
        "desk",
        crate::reply::clients::AWAY,
        "2 tool(s)",
        "run a command",
    ] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **The consent is said on every tool, present or absent** — the whole
/// complaint that landed this pane was being unable to tell a box that will
/// take a caller-named directory from one that refuses it, and a line that
/// appeared only when true would leave the second unsaid.
#[test]
fn every_tool_says_whether_its_box_takes_a_caller_named_directory() {
    let mut model = machines();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains(SUBJECT), "{painted}");
    assert!(painted.contains(NO_SUBJECT), "{painted}");
}

/// **A machine that has advertised nothing says so**, which is not the same
/// claim as a machine with no consent on its tools.
#[test]
fn a_machine_with_no_set_says_it_has_advertised_none() {
    let mut model = machines();
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains(OFFERS_NOTHING), "{painted}");
}

/// The way out is the pane's own control, and it leaves the rows where they
/// are.
#[test]
fn the_close_control_puts_the_pane_down() {
    let window = Window::new();
    let mut model = machines();
    click(&window, CLOSE, |ctx| crate::ui::render(ctx, &mut model));
    assert!(!model.showing(crate::ui::Listing::Clients));
    assert!(model.machines.is_some());
}
