//! One pane, two homes: the query, the cover, the standing set and the
//! one-overlay rule where the home changes under two standing panes.

use crate::settings::Mode;
use crate::state::{Open, Standing};
use crate::test_support::window::{configured, recorded, seated, trailing};
use crate::ui::{Context, Model, Rail};

/// The seated model in the advanced mode with room.
pub(crate) fn docked() -> Model {
    let mut model = Model {
        room: true,
        ..seated()
    };
    model.settings.mode = Mode::Advanced;
    model
}

/// **The home is the mode AND the room**: advanced without room floats, as
/// simple does with it.
#[test]
fn docked_is_the_mode_with_room() {
    let mut model = seated();
    assert!(!model.docked());
    model.room = true;
    assert!(!model.docked(), "simple floats whatever the room");
    model.settings.mode = Mode::Advanced;
    assert!(model.docked());
    model.room = false;
    assert!(!model.docked(), "no room, no rail");
}

/// **A docked pane stands and is no cover; a floating one is** — and each
/// rail answers for its own pane.
#[test]
fn a_rail_stands_for_its_pane_and_covers_nothing_docked() {
    let mut model = docked();
    assert!(!model.standing_on(Rail::Branch) && !model.standing_on(Rail::Context));
    model.begin_trail();
    model.begin_records();
    assert!(model.railed(Rail::Branch) && model.railed(Rail::Context));
    assert!(!model.covered());
    model.settings.mode = Mode::Simple;
    assert!(!model.railed(Rail::Branch), "the same pane, floating");
    assert!(model.standing_on(Rail::Branch));
    assert!(model.covered());
}

/// **The context rail holds one of two**, and a close names its own: a
/// stale records close cannot take the config pane down.
#[test]
fn the_context_rail_holds_one_pane_and_a_close_names_its_own() {
    let mut model = seated();
    model.begin_records();
    assert!(model.recording());
    model.begin_configuring();
    assert!(!model.recording() && model.configuring().is_some());
    model.close_records();
    assert!(model.configuring().is_some(), "the wrong word does nothing");
    model.close_configuring();
    assert_eq!(model.context, None);
    model.begin_records();
    model.close_configuring();
    assert!(model.recording());
}

/// **Two docked panes may not both float**: when the home goes — the mode
/// set back, the window narrowed — the branch rail's yields and the
/// context stays, so one overlay stands as the rule says.
#[test]
fn losing_the_home_under_two_panes_keeps_one() {
    let mut model = docked();
    model.begin_trail();
    model.begin_records();
    model.housed(true);
    assert!(
        model.trailing() && model.recording(),
        "room kept, both stand"
    );
    model.housed(false);
    assert!(!model.trailing() && model.recording());
    assert!(model.covered());
    let mut again = docked();
    again.begin_trail();
    again.begin_configuring();
    again.settings.mode = Mode::Simple;
    again.housed(true);
    assert!(!again.trailing() && again.configuring().is_some());
    let mut alone = docked();
    alone.begin_trail();
    alone.housed(false);
    assert!(alone.trailing(), "one pane floats as it is");
}

/// **Every standing read keys on being shown, wherever shown**: docked, the
/// trail's and the records' stand beside the floating pane's.
#[test]
fn the_standing_set_holds_every_shown_pane() {
    let mut model = docked();
    model.begin_trail();
    model.begin_records();
    model.begin_tuning();
    let standing = Standing::of(&model);
    assert_eq!(
        standing.open,
        vec![Open::Tuning(model.tuned()), Open::Records, Open::Trail]
    );
    assert!(standing.standing(&Open::Trail) && standing.standing(&Open::Records));
    assert_eq!(standing.tuned(), Some(&model.tuned()));
    assert_eq!(standing.signin(), None);
    assert_eq!(Standing::of(&seated()).open, Vec::new());
    assert_eq!(Standing::of(&seated()).tuned(), None);
    let config = Standing::of(&configured());
    assert_eq!(config.open.len(), 1);
    assert_eq!(config.at(), configured().configured());
}

/// **Docked, the config pane is no page of the settings overlay**: the
/// overlay opens on its own page beside it, and Esc takes the overlay first.
#[test]
fn docked_the_config_pane_is_the_rails_and_not_a_settings_page() {
    let mut model = docked();
    model.begin_configuring();
    assert_eq!(model.page(), None);
    assert!(model.turn_to(crate::ui::Page::default()));
    assert_eq!(model.page(), Some(crate::ui::Page::default()));
    assert!(model.configuring().is_some(), "the rail stood through it");
    model.escape();
    assert_eq!(model.page(), None);
    assert!(model.configuring().is_some());
    model.escape();
    assert_eq!(model.context, None);
    let floating = configured();
    assert_eq!(floating.page(), Some(crate::ui::Page::Config));
}

/// The fixtures stand on the fields the rails read.
#[test]
fn the_fixtures_stand_on_the_rails_fields() {
    assert!(trailing().trailing());
    assert_eq!(recorded().context, Some(Context::Records));
    assert_eq!(Rail::Branch.name(), "the branch rail");
    assert_eq!(Rail::Context.name(), "the context rail");
}
