//! The row's menu, driven by a real secondary click: that it opens, what it
//! offers, which items fire and which lead somewhere — and that every item
//! that crosses the boundary carries its `act:` token.

use super::{leads_to, straight};
use crate::paint_probe::frame::Window;
use crate::reply::convs::ConvRow;
use crate::test_support::window::{click, conv, right_click, seated};
use crate::ui::composer::acts::{DELETE, FLAG, RETARGET, STOP};
use crate::ui::{Column, Model, convs, keys, queue, records};
use egui_kittest::Harness;
use egui_kittest::kittest::{Queryable, by};

/// The conversation the row menu is opened on: one that is **asking**, so the
/// convenience `seen` is offered too.
fn asking() -> ConvRow {
    ConvRow {
        attention: 1,
        ..conv("20260830T051200Z-a1b2", "one")
    }
}

/// **The seat standing on the conversations column in the narrow shape.**
///
/// That width is chosen rather than convenient: the composer stands down off
/// the conversation's own column there (`crate::ui::shell`), so every word the
/// assertions below look for is the menu's own rather than the composer second
/// row's under another name.
fn narrow() -> Model {
    Model {
        column: Column::Conversations,
        convs: vec![asking()],
        ..seated()
    }
}

/// The window the narrow shape is judged in, and the row's painted headline.
fn row() -> (Window, String) {
    (Window::sized(400.0, 800.0), convs::headline(&asking()))
}

/// **The menu is not on the glass until a secondary click asks for it**, and
/// then every item is.
#[test]
fn a_secondary_click_opens_the_row_s_acts_and_nothing_else_does() {
    let (window, label) = row();
    let mut model = narrow();
    let quiet = window.text(|ctx| crate::ui::render(ctx, &mut model));
    assert!(!quiet.contains(&leads_to(DELETE)), "{quiet}");
    assert!(!quiet.contains(STOP), "{quiet}");

    right_click(&window, &label, |ctx| crate::ui::render(ctx, &mut model));
    let open = window.text(|ctx| crate::ui::render(ctx, &mut model));
    for word in [STOP, RETARGET, queue::SEEN, records::OPEN] {
        assert!(open.contains(word), "{word:?} is on the menu: {open}");
    }
    for word in [FLAG, DELETE] {
        assert!(
            open.contains(&leads_to(word)) && !open.contains(&format!("{word} ")),
            "{word:?} leads somewhere and says so: {open}"
        );
    }
}

/// **A row that is not asking is not offered the answer to a question nobody
/// asked** — the one convenience this menu adds, offered where the row itself
/// says it applies.
#[test]
fn seen_rides_only_on_a_row_that_is_asking() {
    let words = |row: &ConvRow| -> Vec<&'static str> {
        straight(row).into_iter().map(|(word, _, _)| word).collect()
    };
    assert_eq!(words(&asking()), vec![STOP, RETARGET, queue::SEEN]);
    assert_eq!(words(&conv("id", "quiet")), vec![STOP, RETARGET]);
}

/// **The three that fire are addressed off the ROW and take no selection.**
///
/// This is `crate::ui::queue`'s division one pane over: a queue row's `seen`
/// answers that row without selecting it, and only *go to it* moves the focus.
/// A secondary click that killed a driver must not throw away the transcript
/// the operator was reading.
#[test]
fn a_fired_item_composes_the_row_s_own_gesture_and_moves_no_selection() {
    for (word, op) in [
        (STOP, crate::verbs::STOP.word),
        (RETARGET, crate::verbs::RETARGET.word),
        (queue::SEEN, crate::verbs::SEEN.word),
    ] {
        let (window, label) = row();
        let mut model = Model {
            conversation: Some("something else".to_owned()),
            ..narrow()
        };
        right_click(&window, &label, |ctx| crate::ui::render(ctx, &mut model));
        click(&window, word, |ctx| crate::ui::render(ctx, &mut model));
        let posted = model.outbox.first().unwrap_or_else(|| {
            panic!("{word:?} composed a gesture");
        });
        assert!(posted.act, "{word:?} changes the world");
        assert_eq!(posted.envelope["op"], op);
        assert_eq!(posted.envelope["workspace"], "home");
        assert_eq!(posted.envelope["agent"], asking().root_id);
        assert_eq!(
            model.conversation.as_deref(),
            Some("something else"),
            "{word:?} took no selection"
        );
    }
}

/// **`records…` opens the pane on the row it was asked about**, which is why
/// this one selects: the pane's subject is the selected conversation, and a
/// place opened about nothing would be a place about the wrong thing.
#[test]
fn the_records_item_selects_the_row_and_opens_the_pane_on_it() {
    let (window, label) = row();
    let mut model = Model {
        conversation: Some("something else".to_owned()),
        ..narrow()
    };
    right_click(&window, &label, |ctx| crate::ui::render(ctx, &mut model));
    click(&window, records::OPEN, |ctx| {
        crate::ui::render(ctx, &mut model);
    });
    assert_eq!(
        model.conversation.as_deref(),
        Some(asking().root_id.as_str())
    );
    assert!(
        model.showing(crate::ui::Listing::Records),
        "the pane is open"
    );
    assert!(model.outbox.is_empty(), "opening a pane composes nothing");
}

/// **The two that need words open the box and do not fire**, end to end: after
/// the item, the cursor is in that box, the column carrying the composer is on
/// the glass, and nothing has been asked of any engine.
#[test]
fn flag_and_delete_land_the_cursor_in_their_box_and_ask_for_nothing() {
    for (word, id) in [(FLAG, keys::REASON_ID), (DELETE, keys::ARM_ID)] {
        let (window, label) = row();
        let mut model = narrow();
        right_click(&window, &label, |ctx| crate::ui::render(ctx, &mut model));
        click(&window, &leads_to(word), |ctx| {
            crate::ui::render(ctx, &mut model);
        });
        // Two idle frames: in the narrow shape the list is the central panel
        // and the composer is already behind it, so the cursor lands on the
        // frame after the one the item was clicked in.
        for _ in 0..2 {
            window.text(|ctx| crate::ui::render(ctx, &mut model));
        }
        assert_eq!(model.column, Column::Conversation, "{word:?}");
        assert_eq!(
            model.conversation.as_deref(),
            Some(asking().root_id.as_str()),
            "{word:?}"
        );
        assert!(model.outbox.is_empty(), "{word:?} fired nothing");
        assert_eq!(
            window.focused(),
            Some(egui::Id::new(id)),
            "{word:?} put the cursor in its box"
        );
    }
}

/// **Every item that crosses the boundary carries its token**, read off the
/// real accessibility tree the parity gate reads (§4.16).
///
/// It is asserted as a **count** rather than as presence, because `stop`,
/// `retarget` and the records pane's two reads are already tagged on the
/// composer's second row and a set cannot tell one control from two. `seen`
/// is the one that is nowhere else on this screen, so it goes from nothing to
/// something and the two readings agree.
#[test]
fn the_menu_s_acts_carry_their_tags_on_top_of_the_composer_s() {
    let mut harness = crate::snapshot::seat(narrow(), 1400.0, 900.0);
    let before = counts(&harness);
    let aimed = at(&harness, &convs::headline(&asking()));
    secondary(&mut harness, aimed);
    let after = counts(&harness);
    for (op, was, now) in [
        (crate::verbs::STOP.word, before.0, after.0),
        (crate::verbs::SEEN.word, before.1, after.1),
        (crate::verbs::STEPS.word, before.2, after.2),
    ] {
        assert_eq!(now, was + 1, "{op:?} gained the menu's control");
    }
    assert_eq!(before.1, 0, "`seen` is on no other control of this screen");
}

/// The three counts the assertion above reads, in one pass: `stop`, `seen`,
/// `steps`.
fn counts(harness: &Harness<'_, Model>) -> (usize, usize, usize) {
    let tally = |op: &str| {
        let token = format!("{}{op}", crate::ui::act::PREFIX);
        harness
            .query_all(by())
            .filter(|node| !node.is_hidden())
            .filter_map(|node| node.author_id().map(str::to_owned))
            .filter(|author| author.split_whitespace().any(|held| held == token))
            .count()
    };
    (
        tally(crate::verbs::STOP.word),
        tally(crate::verbs::SEEN.word),
        tally(crate::verbs::STEPS.word),
    )
}

/// **Where the row is, off the same accessibility tree this test reads its
/// tokens from.**
///
/// The paint probe is the other instrument's aim (`crate::test_support::window`
/// locates by painted glyphs), and a coordinate read off one context is a
/// coordinate about another window. Here the tree is already the subject, and
/// `crate::snapshot::blank` reads node rectangles the same way.
fn at(harness: &Harness<'_, Model>, label: &str) -> egui::Pos2 {
    let bounds = harness
        .query_all(by())
        .filter(|node| node.label().or_else(|| node.value()).as_deref() == Some(label))
        .find_map(|node| node.bounding_box())
        .expect("the row is in the accessibility tree");
    egui::pos2(
        whole(f64::midpoint(bounds.x0, bounds.x1)),
        whole(f64::midpoint(bounds.y0, bounds.y1)),
    )
}

/// The whole point at or before `v`, **found rather than cast**: the house lint
/// set denies a lossy numeric conversion and the only home for a suppression is
/// the manifest, so this is `crate::snapshot::blank::whole`'s route with the
/// window's own width as the bound.
fn whole(v: f64) -> f32 {
    const BOUND: u16 = 4096;
    f32::from(
        (0..=BOUND)
            .rev()
            .find(|&n| f64::from(n) <= v)
            .unwrap_or_default(),
    )
}

/// One secondary click at `at`, in the harness the parity gate walks: move,
/// press, release, settle. The paint probe's own `secondary` drives the other
/// instrument; this is the same three events into this one.
fn secondary(harness: &mut Harness<'_, Model>, at: egui::Pos2) {
    harness
        .input_mut()
        .events
        .push(egui::Event::PointerMoved(at));
    harness.step();
    for pressed in [true, false] {
        harness.input_mut().events.push(egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Secondary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
    }
    harness.run();
}
