//! **The bar's rows** (DESIGN §4.44 item 1): one per conversation of the
//! aimed workspace, in the listing's order, with the subtree's indent.
//!
//! Expanded, a row is a block of two lines (bl-2997; operator 2026-10-31:
//! *the name is a useful piece of data, but it's not the most valuable*):
//! the state glyph in its state's ink and `ConvRow::display` in body ink
//! (the selected one's in the brand) with the age at the right end, and
//! beneath them the preview's first line with `ConvRow::name` — the
//! addressable name — at the right end, both in weak ink at the small size.
//! The name stands only where it differs from the display: a row whose
//! display IS its name says it once, at the head. The line beneath is laid
//! whether or not there is a preview, so every open row is one height.
//! Collapsed, it is the glyph and the display alone, which the row elides
//! with `…` at the width it has — so a collapsed bar dragged wider shows
//! more of each (§4.45 item 2; bl-773e). Every word is `crate::ui::convs`'
//! or the wire's — a glyph or an age changes there for both painters.
//!
//! **A trashcan stands at the open row's right end while the pointer is
//! over it** (§4.44 item 1; bl-e837): `theme::paint::Row`'s door, in faint
//! ink and lit under the pointer, under a seat of its own. **The row's help
//! is `convs::help`'s sentence** — the state in words and the failure clause
//! — so the glyph's meaning is one rest of the pointer away (bl-d9ce). Its click asks
//! before deleting — the confirmation on the box (`super::confirm`) — and
//! with Ctrl held it deletes at once; both go through `Model::post_delete`,
//! the one door `/delete` goes through too (§4.42 item 5).
//!
//! **Open, the listing is laid in two parts and parted by age** (DESIGN
//! §4.48 item 5): the conversations kept at the top under `PINNED` — Shift+
//! click on a row keeps it or lets it go, as the row's menu does — then the
//! rest in the listing's own order, a small heading over a hairline wherever
//! a root's age crosses onto another stretch of the calendar
//! (`convs::epoch`). Both parts answer to the filter (`super::verbs`); the
//! collapsed bar is the listing whole, none of the four.

use crate::reply::convs::ConvRow;
use crate::ui::{Model, convs, theme};

/// What an aimed workspace with no conversation says, open. **It names the next
/// act**, the `+ new` at the bar's head.
pub const EMPTY: &str = "no conversations here yet — + new begins one";
/// What the bar says, open, with nothing aimed: the foot is where one is.
pub const NOTHING_AIMED: &str = "pick a workspace below";
/// The kept group's heading, the baseline's word.
pub const KEPT: &str = "PINNED";
/// What the kept group says while it keeps nothing.
pub const KEPT_HINT: &str = "shift-click a conversation to pin";
/// What the listing says when the filter leaves nothing.
pub const NOTHING_MATCHES: &str = "nothing matches";
/// What Shift on a row's click does, said after its own sentence.
pub const SHIFT_KEEPS: &str = "; Shift+click keeps it at the top of this bar";

/// What a frame's rows were asked to do.
#[derive(Default)]
struct Asked {
    picked: Option<String>,
    kept: Option<String>,
    trashed: Option<(String, bool)>,
}

/// **Paint the rows**, scrolled where they outrun the bar, and do what was
/// clicked — `Model::select`, the one door, or the keeping.
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
    let kept = model.kept();
    let sift = model.bar.sift.clone();
    let (top, rest): (Vec<&ConvRow>, Vec<&ConvRow>) = rows
        .iter()
        .filter(|row| !open || super::verbs::sifted(row, &sift))
        .partition(|row| open && kept.contains(&row.root_id));
    let now = convs::epoch::now();
    let mut asked = Asked::default();
    egui::ScrollArea::vertical()
        .id_salt("bar")
        .auto_shrink(false)
        .show(ui, |ui| {
            if open {
                heading(ui, KEPT, theme::BRAND);
                if kept.is_empty() {
                    inset(ui, KEPT_HINT, theme::INK_WEAK);
                }
                for row in &top {
                    listed(ui, model, row, open, &mut asked);
                }
                ui.add_space(theme::space::S);
                if rest.is_empty() && !sift.trim().is_empty() {
                    theme::paint::empty(ui, NOTHING_MATCHES);
                }
            }
            let mut last = None;
            for row in &rest {
                if open && row.depth == 0 {
                    let epoch = convs::epoch::epoch(now, row.age_secs);
                    if last.as_ref() != Some(&epoch) {
                        parting(ui, &epoch);
                        last = Some(epoch);
                    }
                }
                listed(ui, model, row, open, &mut asked);
            }
        });
    if let Some(id) = asked.picked {
        model.select(&id);
    }
    if let Some(id) = asked.kept {
        model.toggle_kept(&id);
    }
    if let Some((id, at_once)) = asked.trashed {
        if at_once {
            model.post_delete(&id);
        } else {
            model.begin_deleting(&id);
        }
    }
}

/// **A heading at the small size**, in `ink`, standing over the rows' glyphs.
fn heading(ui: &mut egui::Ui, word: &str, ink: egui::Color32) {
    inset(ui, &word.to_uppercase(), ink);
}

/// **A line inset to where a row's glyph stands**: past its state rule and
/// the air after it (`theme::paint::Row`), at the small size in `ink`,
/// wrapped at the bar's width so no line of it widens the bar.
///
/// **Painted, not laid as a widget**: a widget takes the next of the
/// panel's ids, so a heading laid above the rows would hand every row a
/// new one as the bar opens — and the row a pointer opened the bar on would
/// no longer be the row under it.
fn inset(ui: &mut egui::Ui, words: &str, ink: egui::Color32) {
    let left = theme::RULE + theme::space::M;
    let font = egui::TextStyle::Small.resolve(ui.style());
    let galley = ui.painter().layout(
        words.to_owned(),
        font,
        ink,
        (ui.available_width() - left).max(0.0),
    );
    let tall = galley.size().y;
    ui.painter()
        .galley(ui.cursor().min + egui::vec2(left, 0.0), galley, ink);
    ui.add_space(tall + ui.spacing().item_spacing.y);
}

/// **An age separator**: the stretch's words over a hairline, a line between
/// two rows and never a second ordering — painted, for [`inset`]'s reason.
fn parting(ui: &mut egui::Ui, epoch: &str) {
    ui.add_space(theme::space::XS);
    heading(ui, epoch, theme::INK_WEAK);
    let at = ui.cursor().min;
    ui.painter().hline(
        at.x..=at.x + ui.available_width(),
        at.y,
        egui::Stroke::new(1.0, theme::HAIRLINE),
    );
    ui.add_space(1.0 + ui.spacing().item_spacing.y);
}

/// **One row of the listing and what its click asked**: a click selects it,
/// Shift held keeps it or lets it go, the trashcan deletes; its menu hangs
/// off it as the roster's does (§4.23).
fn listed(ui: &mut egui::Ui, model: &mut Model, row: &ConvRow, open: bool, asked: &mut Asked) {
    let (seat, trash) = one(ui, model, row, open);
    if seat.clicked() {
        if ui.input(|i| i.modifiers.shift) {
            asked.kept = Some(row.root_id.clone());
        } else {
            asked.picked = Some(row.root_id.clone());
        }
    }
    if trash.is_some_and(|door| door.clicked()) {
        let at_once = ui.input(|i| i.modifiers.command);
        asked.trashed = Some((row.root_id.clone(), at_once));
    }
    if let Some(aim) = model.aim.clone() {
        convs::menu::show(&seat, model, &aim, row);
    }
}

/// **One row**, answering the seat a click lands on and the trashcan's where
/// it stood. Open, it is the whole block — glyph and display, the age at the
/// right end, the preview beneath with the name at its right end — under one
/// seat, so the tint fills what a click takes (DESIGN §4.44 item 5); the
/// trashcan is the open row's alone, since a collapsed row has no room past
/// its glyph.
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
    let preview = row.preview.lines().next().unwrap_or_default();
    let beneath = if open {
        vec![(preview.to_owned(), theme::INK_WEAK, egui::TextStyle::Small)]
    } else {
        Vec::new()
    };
    let name = open
        .then(|| row.name.clone().filter(|name| *name != row.display))
        .flatten();
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
        aside: open.then_some((age, None)),
        beneath,
        tail: name,
        door: open.then_some((theme::glyph::TRASH, convs::TRASH_HELP)),
        help: convs::help(row) + if open { SHIFT_KEEPS } else { "" },
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
