//! **The settings overlay: one frame, a tree of pages** (DESIGN §4.48 item
//! 7, amending §4.46 item 4 and §4.42 item 6).
//!
//! The baseline's Settings is one floating card: a navigation tree at the
//! left, a breadcrumb at the head of the page, a search field, and pages of
//! rows. This seat had `/settings` as one flat table of knobs and six panes
//! beside it that were each a page of what a person would call settings.
//! **They are pages of this one overlay now**: *Seat* (the knob table's three
//! pages, `crate::settings::Page`), *Workspace* (`/roles`, `/login`,
//! `/config`, `/clients`) and *Servers* (registering this box, `/offers`,
//! `/doctor`).
//!
//! **Every word stands.** A page is opened by its word where it has one
//! (`crate::ui::Page::word`), so the menu, the reach bound and the standing
//! reads are untouched; `/settings` and Ctrl+, open *Seat › look*; the mark
//! reopens the page last shown (`Model::reopen_settings`). Which page stands
//! is `Model::page`, derived from the panes' own fields — this file keeps no
//! state about it.
//!
//! **The tree is a column where there is room and a select where there is
//! not** (`tree`): the overlay floats in the column, and at the matrix size
//! the column is narrower than a tree and a page side by side.
//!
//! **The knob pages are rows in the baseline's shape** — label, help beneath
//! in weak ink, control at the right end (`rows`) — and a change lands on the
//! frame it is made (§4.46 item 4). The four workspace pages keep their
//! wrapped layouts, because their control groups do not fit at the right end
//! of a 320-point page: each keeps its tags, its standing read and its own
//! `set` or `write` where it crosses the wire.

use crate::ui::{Model, Page, overlay, theme};

/// **The word typed after the slash** that opens it on *Seat › look*
/// (`crate::ui::slash`), and the chord Ctrl+, fires (`crate::ui::keys`).
pub const WORD: &str = "settings";
/// What the menu says beside the word.
pub const SUMMARY: &str =
    "this seat's settings: its own knobs, the workspace's, the servers' — Ctrl+,";
/// The breadcrumb's root.
pub const HEADING: &str = "Settings";
/// **The overlay's inner width at which the tree stands as a column**: the
/// tree's width and a page as wide as the matrix size's window. Narrower,
/// the tree is a select above the page.
pub const WIDE: f32 = tree::WIDTH + 400.0;

/// Paint the overlay on whichever page stands, and take what changed on it.
/// Answers whether there was one to paint.
pub fn render(ui: &mut egui::Ui, model: &mut Model) -> bool {
    let Some(page) = model.page() else {
        return false;
    };
    model.visited = page;
    breadcrumb(ui, page);
    if search::render(ui, model) {
        return true;
    }
    if ui.available_width() >= WIDE {
        ui.horizontal_top(|ui| {
            tree::column(ui, model, page);
            ui.separator();
            ui.vertical(|ui| body(ui, model, page));
        });
    } else {
        tree::select(ui, model, page);
        body(ui, model, page);
    }
    true
}

/// **`Settings › Workspace › roles`**: each word its own label, so a word is
/// a thing the walk can find, with the way out at the line's other end.
fn breadcrumb(ui: &mut egui::Ui, page: Page) {
    // The hint takes the right end first, and the path wraps in what is
    // left — so at the matrix size the two never cross.
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        overlay::way_out(ui);
        // Laid left to right by name: a wrapped row inside a right-to-left
        // parent would run the other way.
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), theme::ROW),
            egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(true),
            |ui| {
                ui.label(egui::RichText::new(HEADING).color(theme::INK));
                for word in [page.group().label(), page.label()] {
                    ui.label(egui::RichText::new(theme::glyph::PROMPT).color(theme::INK_FAINT));
                    ui.label(egui::RichText::new(word).color(theme::INK_WEAK));
                }
            },
        );
    });
    ui.add_space(theme::space::XS);
}

/// The page's own rows, scrolled: the knob table's for a seat page, and the
/// pane's for the rest — each pane still answering for its own field.
fn body(ui: &mut egui::Ui, model: &mut Model, page: Page) {
    use crate::ui::{clients, config, doctor, login, tuning};
    egui::ScrollArea::vertical()
        .id_salt(WORD)
        .auto_shrink(false)
        .show(ui, |ui| match page {
            Page::Seat(seat) => rows::page(ui, model, seat),
            Page::Roles => {
                tuning::render(ui, model);
            }
            Page::Providers => {
                login::render(ui, model);
            }
            Page::Config => {
                config::render(ui, model);
            }
            Page::Clients => {
                clients::render(ui, model);
            }
            Page::Register => crate::ui::register::render(ui, model),
            Page::Offers => {
                crate::ui::offers::render(ui, model);
            }
            Page::Doctor => {
                doctor::render(ui, model);
            }
        });
}

/// The knob pages' rows, and the row shape every page's are laid in.
pub mod rows;
/// The box at the head that finds a row across every page.
pub mod search;
/// The tree: a column of sections and pages, or a select.
pub mod tree;

#[cfg(test)]
mod tests;
