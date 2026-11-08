//! **The open bar's four** (DESIGN §4.48 item 5): `+ new` and the pages as
//! rows, the filter, the kept group, and the age separators — and the
//! collapsed bar showing none of them.

use super::{DESK, ROW, bar, click_bar, said, tick};
use crate::paint_probe::frame::{Window, press, screen_sized};
use crate::test_support::window::{click, conv, right_click, seated};
use crate::ui::bar::{rows, verbs};
use crate::ui::{Listing, Model, convs, keys, slash};

/// A seated model with the bar pinned open.
fn open() -> Model {
    let mut model = seated();
    model.bar.pinned = true;
    model
}

/// What the open bar says on one frame.
fn says(model: &mut Model) -> Vec<String> {
    said(&tick(&egui::Context::default(), model, 0.0, None))
}

/// **A page's row**, as the bar spells it.
fn page(word: &str) -> String {
    format!("{} {word}", verbs::PAGE_MARK)
}

/// **The pages are read off the menu**: `+ new` with its chord first, then
/// every page the menu offers and nothing else; collapsed, none of it, the
/// `+` alone standing for `+ new`.
#[test]
fn the_open_bar_lists_new_then_every_page_and_the_collapsed_bar_none() {
    let mut model = open();
    let seen = says(&mut model);
    let new = format!("{} {}", super::super::PLUS, verbs::NEW);
    let at = |text: &str| seen.iter().position(|run| run == text);
    let pages: Vec<&str> = slash::rows(&model)
        .into_iter()
        .filter(|row| row.page)
        .map(|row| row.word)
        .collect();
    assert_eq!(
        pages,
        ["fleet", "attention", "trail", "board", "prices"],
        "the menu's pages"
    );
    assert!(at(&new) < at(&page(pages[0])), "{seen:?}");
    assert!(seen.contains(&keys::chord_for(keys::Act::New)), "{seen:?}");
    for word in pages {
        assert!(at(&page(word)).is_some(), "{word}: {seen:?}");
    }
    assert!(!seen.contains(&page("records")), "not a page: {seen:?}");
    model.bar.pinned = false;
    let shut = says(&mut model);
    assert!(shut.contains(&super::super::PLUS.to_owned()), "{shut:?}");
    for gone in [page("attention"), new, rows::KEPT.to_owned()] {
        assert!(!shut.contains(&gone), "{gone}: {shut:?}");
    }
}

/// **A page's row fires its word** through the menu's own door, and the
/// attention row wears the waiting count while there is one.
#[test]
fn a_page_row_fires_its_word_and_attention_wears_the_count() {
    let mut model = open();
    model.roster[0].workspaces[0].attention = 3;
    assert!(says(&mut model).contains(&"3".to_owned()));
    click_bar(&Window::new(), &mut model, &page("attention"));
    assert_eq!(model.listing, Some(Listing::Queue));
}

/// `+ new`'s row begins a conversation on the aimed workspace.
#[test]
fn the_new_row_begins_a_conversation() {
    let mut model = open();
    let new = format!("{} {}", super::super::PLUS, verbs::NEW);
    click_bar(&Window::new(), &mut model, &new);
    assert_eq!(model.conversation, None, "the box takes the first message");
}

/// **The filter narrows by the rows' own words**, a word's start at a time,
/// says so when nothing is left, and Esc in it clears it.
#[test]
fn the_filter_narrows_the_rows_and_esc_clears_it() {
    let mut model = open();
    let mut other = conv("20260830T051300Z-c3d4", "tidy the docs");
    other.preview = "rewrite the readme".to_owned();
    model.convs.push(other);
    model.bar.sift = "READ".to_owned();
    let seen = says(&mut model);
    assert!(!seen.contains(&ROW.to_owned()), "{seen:?}");
    assert!(
        seen.iter().any(|run| run.contains("tidy the docs")),
        "{seen:?}"
    );
    model.bar.sift = "aint".to_owned();
    let seen = says(&mut model);
    assert!(seen.contains(&rows::NOTHING_MATCHES.to_owned()), "{seen:?}");
    let window = Window::new();
    let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
    window.frame(Vec::new(), &mut body);
    window.focus(egui::Id::new(keys::FILTER_ID));
    window.frame(vec![press(egui::Key::Escape)], &mut body);
    assert!(model.bar.sift.is_empty());
    assert!(verbs::sifted(&model.convs[0], " "), "blank narrows nothing");
}

/// **Shift+click keeps a row at the top under `PINNED`, and again lets it
/// go**; with nothing kept the group says how to keep one.
#[test]
fn shift_click_keeps_a_row_and_lets_it_go() {
    let ctx = egui::Context::default();
    let mut model = open();
    let seen = says(&mut model);
    assert!(seen.contains(&rows::KEPT_HINT.to_owned()), "{seen:?}");
    for (time, kept) in [(1.0, true), (2.0, false)] {
        let at = bar(&tick(&ctx, &mut model, time, None))
            .into_iter()
            .find(|run| run.text == ROW)
            .expect("the row")
            .shown
            .center();
        for (pressed, later) in [(true, 0.01), (false, 0.02)] {
            let mut input = screen_sized(DESK, 900.0);
            input.time = Some(time + later);
            input.modifiers = egui::Modifiers::SHIFT;
            input.events = vec![
                egui::Event::PointerMoved(at),
                egui::Event::PointerButton {
                    pos: at,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::SHIFT,
                },
            ];
            let _ = ctx.run(input, |ctx| crate::ui::render(ctx, &mut model));
        }
        assert_eq!(model.kept().is_empty(), !kept);
    }
    model.toggle_kept("20260830T051200Z-a1b2");
    let seen = says(&mut model);
    assert!(!seen.contains(&rows::KEPT_HINT.to_owned()), "{seen:?}");
    assert!(seen.contains(&ROW.to_owned()), "{seen:?}");
}

/// **The row's menu keeps it and lets it go**, in its own words.
#[test]
fn the_row_menu_pins_and_unpins() {
    let window = Window::new();
    let mut model = open();
    for word in [convs::KEEP, convs::UNKEEP] {
        let mut body = |ctx: &egui::Context| crate::ui::render(ctx, &mut model);
        right_click(&window, ROW, &mut body);
        click(&window, word, &mut body);
    }
    assert!(model.kept().is_empty());
}

/// **The rows part by age**: a heading over each stretch, in the listing's
/// own order, and none in a kept group or the collapsed bar.
#[test]
fn the_rows_part_by_age() {
    let mut model = open();
    let mut old = conv("20260830T051300Z-c3d4", "an old one");
    old.age_secs = 3 * 366 * 86_400;
    model.convs.push(old);
    let seen = says(&mut model);
    let today = convs::epoch::TODAY.to_uppercase();
    let past = convs::epoch::epoch(convs::epoch::now(), 3 * 366 * 86_400).to_uppercase();
    let at = |text: &str| seen.iter().position(|run| run == text);
    assert!(at(&today) < at(ROW), "{seen:?}");
    assert!(at(ROW) < at(&past), "{seen:?}");
    model.bar.pinned = false;
    assert!(!says(&mut model).contains(&today));
}
