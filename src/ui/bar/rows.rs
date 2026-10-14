//! **The bar's rows** (DESIGN §4.44 item 1): one per conversation of the
//! aimed wall, in the listing's order, with the subtree's indent.
//!
//! Expanded, a row is the state glyph in its state's ink, the name in body
//! ink (the selected one's in the brand), the age at the right end and the
//! preview beneath, both in weak ink. Collapsed, it is the glyph and the
//! first [`super::CHARS`] characters of the name. Every word is
//! `crate::ui::convs`' — a glyph or an age changes there for both painters.

use crate::reply::convs::ConvRow;
use crate::ui::{Model, convs, theme};

/// What an aimed wall with no conversation says, open. **It names the next
/// act**, the `+ new` at the bar's head.
pub const EMPTY: &str = "no conversations here yet — + new begins one";
/// What the bar says, open, with nothing aimed: the foot is where one is.
pub const NOTHING_AIMED: &str = "pick a workspace below";

/// **Paint the rows**, scrolled where they outrun the bar, and select the
/// one clicked — `Model::select`, the one door.
pub(super) fn render(ui: &mut egui::Ui, model: &mut Model, open: bool) {
    let rows = model.rows();
    if rows.is_empty() {
        if open {
            let said = match &model.aim {
                None => NOTHING_AIMED,
                Some(aim) if model.answered.as_ref() == Some(aim) => EMPTY,
                Some(_) => convs::NOT_ANSWERED,
            };
            theme::paint::empty(ui, said);
        }
        return;
    }
    let mut picked = None;
    egui::ScrollArea::vertical()
        .id_salt("bar")
        .auto_shrink(false)
        .show(ui, |ui| {
            for row in &rows {
                if one(ui, model, row, open).clicked() {
                    picked = Some(row.root_id.clone());
                }
            }
        });
    if let Some(id) = picked {
        model.select(&id);
    }
}

/// **One row**, answering the seat a click lands on.
fn one(ui: &mut egui::Ui, model: &Model, row: &ConvRow, open: bool) -> egui::Response {
    let selected = model.conversation.as_ref() == Some(&row.root_id);
    let ink = if selected { theme::BRAND } else { theme::INK };
    let (glyph, glyph_ink) = convs::mark(row);
    let seat = if open {
        let indent = convs::indent(row.depth) - convs::STEP;
        let seat = aged(ui, row, |ui| {
            theme::paint::marked(
                ui,
                (&glyph, glyph_ink),
                &row.display,
                ink,
                convs::rule(row),
                selected,
                indent,
            )
        });
        preview(ui, row, indent);
        seat
    } else {
        theme::paint::marked(
            ui,
            (&glyph, glyph_ink),
            &super::cut(&row.display),
            ink,
            convs::rule(row),
            selected,
            0.0,
        )
    };
    crate::ui::act::tag(
        &seat,
        &[crate::verbs::TRANSCRIPT.word, crate::verbs::FOLLOW.word],
    );
    seat
}

/// **The row with its age at the right end**, in weak ink: the row takes the
/// width the age leaves it, so a long name is cut before the age, never under.
fn aged(
    ui: &mut egui::Ui,
    row: &ConvRow,
    paint: impl FnOnce(&mut egui::Ui) -> egui::Response,
) -> egui::Response {
    let age = convs::age(row.age_secs);
    let room = ui.available_width() - crate::ui::status::width(ui, &age) - theme::space::S;
    let line = ui.horizontal(|ui| {
        let seat = ui.allocate_ui(egui::vec2(room.max(0.0), theme::ROW), paint);
        ui.label(egui::RichText::new(age).small().color(theme::INK_WEAK));
        seat.inner
    });
    line.inner
}

/// **The preview beneath**, its first line in weak ink, cut with `…` — where
/// the conversation has said anything at all.
fn preview(ui: &mut egui::Ui, row: &ConvRow, indent: f32) {
    let Some(line) = row.preview.lines().next().filter(|line| !line.is_empty()) else {
        return;
    };
    ui.horizontal(|ui| {
        ui.add_space(indent + theme::RULE + theme::space::M);
        let words = egui::RichText::new(line).small().color(theme::INK_WEAK);
        ui.add(egui::Label::new(words).truncate());
    });
}
