//! **The tree**: the baseline's navigation at the overlay's left — sections
//! that fold, their pages under them, the active path revealed (DESIGN §4.48
//! item 7). It is a column where the overlay is wide enough for a tree and a
//! page side by side, and a select above the page where it is not: the
//! overlay floats in the column, and at the matrix size (§4.32) that is
//! narrower than the two together. A select rather than a glyph column
//! because the pages have words and no glyphs, and a column of nine glyphs
//! nobody has seen before is a legend to learn.
//!
//! **A row is a door to its word.** Clicking a page turns the overlay to it
//! (`Model::turn_to`), which fires the page's word where it has one — so the
//! tree offers exactly what the menu offers, and a page whose word is off the
//! menu (the workspace's four, with nothing aimed) is dark rather than dead.

use crate::ui::{Group, Model, Page, theme};

/// How wide the column stands.
pub const WIDTH: f32 = 150.0;
/// What the select says under the pointer.
pub const SELECT_HELP: &str = "which page the settings show";

/// **The column**: each section a header that folds, its pages as rows
/// under it. The active section cannot fold — the path to the page that
/// stands is always on the glass.
pub fn column(ui: &mut egui::Ui, model: &mut Model, page: Page) {
    ui.vertical(|ui| {
        ui.set_width(WIDTH);
        for group in Group::ALL {
            let id = ui.make_persistent_id(group.label());
            let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id,
                true,
            );
            if group == page.group() {
                state.set_open(true);
            }
            state
                .show_header(ui, |ui| {
                    ui.label(egui::RichText::new(group.label()).color(theme::INK));
                })
                .body(|ui| {
                    for leaf in group.pages() {
                        row(ui, model, leaf, leaf == page);
                    }
                });
        }
    });
}

/// One page's row: chosen wears the brand rule; one not offered is faint and
/// takes no click.
fn row(ui: &mut egui::Ui, model: &mut Model, leaf: Page, chosen: bool) {
    let offered = model.offers(leaf);
    let ink = if offered {
        theme::INK
    } else {
        theme::INK_FAINT
    };
    let seat = ui
        .add_enabled_ui(offered, |ui| {
            theme::paint::row(ui, leaf.label(), ink, None, chosen, 0.0, leaf.help())
        })
        .inner;
    if seat.clicked() {
        model.turn_to(leaf);
    }
}

/// **The select**: the path that stands, opening on every page's path.
pub fn select(ui: &mut egui::Ui, model: &mut Model, page: Page) {
    let mut chosen = None;
    let seat = egui::ComboBox::from_id_salt(SELECT_HELP)
        .selected_text(path(page))
        .width(ui.available_width())
        .show_ui(ui, |ui| {
            for leaf in Page::ALL {
                let offered = model.offers(leaf);
                let seat = ui.add_enabled(
                    offered,
                    egui::SelectableLabel::new(leaf == page, path(leaf)),
                );
                if seat.clicked() {
                    chosen = Some(leaf);
                }
            }
        })
        .response;
    // **The select is named by the path it shows**: the toolkit's own combo
    // paints its text as a galley and names the node by a side label, which
    // this one has none of — so the walk could not find it by what it says.
    let said = path(page);
    seat.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::ComboBox, true, &said));
    theme::paint::told(seat, SELECT_HELP);
    if let Some(leaf) = chosen {
        model.turn_to(leaf);
    }
    ui.add_space(theme::space::XS);
}

/// `Workspace › roles`: the section and the page, as the breadcrumb says them.
pub fn path(page: Page) -> String {
    format!(
        "{} {} {}",
        page.group().label(),
        theme::glyph::PROMPT,
        page.label()
    )
}
