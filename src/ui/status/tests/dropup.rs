//! **A selector's dropup on the glass** (bl-d74b): the cached rows at once,
//! the reload mark until the read answers and the check after, the keys, a
//! pick's envelope, and every way it closes.

use super::doors::click_rule;
use crate::paint_probe::frame::{Window, click, press};
use crate::reply::{Read, Reply};
use crate::test_support::window::{dropped, own, selecting};
use crate::ui::{Model, Selector, theme};
use egui_kittest::kittest::Queryable;

/// Every run one idle frame put on the glass.
fn glass(window: &Window, model: &mut Model) -> Vec<crate::paint_probe::Seen> {
    crate::test_support::window::seen(window, |ctx| crate::ui::render(ctx, model))
}

/// **Whether the provider's door ends in `glyph`, and its run wears it in
/// `ink`** — the glass says the mark inside the door (DESIGN §4.44 item 5),
/// and the run the door is painted from says the mark's own ink.
fn wears(
    model: &Model,
    seen: &[crate::paint_probe::Seen],
    glyph: &str,
    ink: egui::Color32,
) -> bool {
    let run = &crate::ui::status::runs(model).0[0];
    let door = format!("{} {glyph}", run.text);
    run.mark == Some((glyph, ink)) && seen.iter().any(|seen| seen.text == door)
}

/// **The cached rows paint before the read answers, under the reload mark in
/// the annotation accent; the answer turns it into the check in the
/// attention accent.**
#[test]
fn the_cached_list_stands_under_the_reload_until_the_answer_lands() {
    let window = Window::new();
    let mut model = selecting();
    click_rule(&window, &mut model, &dropped("housevendor"));
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
    assert!(
        wears(&model, &seen, theme::glyph::RELOAD, annotation),
        "{texts:?}"
    );
    model.absorb(&own().channel, Read::Answer(Reply::Providers(Vec::new())));
    let seen = glass(&window, &mut model);
    let attention = theme::accent(theme::State::Attention);
    assert!(wears(&model, &seen, theme::glyph::DONE, attention));
    assert!(!wears(&model, &seen, theme::glyph::RELOAD, annotation));
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
    click_rule(&window, &mut model, &dropped("priority off"));
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
        model.roster[0].workspaces[0].attention = 2;
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
        "a dead server counts nothing"
    );
}

/// **The list is as wide as its longest row** (bl-8d6d): a long model id
/// stands whole at the desk, never cut with `…` where there is room.
#[test]
fn a_long_model_id_stands_whole_at_the_desk() {
    let long = "house-model-with-a-long-identifier-2026-10-01-preview-extended-context";
    let mut model = selecting();
    model
        .offers
        .insert("housevendor".to_owned(), vec![long.to_owned()]);
    model.drop_up(Selector::Model);
    let window = Window::sized(1400.0, 900.0);
    window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
    let seen = glass(&window, &mut model);
    let row = seen
        .iter()
        .find(|run| run.text.starts_with("house-model-with"))
        .expect("the long row is on the glass");
    assert_eq!(row.text, long, "cut: {:?}", row.text);
}

/// **The list's foot stands on the rule and the list floats over the box**
/// (DESIGN §4.45 item 3), at the desk and at the phone: every row of it
/// stands above the rule's line, and the lowest of them below the box's top
/// — the rows cover the box, not the transcript's foot.
#[test]
fn the_list_stands_on_the_rule_over_the_box() {
    for (w, h) in [(1400.0, 900.0), (400.0, 800.0)] {
        let mut model = selecting();
        model.drop_up(Selector::Model);
        let window = Window::sized(w, h);
        window.frame(Vec::new(), |ctx| crate::ui::render(ctx, &mut model));
        let seen = glass(&window, &mut model);
        let lowest = |pick: &dyn Fn(&crate::paint_probe::Seen) -> bool| {
            seen.iter()
                .filter(|run| pick(run))
                .map(|run| run.laid)
                .max_by(|a, b| a.min.y.total_cmp(&b.min.y))
        };
        let rule = lowest(&|run| run.text == crate::ui::status::LEAD).expect("the rule");
        let prompt = lowest(&|run| run.text == theme::glyph::PROMPT && run.ink == theme::BRAND)
            .expect("the box");
        let rows: Vec<_> = seen
            .iter()
            .filter(|run| run.text == "• house-model-1" || run.text == "house-model-2")
            .map(|run| run.laid)
            .collect();
        assert_eq!(rows.len(), 2, "at {w}: both rows are on the glass");
        for row in &rows {
            assert!(
                row.max.y <= rule.min.y + 1.0,
                "at {w}: the row is above the rule"
            );
        }
        let foot = rows.iter().map(|row| row.max.y).fold(0.0, f32::max);
        assert!(foot > prompt.min.y, "at {w}: the list covers the box");
        let hint = lowest(&|run| run.text == super::super::dropup::HINT).expect("the hint");
        assert!(
            hint.max.y <= rows[0].min.y + 1.0,
            "at {w}: the keys stand over the rows"
        );
    }
}
