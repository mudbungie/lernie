//! **The bar's rows** (DESIGN §4.44 item 1): one per conversation of the
//! aimed wall, in the listing's order, with the subtree's indent.
//!
//! Expanded, a row is the state glyph in its state's ink, the name in body
//! ink (the selected one's in the brand), the age at the right end and the
//! preview beneath, both in weak ink. Collapsed, it is the glyph and the
//! name alone, which the row elides with `…` at the width it has — so a
//! collapsed bar dragged wider shows more of each name (§4.45 item 2;
//! bl-773e). Every word is `crate::ui::convs`' — a glyph or an age changes
//! there for both painters.
//!
//! **A trashcan stands at the open row's right end while the pointer is
//! over it** (§4.44 item 1; bl-e837): `theme::paint::Row`'s door, in faint
//! ink and lit under the pointer, under a seat of its own. Its click asks
//! before deleting — the confirmation on the box (`super::confirm`) — and
//! with Ctrl held it deletes at once; both go through `Model::post_delete`,
//! the one door `/delete` goes through too (§4.42 item 5).

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
    let mut trashed = None;
    egui::ScrollArea::vertical()
        .id_salt("bar")
        .auto_shrink(false)
        .show(ui, |ui| {
            for row in &rows {
                let (seat, trash) = one(ui, model, row, open);
                if seat.clicked() {
                    picked = Some(row.root_id.clone());
                }
                if trash.is_some_and(|door| door.clicked()) {
                    let at_once = ui.input(|i| i.modifiers.command);
                    trashed = Some((row.root_id.clone(), at_once));
                }
            }
        });
    if let Some(id) = picked {
        model.select(&id);
    }
    if let Some((id, at_once)) = trashed {
        if at_once {
            model.post_delete(&id);
        } else {
            model.begin_deleting(&id);
        }
    }
}

/// **One row**, answering the seat a click lands on and the trashcan's where
/// it stood. Open, it is the whole block — glyph and name, the age at the
/// right end, the preview beneath — under one seat, so the tint fills what a
/// click takes (DESIGN §4.44 item 5); the trashcan is the open row's alone,
/// since a collapsed row has no room past its glyph.
fn one(
    ui: &mut egui::Ui,
    model: &Model,
    row: &ConvRow,
    open: bool,
) -> (egui::Response, Option<egui::Response>) {
    let selected = model.conversation.as_ref() == Some(&row.root_id);
    let ink = if selected { theme::BRAND } else { theme::INK };
    let (glyph, glyph_ink) = convs::mark(row);
    let age = convs::age(row.age_secs);
    let preview = row.preview.lines().next().filter(|line| !line.is_empty());
    let beneath = preview
        .filter(|_| open)
        .map(|line| (line.to_owned(), theme::INK_WEAK, egui::TextStyle::Small))
        .into_iter()
        .collect();
    let (seat, trash) = theme::paint::Row {
        mark: (glyph, glyph_ink),
        words: row.display.clone(),
        ink,
        state: convs::rule(row),
        chosen: selected,
        indent: if open {
            convs::indent(row.depth) - convs::STEP
        } else {
            0.0
        },
        aside: open.then_some(age),
        beneath,
        door: open.then_some((theme::glyph::TRASH, convs::TRASH_HELP)),
        help: convs::SELECT_HELP.to_owned(),
    }
    .paint_doored(ui);
    crate::ui::act::tag(
        &seat,
        &[crate::verbs::TRANSCRIPT.word, crate::verbs::FOLLOW.word],
    );
    if let Some(trash) = &trash {
        crate::ui::act::tag(trash, &[crate::verbs::DELETE_AGENT.word]);
    }
    (seat, trash)
}
