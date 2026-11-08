//! The pages: which one stands is read off the panes' own fields, a page is
//! opened by its word where it has one, and the mark's door reopens the last.

use super::{Group, Page};
use crate::test_support::window::seated;
use crate::ui::{Lookup, Model};

/// **Nine pages, three groups, every label distinct**, and each group's pages
/// are exactly the ones that name it.
#[test]
fn the_tree_is_nine_pages_under_three_groups_with_distinct_labels() {
    let labels: std::collections::BTreeSet<_> = Page::ALL.iter().map(|page| page.label()).collect();
    assert_eq!(labels.len(), Page::ALL.len());
    let grouped: Vec<Page> = Group::ALL.into_iter().flat_map(Group::pages).collect();
    assert_eq!(
        grouped,
        Page::ALL.to_vec(),
        "the groups walk the tree in order"
    );
    assert_eq!(Page::default(), Page::Seat(crate::settings::Page::Look));
    for page in Page::ALL {
        assert!(!page.help().is_empty(), "{page:?} says what it is about");
    }
}

/// **The page is derived**: each pane's field answers for its page, the
/// lookup for the rest, and nothing standing is no page.
#[test]
fn the_page_is_read_off_the_panes_own_fields() {
    let mut model = seated();
    assert_eq!(model.page(), None);
    model.begin_tuning();
    assert_eq!(model.page(), Some(Page::Roles));
    model.escape();
    model.begin_login();
    assert_eq!(model.page(), Some(Page::Providers));
    model.escape();
    model.begin_configuring();
    assert_eq!(model.page(), Some(Page::Config));
    model.escape();
    model.begin_clients();
    assert_eq!(model.page(), Some(Page::Clients));
    model.escape();
    model.begin_doctor();
    assert_eq!(model.page(), Some(Page::Doctor));
    model.escape();
    model.lookup = Some(Lookup::Settings(Page::Enrollment));
    assert_eq!(model.page(), Some(Page::Enrollment));
    model.lookup = Some(Lookup::Commands);
    assert_eq!(model.page(), None, "another lookup is not a page");
}

/// **A page with a word is offered while its word is**: the workspace's
/// four and the doctor need what their words need, and the rest are always.
#[test]
fn a_page_is_offered_exactly_while_its_word_is() {
    let aimed = seated();
    for page in Page::ALL {
        assert!(aimed.offers(page), "{page:?} with an aim");
    }
    let bare = Model::default();
    for page in Page::ALL {
        let expected = !matches!(
            page,
            Page::Roles | Page::Providers | Page::Config | Page::Clients
        );
        assert_eq!(bare.offers(page), expected, "{page:?} with nothing aimed");
    }
}

/// **Turning to a page opens it alone**, over whatever stood, by its word
/// where it has one — so a word's page posts what the word posts.
#[test]
fn turning_to_a_page_opens_it_alone_by_its_word() {
    let mut model = seated();
    model.begin_tuning();
    assert!(model.turn_to(Page::Seat(crate::settings::Page::Bar)));
    assert_eq!(model.tuning, None, "what stood went down");
    assert_eq!(
        model.lookup,
        Some(Lookup::Settings(Page::Seat(crate::settings::Page::Bar)))
    );
    assert!(model.turn_to(Page::Doctor));
    assert_eq!(model.page(), Some(Page::Doctor));
    assert!(!model.outbox.is_empty(), "the doctor's word posts its read");
    assert!(model.turn_to(Page::Roles));
    assert_eq!(model.page(), Some(Page::Roles));
    let mut bare = Model::default();
    assert!(!bare.turn_to(Page::Roles), "no aim, no roles");
    assert_eq!(bare.page(), None);
}

/// **Nothing opens under the enrollment**, which only its own gesture leaves.
#[test]
fn nothing_turns_under_the_enrollment() {
    let mut model = seated();
    model.begin_enrollment();
    assert!(!model.turn_to(Page::Enrollment));
    assert!(!model.turn_to(Page::Roles));
    assert!(model.enroll.is_some());
    assert_eq!(model.page(), None);
}

/// **The mark reopens the page last shown**, and *Seat › look* where that
/// page is not offered now.
#[test]
fn the_mark_reopens_the_last_page_or_look() {
    let mut model = seated();
    model.visited = Page::Clients;
    model.reopen_settings();
    assert_eq!(model.page(), Some(Page::Clients));
    let mut bare = Model {
        visited: Page::Clients,
        ..Model::default()
    };
    bare.reopen_settings();
    assert_eq!(bare.page(), Some(Page::default()));
}
