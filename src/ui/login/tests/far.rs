//! A catalog row named `<row>@<client>` (yog's `docs/REMOTE.md` §5.7): the
//! sentence its machine's heading carries, and the qualified name both of its
//! controls put on the wire.

use super::super::{OFFERS, SIGN_IN, far, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, provider, seated};
use crate::ui::{Login, Model, Posted};
use serde_json::json;

/// The pane open on a catalog of one row, `name`.
fn listing(name: &str) -> Model {
    Model {
        catalog: Some(vec![provider(name)]),
        login: Some(Login::default()),
        ..seated()
    }
}

/// **A client's heading says its browser opens on that machine**, once, under
/// the machine's name — and the server's own rows say no such thing.
#[test]
fn a_row_on_another_machine_says_its_browser_opens_there() {
    let mut model = listing("housevendor@attic");
    let painted = pane(|ui| {
        render(ui, &mut model);
    });
    let said = far("attic");
    assert!(said.contains("opens it on attic"), "{said}");
    assert_eq!(painted.matches(&said).count(), 1, "{painted}");
    let lines: Vec<&str> = painted.lines().collect();
    let at = |text: &str| {
        lines
            .iter()
            .position(|line| *line == text)
            .expect("painted")
    };
    let (heading, sentence, row) = (at("attic"), at(&said), at("housevendor"));
    assert!(heading < sentence && sentence < row, "{painted}");
    let mut own = listing("housevendor");
    let painted = pane(|ui| {
        render(ui, &mut own);
    });
    assert!(!painted.contains("opens it on"), "{painted}");
}

/// **Both controls send the qualified name**: the row is painted without its
/// machine, and the sign-in, the lane it stands up and the offering are all
/// addressed by the catalog's own spelling.
#[test]
fn a_row_on_another_machine_is_signed_in_to_and_asked_by_its_qualified_name() {
    for (label, expected) in [
        (
            SIGN_IN,
            Posted::act(
                json!({"op": "login", "workspace": "home", "provider": "housevendor@attic"}),
            ),
        ),
        (
            OFFERS,
            Posted::read(
                json!({"op": "models", "workspace": "home", "provider": "housevendor@attic"}),
            ),
        ),
    ] {
        let mut model = listing("housevendor@attic");
        let window = Window::new();
        click(&window, label, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render(ui, &mut model);
            });
        });
        assert_eq!(model.outbox, vec![expected], "{label}");
        let pane = model.login.clone().expect("the pane");
        let held = pane.following.or(pane.asking);
        assert_eq!(held.as_deref(), Some("housevendor@attic"), "{label}");
    }
    let mut model = listing("housevendor@attic");
    model.post_signin("housevendor@attic");
    assert_eq!(model.following().as_deref(), Some("housevendor@attic"));
}
