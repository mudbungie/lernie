//! **The open bar's verb rows and its filter** (DESIGN §4.48 item 5): the
//! baseline's pages under *New session*, and its *Search sessions* — read
//! here off what the seat already has rather than authored a second time.
//!
//! **`+ new` leads**, the head's door re-laid as a row with its chord as the
//! row's aside; then **every page of the completion menu**
//! (`crate::ui::slash::Row::page`), each a door firing its `/word` through
//! the menu's own door (`crate::ui::slash::fire`), wearing the word's tags
//! and its summary as its help. A page added to the menu is a row here with
//! no second edit, and a word is a word: no row gains a chord for being one.
//!
//! **The filter narrows the bar's rows on this seat**, by their own words —
//! `display`, `name` and the preview — with the menu's matcher
//! (`crate::ui::slash::narrows`). It is not `/find`, which asks every server
//! to search content and answers in its own pane, so its hint says *filter*
//! and its help says so. Esc in it clears it.

use crate::ui::keys::{self, Act};
use crate::ui::{Model, slash, theme};

/// The filter's hint.
pub const FILTER: &str = "filter";
/// What the filter does, and what it is not.
pub const FILTER_HELP: &str = "narrows this bar's conversations by their words, here on this seat — /find searches the servers";
/// The `+ new` row's word, after [`super::PLUS`].
pub const NEW: &str = "new";
/// The glyph a page's row leads with: the menu's own lead, since the row
/// fires the menu's word.
pub const PAGE_MARK: &str = "/";

/// **`+ new`, the first row under the head** while a workspace is aimed and
/// the box can take the caret: `+ new` with its chord at the right end open,
/// `+` alone collapsed — one seat under one id in both, so a pointer that
/// opens the bar on it is still on it. Its tags are the start's two ops, and
/// a click begins a conversation on the aimed workspace.
pub(super) fn new(ui: &mut egui::Ui, model: &mut Model, open: bool) {
    let Some(aim) = model.aim.clone() else {
        return;
    };
    if model.covered() && !model.switching {
        return;
    }
    let help = keys::cited(super::NEW_HELP, &[Act::New]);
    let (word, chord) = if open {
        (NEW, Some((keys::chord_for(Act::New), None)))
    } else {
        ("", None)
    };
    let seat = ui
        .push_id(NEW, |ui| row(ui, super::PLUS, word, chord, help))
        .inner;
    crate::ui::act::tag(&seat, &[crate::verbs::PREPARE, crate::verbs::PROMPT]);
    if seat.clicked() {
        model.begin_at(&aim);
        model.chose();
    }
}

/// **One verb row**: its glyph in weak ink, its word, an aside where it has
/// one, its help.
fn row(
    ui: &mut egui::Ui,
    mark: &str,
    word: &str,
    aside: Option<(String, Option<theme::State>)>,
    help: String,
) -> egui::Response {
    theme::paint::Row {
        mark: (mark.to_owned(), theme::INK_WEAK),
        words: word.to_owned(),
        ink: theme::INK,
        state: None,
        chosen: false,
        indent: 0.0,
        aside,
        beneath: Vec::new(),
        tail: None,
        door: None,
        help,
    }
    .paint(ui)
}

/// **Paint the pages' rows and the filter**, and fire what was clicked.
pub(super) fn render(ui: &mut egui::Ui, model: &mut Model) {
    let asking = crate::ui::status::waiting::waiting(model);
    let mut fired = None;
    for page in slash::rows(model).into_iter().filter(|page| page.page) {
        let count = (page.word == crate::ui::queue::WORD && asking > 0)
            .then(|| (asking.to_string(), Some(theme::State::Attention)));
        let help = keys::cited_word(page.summary, page.word);
        let seat = row(ui, PAGE_MARK, page.word, count, help);
        crate::ui::act::tag(&seat, page.ops);
        if seat.clicked() {
            fired = Some(page.word);
        }
    }
    if let Some(word) = fired {
        slash::fire(model, word, "");
    }
    ui.add_space(theme::space::XS);
    let id = egui::Id::new(keys::FILTER_ID);
    let field = theme::paint::field(ui, id, &mut model.bar.sift, FILTER, None, FILTER_HELP);
    if (field.has_focus() || field.lost_focus()) && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        model.bar.sift.clear();
    }
    ui.add_space(theme::space::XS);
}

/// **Whether a row answers to the filter** — every row while it is empty.
pub fn sifted(row: &crate::reply::convs::ConvRow, sift: &str) -> bool {
    sift.trim().is_empty()
        || [
            Some(row.display.as_str()),
            row.name.as_deref(),
            Some(row.preview.as_str()),
        ]
        .into_iter()
        .flatten()
        .any(|words| slash::narrows(words, sift))
}
