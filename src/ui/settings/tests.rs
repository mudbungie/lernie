//! The overlay: the breadcrumb, the tree as a column and as a select, every
//! knob a row of its page, the pin, the defaults and the search.

use super::rows::{DEFAULTS, PINNED};
use super::{HEADING, WORD, render};
use crate::paint_probe::frame::Window;
use crate::settings::{KNOBS, Settings};
use crate::test_support::window::{click, pane, seated};
use crate::ui::{Lookup, Model, Page};
use egui_kittest::kittest::Queryable;

/// The knobs' rows, the search and the select.
mod rows;

/// A seat with the overlay standing on `page`.
pub(super) fn open(page: Page) -> Model {
    Model {
        lookup: Some(Lookup::Settings(page)),
        ..seated()
    }
}

/// A closed overlay paints nothing and says so.
#[test]
fn a_shut_overlay_paints_nothing_and_reports_it() {
    let mut model = seated();
    let mut stood = true;
    let painted = pane(|ui| stood = render(ui, &mut model));
    assert!(!stood);
    assert!(!painted.contains(HEADING), "{painted}");
}

/// **The breadcrumb heads every page, each word its own**, and the tree
/// stands beside the page as a column at the desk size with every group and
/// every page on it.
#[test]
fn the_breadcrumb_and_the_tree_stand_on_every_page() {
    for page in Page::ALL {
        let harness = crate::snapshot::seat(open(page), 1400.0, 900.0);
        for word in [HEADING, page.group().label(), page.label()] {
            assert!(
                harness.query_all_by_label(word).next().is_some(),
                "{word:?} on {page:?}"
            );
        }
        for leaf in Page::ALL {
            assert!(
                harness.query_all_by_label(leaf.label()).next().is_some(),
                "{leaf:?} in the tree on {page:?}"
            );
        }
    }
}

/// **A row of the tree turns the overlay to its page**, and the page last
/// shown is remembered for the mark.
#[test]
fn a_tree_row_turns_the_overlay_and_the_page_is_remembered() {
    let mut harness = crate::snapshot::seat(open(Page::default()), 1400.0, 900.0);
    harness.get_by_label(Page::Clients.label()).click();
    harness.run();
    assert_eq!(harness.state().page(), Some(Page::Clients));
    harness.run();
    assert_eq!(harness.state().visited, Page::Clients);
    harness.get_by_label(Page::Doctor.label()).click();
    harness.run();
    assert_eq!(harness.state().page(), Some(Page::Doctor));
}

/// **A page whose word is off the menu is dark**: with nothing aimed the
/// workspace's rows take no click.
#[test]
fn a_page_not_offered_takes_no_click() {
    let bare = Model {
        lookup: Some(Lookup::Settings(Page::default())),
        ..Model::default()
    };
    let mut harness = crate::snapshot::seat(bare, 1400.0, 900.0);
    harness.get_by_label(Page::Roles.label()).click();
    harness.run();
    assert_eq!(harness.state().page(), Some(Page::default()));
}

/// **Every knob is a row of its page**: its label and its sentence on that
/// page and on no other, the pin on the bar's, the defaults link on each.
#[test]
fn every_knob_is_a_row_of_its_page_with_its_sentence() {
    for seat in crate::settings::Page::ALL {
        let mut model = open(Page::Seat(seat));
        let mut stood = false;
        let painted = pane(|ui| stood = render(ui, &mut model));
        assert!(stood);
        assert!(painted.contains(DEFAULTS), "{painted}");
        assert_eq!(painted.contains(PINNED), seat == crate::settings::Page::Bar);
        for knob in KNOBS {
            assert_eq!(
                painted.contains(knob.label),
                knob.page == seat,
                "{}:\n{painted}",
                knob.key
            );
            assert_eq!(
                painted.contains(knob.help),
                knob.page == seat,
                "{}:\n{painted}",
                knob.key
            );
        }
    }
}

/// **The word and the chord open it on *Seat › look*, and Esc puts it down**
/// — the one page on the menu that posts nothing.
#[test]
fn the_word_and_the_chord_open_look_and_escape_closes_it() {
    let mut model = seated();
    assert!(crate::ui::slash::fire(&mut model, WORD, ""));
    assert_eq!(model.page(), Some(Page::default()));
    assert!(model.covered());
    assert!(model.outbox.is_empty(), "nothing crosses the boundary");
    model.escape();
    assert!(!model.covered());
    let window = Window::new();
    let chord = egui::Event::Key {
        key: egui::Key::Comma,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::COMMAND,
    };
    window.frame(vec![chord], |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.page(), Some(Page::default()), "Ctrl+, opens it");
    assert!(
        crate::ui::slash::rows(&model)
            .iter()
            .any(|row| row.word == WORD),
        "on the menu"
    );
}

/// **The doctor's page says the dial state the seat panel says**, where a
/// server will not dial.
#[test]
fn the_servers_pages_say_the_dial_state() {
    let mut model = open(Page::Doctor);
    model.roster[0].held = crate::ui::Held::Unheld("nothing provisioned".to_owned());
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    assert!(painted.contains("nothing provisioned"), "{painted}");
}

/// **The defaults link puts the knobs back and leaves the pin**, which is
/// not a knob.
#[test]
fn defaults_reset_the_knobs_and_leave_the_pin() {
    let window = Window::new();
    let mut model = open(Page::Seat(crate::settings::Page::Bar));
    model.settings.wheel = 200.0;
    model.bar.pinned = true;
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    click(&window, DEFAULTS, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(model.settings, Settings::default());
    assert!(model.bar.pinned);
}
