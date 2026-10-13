//! **A selector's dropup on the glass** (bl-d74b): the cached rows at once,
//! the reload mark until the read answers and the check after, the keys, a
//! pick's envelope, and every way it closes.

use super::doors::click_rule;
use crate::paint_probe::frame::{Window, click, press};
use crate::reply::{Read, Reply};
use crate::test_support::window::{own, selecting};
use crate::ui::{Model, Selector, theme};
use egui_kittest::kittest::Queryable;

/// Every run one idle frame put on the glass.
fn glass(window: &Window, model: &mut Model) -> Vec<crate::paint_probe::Seen> {
    crate::test_support::window::seen(window, |ctx| crate::ui::render(ctx, model))
}

/// Whether `glyph` stands on the glass in `ink`.
fn wears(seen: &[crate::paint_probe::Seen], glyph: &str, ink: egui::Color32) -> bool {
    seen.iter().any(|run| run.text == glyph && run.ink == ink)
}

/// **The cached rows paint before the read answers, under the reload mark in
/// the annotation accent; the answer turns it into the check in the
/// attention accent.**
#[test]
fn the_cached_list_stands_under_the_reload_until_the_answer_lands() {
    let window = Window::new();
    let mut model = selecting();
    click_rule(&window, &mut model, "housevendor");
    assert_eq!(
        model.outbox.last().map(|p| p.envelope.clone()),
        Some(crate::verbs::providers("home".to_owned()))
    );
    let seen = glass(&window, &mut model);
    let texts: Vec<&str> = seen.iter().map(|run| run.text.as_str()).collect();
    assert!(texts.contains(&"othervendor"), "{texts:?}");
    assert!(
        texts.contains(&"• housevendor"),
        "the row in force: {texts:?}"
    );
    let annotation = theme::accent(theme::State::Annotation);
    assert!(wears(&seen, theme::glyph::RELOAD, annotation), "{texts:?}");
    model.absorb(&own().channel, Read::Answer(Reply::Providers(Vec::new())));
    let seen = glass(&window, &mut model);
    let attention = theme::accent(theme::State::Attention);
    assert!(wears(&seen, theme::glyph::DONE, attention));
    assert!(!wears(&seen, theme::glyph::RELOAD, annotation));
    assert!(
        seen.iter()
            .any(|run| run.text == super::super::dropup::NOTHING)
    );
}

/// **↑↓ walk, Enter picks**, and the pick is the role tuning's own envelope
/// for the worker — the prompt neither sends nor takes the keys.
#[test]
fn the_keys_walk_and_pick() {
    let window = Window::new();
    let mut model = selecting();
    model.draft = "half a thought".to_owned();
    model.drop_up(Selector::Effort);
    window.focus(egui::Id::new(crate::ui::keys::BOX_ID));
    for key in [
        egui::Key::ArrowDown,
        egui::Key::ArrowUp,
        egui::Key::ArrowDown,
        egui::Key::Enter,
    ] {
        window.frame(vec![press(key)], |ctx| crate::ui::render(ctx, &mut model));
    }
    let high = crate::verbs::effort(
        "home".to_owned(),
        "worker".to_owned(),
        Some("high".to_owned()),
    );
    assert_eq!(model.outbox.last().map(|p| p.envelope.clone()), Some(high));
    assert!(model.dropup.is_none());
    assert_eq!(model.draft, "half a thought", "the box sent nothing");
}

/// **A click on a row picks it**, and the row wears the op it spends.
#[test]
fn a_click_on_a_row_picks_it() {
    let window = Window::new();
    let mut model = selecting();
    click_rule(&window, &mut model, "priority off");
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    crate::test_support::window::click(&window, "on", &mut body);
    let on = crate::verbs::priority("home".to_owned(), "worker".to_owned(), true);
    assert_eq!(model.outbox.last().map(|p| p.envelope.clone()), Some(on));
    let harness = crate::snapshot::seat(
        {
            let mut open = selecting();
            open.drop_up(Selector::Model);
            open
        },
        1400.0,
        900.0,
    );
    let row = harness
        .query_all_by_label("house-model-2")
        .next()
        .expect("the row is on the glass");
    let token = format!("{}{}", crate::ui::act::PREFIX, crate::verbs::MODEL.word);
    assert_eq!(row.author_id(), Some(token.as_str()));
}

/// **Esc closes it, a click elsewhere closes it, and an overlay or the
/// completion menu puts it down.**
#[test]
fn it_closes_on_esc_elsewhere_and_under_what_floats() {
    let window = Window::new();
    let mut model = selecting();
    model.drop_up(Selector::Workspace);
    crate::test_support::window::escape(&window, |ctx| crate::ui::render(ctx, &mut model));
    assert!(model.dropup.is_none(), "Esc");
    model.drop_up(Selector::Workspace);
    glass(&window, &mut model);
    click(&window, egui::pos2(600.0, 200.0), |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(model.dropup.is_none(), "a click elsewhere");
    model.drop_up(Selector::Workspace);
    model.draft = "/".to_owned();
    glass(&window, &mut model);
    assert!(model.dropup.is_none(), "the menu");
    model.draft.clear();
    model.drop_up(Selector::Workspace);
    model.switching = true;
    glass(&window, &mut model);
    assert!(model.dropup.is_none(), "an overlay");
    model.switching = false;
    model.drop_up(Selector::Workspace);
    model.escape();
    assert!(model.dropup.is_none(), "the ladder's first rung");
}

/// **The waiting count left the rule for the seat panel and the switcher's
/// head**, where it is the queue's door.
#[test]
fn the_waiting_count_is_the_queue_s_door_off_the_rule() {
    let asking = || {
        let mut model = crate::test_support::window::seated();
        model.conversation = None;
        model.roster[0].walls[0].attention = 2;
        model
    };
    let mut model = asking();
    crate::test_support::window::click(&Window::new(), "2 waiting", |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(
        model.showing(crate::ui::Listing::Queue),
        "from the seat panel"
    );
    let mut model = Model {
        conversation: crate::test_support::window::seated().conversation,
        ..asking()
    };
    model.switching = true;
    crate::test_support::window::click(&Window::new(), "2 waiting", |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert!(
        model.showing(crate::ui::Listing::Queue) && !model.switching,
        "from the switcher"
    );
    let mut down = asking();
    down.roster[0].held = crate::ui::Held::Unheld("no".to_owned());
    assert_eq!(
        super::super::waiting::waiting(&down),
        0,
        "a dead engine counts nothing"
    );
}
