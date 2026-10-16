//! **The bar's foot is the workspace** (DESIGN §4.44): the aimed wall's name
//! with `⏶`, elided at the bar's width collapsed, `workspace ⏶` with nothing
//! aimed — and the list a click opens, standing upward from it.

use super::{click_bar, said, tick};
use crate::paint_probe::frame::Window;
use crate::test_support::window::{dropped, seated};
use crate::ui::{Model, Selector};

/// **The foot's words**: the wall's name and the selector's mark, whole
/// where the bar holds it and elided with `…` where it does not (bl-773e) —
/// a longer name shows more of itself at twice the collapsed width — and
/// `workspace` with nothing aimed.
#[test]
fn the_foot_names_the_aimed_wall_or_the_word() {
    let ctx = egui::Context::default();
    let mut model = seated();
    let shut = said(&tick(&ctx, &mut model, 0.0, None));
    assert!(
        shut.contains(&dropped("home")),
        "the mark in the door, a short name whole: {shut:?}"
    );
    let long = "a workspace with a long name";
    let foot = |model: &mut Model| {
        let ctx = egui::Context::default();
        said(&tick(&ctx, model, 0.0, None))
            .into_iter()
            .find(|t| t.starts_with("a w"))
            .expect("the foot")
    };
    let mut named = seated();
    named.aim.as_mut().expect("aimed").address = long.to_owned();
    let cut = foot(&mut named);
    let tail = format!("{}{}", crate::ui::theme::glyph::ELIDED, dropped(""));
    assert!(cut.ends_with(&tail), "{cut:?}");
    named.settings.bar_shut = Some(super::super::SHUT * 2.0);
    let more = foot(&mut named);
    assert!(more.len() > cut.len(), "{more:?} shows more than {cut:?}");
    model.bar.pinned = true;
    let open = said(&tick(&ctx, &mut model, 0.0, None));
    assert!(open.contains(&dropped("home")), "{open:?}");
    let mut nowhere = Model::default();
    nowhere.bar.pinned = true;
    let unaimed = said(&tick(&ctx, &mut nowhere, 0.0, None));
    let word = crate::ui::status::selectors::NO_WORKSPACE;
    assert!(unaimed.contains(&dropped(word)), "{unaimed:?}");
}

/// **A click opens the workspace's list upward from the foot** — every wall
/// and `+ new workspace` last, above the foot and over the column — and
/// posts the read that refreshes it; a pick aims, and a second click closes.
#[test]
fn a_click_opens_the_list_above_the_foot_and_a_pick_aims() {
    let window = Window::sized(super::DESK, 900.0);
    let mut model = seated();
    model.bar.pinned = true;
    model.roster[0]
        .walls
        .push(crate::test_support::window::wall("annex"));
    click_bar(&window, &mut model, &dropped("home"));
    assert_eq!(
        model.dropup.as_ref().map(|o| o.selector),
        Some(Selector::Workspace)
    );
    assert_eq!(
        model.outbox.last().map(|p| p.envelope.clone()),
        Some(crate::verbs::workspaces())
    );
    let seen = crate::test_support::window::seen(&window, |ctx| crate::ui::render(ctx, &mut model));
    let foot = seen
        .iter()
        .filter(|run| run.text == format!("home {}", crate::ui::theme::glyph::RELOAD))
        .max_by(|a, b| a.laid.min.y.total_cmp(&b.laid.min.y))
        .expect("the foot");
    let found = crate::ui::roster::engine::FOUND;
    for row in ["annex", found] {
        let run = seen
            .iter()
            .find(|run| run.text == row)
            .unwrap_or_else(|| panic!("{row:?} on the list"));
        assert!(
            run.laid.max.y <= foot.laid.min.y,
            "{row:?} stands above the foot"
        );
    }
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    crate::test_support::window::click(&window, "annex", &mut body);
    assert_eq!(
        model.aim.as_ref().map(|aim| aim.address.as_str()),
        Some("annex")
    );
    assert!(model.dropup.is_none(), "a pick closes it");
    click_bar(&window, &mut model, &dropped("annex"));
    assert!(model.dropup.is_some());
    let reloading = format!("annex {}", crate::ui::theme::glyph::RELOAD);
    click_bar(&window, &mut model, &reloading);
    assert!(model.dropup.is_none(), "a second click closes it");
}

/// **`+ new workspace` is the list's last row** under this box's own engine
/// (bl-673e): a pick begins the founding, the box asking for `<name> <goal>`.
#[test]
fn new_workspace_on_the_list_begins_the_founding() {
    let window = Window::sized(super::DESK, 900.0);
    let mut model = seated();
    click_bar(&window, &mut model, &dropped("home"));
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    let found = crate::ui::roster::engine::FOUND;
    crate::test_support::window::click(&window, found, &mut body);
    assert_eq!(model.founding.as_deref(), Some(crate::seat::OWN));
    assert_eq!(model.conversation, None);
    let mut unheld = seated();
    unheld.roster[0].held = crate::ui::Held::Unheld("no".to_owned());
    unheld.drop_up(Selector::Workspace);
    assert!(
        !unheld.choices().iter().any(|choice| choice.text == found),
        "an engine that will not dial founds nothing"
    );
}

/// **The foot wears the read its list posts**, as the rule's segment did.
#[test]
fn the_foot_is_a_tagged_link() {
    use egui_kittest::kittest::Queryable;
    let mut model = seated();
    model.bar.pinned = true;
    let harness = crate::snapshot::seat(model, super::DESK, 900.0);
    let token = format!(
        "{}{}",
        crate::ui::act::PREFIX,
        crate::verbs::WORKSPACES.word
    );
    assert!(
        harness
            .query_all_by_label("home")
            .any(|node| node.author_id() == Some(token.as_str())),
        "the foot is the workspaces read's door"
    );
}
