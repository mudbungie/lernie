//! The register page: the field, the button, and each way a claim stands.

use super::{ABOUT, CLAIMING, HINT, REGISTER, WORD, render};
use crate::seat::register::Registered;
use crate::test_support::window::pane;
use crate::ui::{Claim, Lookup, Model, Page};
use egui_kittest::kittest::Queryable;

fn painted(model: &mut Model) -> String {
    pane(|ui| render(ui, model))
}

fn registered() -> Registered {
    Registered {
        workspace: "lab".to_owned(),
        name: "laptop".to_owned(),
        grade: "operator".to_owned(),
        word: "K7M2QX".to_owned(),
        entry: std::path::PathBuf::from("/home/u/.local/share/lernie/wire/workspaces/lab"),
        addresses: vec!["engine.example:7737".to_owned()],
        proved: Ok(()),
    }
}

/// At rest the page says what it is for and shows the shape of a URL.
#[test]
fn at_rest_the_page_says_what_it_is_for() {
    let said = painted(&mut Model::default());
    for words in ["Joins this box to a server", HINT, REGISTER] {
        assert!(said.contains(words), "{words:?}:\n{said}");
    }
    assert!(ABOUT.contains("the key never leaves it"));
    assert!(!said.contains(CLAIMING) && !said.contains(WORD), "{said}");
}

/// **The page opens with nothing aimed, and the button fires the claim** for
/// what is typed — once: while it is out the page says so and the button is
/// dark.
#[test]
fn the_button_fires_the_claim_for_what_is_typed() {
    let mut model = Model {
        lookup: Some(Lookup::Settings(Page::Register)),
        ..Model::default()
    };
    model.registering.url = "https://engine.example/yog/register/ABCD2345".to_owned();
    let mut harness = crate::snapshot::seat(model, 1400.0, 900.0);
    harness.get_by_label(REGISTER).click();
    harness.run();
    assert_eq!(harness.state().registering.phase, Claim::Claiming);
    assert_eq!(
        harness.state().registering.asked.as_deref(),
        Some("https://engine.example/yog/register/ABCD2345")
    );
    assert!(harness.query_by_label(CLAIMING).is_some());
    harness.get_by_label(REGISTER).click();
    harness.run();
    assert_eq!(harness.state().registering.phase, Claim::Claiming);
}

/// With nothing typed the button is dark and a click fires nothing.
#[test]
fn nothing_typed_fires_nothing() {
    let model = Model {
        lookup: Some(Lookup::Settings(Page::Register)),
        ..Model::default()
    };
    let mut harness = crate::snapshot::seat(model, 1400.0, 900.0);
    harness.get_by_label(REGISTER).click();
    harness.run();
    assert_eq!(harness.state().registering.phase, Claim::Idle);
}

/// **A claim that succeeded says the sentences the command line says**, the
/// word on a line of its own: where the entry is, where to approve,
/// and that the server is waiting.
#[test]
fn a_registered_box_shows_the_word_and_where_to_approve_it() {
    let mut model = Model::default();
    model.registering.phase = Claim::Done(Ok(registered()));
    let said = painted(&mut model);
    for words in [
        "Confirmation word: K7M2QX",
        "Registered as laptop (operator) in workspace lab",
        "approve the offer whose row shows that word",
        "Waiting for approval",
    ] {
        assert!(said.contains(words), "{words:?}:\n{said}");
    }
}

/// A claim that failed says why, in the act's own sentence.
#[test]
fn a_refused_claim_says_why() {
    let mut model = Model::default();
    model.registering.phase = Claim::Done(Err(
        "this offer cannot be claimed: engine.example answered 404".to_owned(),
    ));
    let said = painted(&mut model);
    assert!(said.contains("this offer cannot be claimed"), "{said}");
}
