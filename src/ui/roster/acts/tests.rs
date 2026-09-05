//! The four controls on the strip: what each spends, and that they stand down
//! together under a covering pane.

use super::{REFRESH, render};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{click, pane, seated};
use crate::ui::{Model, commands, find, queue};

/// **All four are offered on a seat that has aimed at nothing**, which is the
/// seat most likely to be asking any of them.
#[test]
fn every_act_is_offered_with_nothing_aimed_at() {
    let mut model = Model::default();
    let painted = pane(|ui| render(ui, &mut model));
    for word in [REFRESH, queue::OPEN, commands::OPEN, find::OPEN] {
        assert!(painted.contains(word), "{word:?}:\n{painted}");
    }
}

/// **The refresh asks and opens nothing** — it is the affordance the roster's
/// own standing read owed itself.
#[test]
fn the_refresh_asks_every_channel_again_and_opens_no_pane() {
    let window = Window::new();
    let mut model = seated();
    click(&window, REFRESH, |ctx| crate::ui::render(ctx, &mut model));
    assert_eq!(
        model.outbox,
        vec![crate::ui::Posted::read(crate::verbs::workspaces())]
    );
    assert!(!model.covered(), "nothing was opened");
}

/// **The strip stands down under a covering pane**, exactly as the per-wall
/// controls do: what each opens would replace what is standing there.
#[test]
fn the_strip_stands_down_under_a_covering_pane() {
    let mut model = Model {
        listing: Some(crate::ui::Listing::Queue),
        ..seated()
    };
    let painted = crate::test_support::window::painted(&mut model);
    for word in [REFRESH, commands::OPEN, find::OPEN] {
        assert!(!painted.contains(word), "{word:?}:\n{painted}");
    }
}
