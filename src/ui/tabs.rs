//! **The tab strip** (DESIGN §4.44 item 2; bl-03c1): over the transcript, one
//! tab per conversation open in the aimed workspace.
//!
//! Each tab is the conversation's name — the asking `?` in the attention
//! accent before it while it asks (`crate::ui::convs::mark`), the selected one
//! in body ink over a brand hairline and the rest in weak ink — and a close
//! `×` in faint ink after it. The strip stands only while a tab stands: a workspace
//! with nothing open is the transcript's own empty column.
//!
//! **Nothing here decides what is open.** The tabs are [`Model::tabs`], filled
//! by [`Model::select`] alone; a click on a tab IS that select, so it opens
//! nothing it was not already, and the close is [`Model::close_tab`].

use crate::reply::convs::ConvRow;
use crate::ui::{Model, theme};

/// What a tab does, and what its `×` does.
pub const TAB_HELP: &str = "shows this conversation";
pub const CLOSE_HELP: &str = "closes this tab; the conversation stays";

/// What a tab is called: the row's name where the seat holds the row (the
/// listing's, or a start's claim before the listing knows it), else its id.
fn row_of(model: &Model, root_id: &str) -> Option<ConvRow> {
    model
        .pending()
        .into_iter()
        .chain(model.convs.iter().cloned())
        .find(|row| row.root_id == root_id)
}

/// **Paint the strip**, if any tab of the aimed workspace stands.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let strip = model.strip();
    if strip.is_empty() {
        return;
    }
    let mut chosen = None;
    let mut closed = None;
    // **Wrapped, each run whole** (`extend`): a strip with more tabs than the
    // width breaks between tabs, never inside a name.
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = theme::space::XS;
        for root_id in &strip {
            let row = row_of(model, root_id);
            let selected = model.conversation.as_deref() == Some(root_id.as_str());
            // **Asking outranks nothing here but is said first** (§2, *the
            // eye lands on green*): the switcher's own glyph and ink.
            if let Some(row) = row.as_ref().filter(|row| row.attention > 0) {
                let (glyph, ink) = crate::ui::convs::mark(row);
                ui.add(egui::Label::new(egui::RichText::new(glyph).color(ink)).extend());
            }
            let name = row.map_or_else(|| root_id.clone(), |row| row.display);
            let ink = if selected {
                theme::INK
            } else {
                theme::INK_WEAK
            };
            let tab = theme::paint::tab(ui, &name, ink, TAB_HELP);
            // **Selecting reads its transcript and opens its follow lane**, as
            // a switcher row's select does (`crate::ui::convs::row`).
            crate::ui::act::tag(
                &tab,
                &[crate::verbs::TRANSCRIPT.word, crate::verbs::FOLLOW.word],
            );
            // **The hairline is a filled rect**, so the probe's fill walk can
            // read it back (`crate::paint_probe::fills_of`).
            if selected {
                let y = tab.rect.max.y + theme::space::XXS;
                let line = egui::Rect::from_x_y_ranges(tab.rect.x_range(), y..=y + 1.0);
                ui.painter().rect_filled(line, 0.0, theme::BRAND);
            }
            if tab.clicked() {
                chosen = Some(root_id.clone());
            }
            // **The close posts nothing**, so it wears no `act:` tag.
            // **The selected tab's × cites Ctrl+W** (§4.46 item 3): the chord
            // closes the selected one, and only that × is the same act.
            let help = if selected {
                crate::ui::keys::cited(CLOSE_HELP, &[crate::ui::keys::Act::CloseTab])
            } else {
                CLOSE_HELP.to_owned()
            };
            let close = theme::paint::tab(ui, theme::glyph::CLOSE, theme::INK_FAINT, &help);
            if close.clicked() {
                closed = Some(root_id.clone());
            }
            ui.add_space(theme::space::M);
        }
    });
    ui.add_space(theme::space::XS);
    if let Some(root_id) = chosen {
        model.select(&root_id);
    }
    if let Some(root_id) = closed {
        model.close_tab(&root_id);
    }
}

#[cfg(test)]
mod tests;
