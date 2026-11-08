//! **The rails** (DESIGN §4.48 item 8): the two columns the advanced mode
//! docks at the window's right — the branch rail beside the column, the
//! context rail at the edge — each a pane this seat already has, re-homed
//! from the overlay while `Model::docked` says so (`crate::ui::model::rail`).
//!
//! # What a rail paints is the pane, whole
//!
//! The branch rail paints the trail (`crate::ui::trail`) and the place a
//! trail is cut in (`crate::ui::clear`); the context rail paints the records
//! pane (`crate::ui::records`) or the config pane (`crate::ui::config`),
//! whichever stands. Each pane's own `render` is called, with its own name
//! line and its own standing read, exactly as the overlay calls it: the pane
//! has no second painter, and nothing here knows what a trail row is.
//!
//! # The width is the bar's rule, read for two
//!
//! The rails may share two fifths of the window — the bar's own ceiling
//! (§4.44 item 1) — and never what would squeeze the column under its floor
//! beside the bar ([`COLUMN`]); the bar is counted at the width it stands
//! at when the pointer is away, open only while pinned, so a hover does not
//! move the rails. Each rail is a resizable panel: its edge is dragged
//! between [`FLOOR`] and what the other rail leaves of the allowance, so the
//! two never exceed it between them (§4.45 item 2's clamping, on what is
//! shown). The dragged width is the toolkit's memory for the run and not a
//! knob: a rail is a column the operator opens and closes, and a place-file
//! width for one would be a setting on a thing that is not usually there.
//!
//! # Room is decided once, for two rails, before the frame
//!
//! A window is roomy ([`room`]) where the allowance holds two rails at their
//! floor. It is asked for two whatever stands, so that opening the second
//! rail never changes the answer mid-frame — and where there is no room the
//! panes float as the simple mode lays them, which is `Model::housed`'s
//! reading rather than this file's. The matrix's phone size has no room; its
//! narrow size has exactly enough.

use crate::ui::{Model, Rail, bar, overlay, theme};

/// **The least a rail may be**, in points: a provenance line's worth of
/// wrapped words, and the widest box a pane lays (the config pane's name
/// box, which the toolkit cuts to the width it has).
pub const FLOOR: f32 = 180.0;
/// **The least the column keeps**: what the matrix's phone size leaves it
/// beside the shut bar (`crate::snapshot::SIZES`, `crate::settings::BAR_SHUT`)
/// — the narrowest shape every pane is built to fit (§4.32).
pub const COLUMN: f32 = 400.0 - crate::settings::BAR_SHUT;

/// **Where a rail's scroll clips**, written each frame it stands, for the
/// one reader that has to know (`crate::snapshot::clipped`): the overlay's
/// [`overlay::BOX`], one per rail.
pub(crate) const fn clip(rail: Rail) -> &'static str {
    match rail {
        Rail::Branch => "the branch rail's box",
        Rail::Context => "the context rail's box",
    }
}

/// **The bar's width as the rails count it**: open while pinned, shut
/// otherwise — the pointer's linger moves the column, never the rails.
fn bar_width(ctx: &egui::Context, model: &Model) -> f32 {
    let window = ctx.screen_rect().width();
    let (shut, wide) = bar::drag::shown(&model.settings, window, bar::drag::floor(ctx));
    if model.bar.pinned { wide } else { shut }
}

/// **What the rails may share** in a window `window` wide beside a bar
/// `bar` wide: the bar's ceiling, and never into the column's floor.
pub fn allowance(window: f32, bar: f32) -> f32 {
    bar::drag::ceiling(window).min(window - bar - COLUMN)
}

/// **Whether the window has room for two rails at their floor** — the
/// reading `Model::housed` takes at the top of the frame.
pub fn room(ctx: &egui::Context, model: &Model) -> bool {
    allowance(ctx.screen_rect().width(), bar_width(ctx, model)) >= 2.0 * FLOOR
}

/// **Lay every rail that stands**, context first so it takes the edge, each
/// a resizable panel on the window's right. Nothing paints under the
/// enrollment, and nothing is laid while the panes float.
pub fn render(ctx: &egui::Context, model: &mut Model) {
    if model.enroll.is_some() {
        return;
    }
    let laid: Vec<Rail> = Rail::ALL
        .into_iter()
        .filter(|rail| model.railed(*rail))
        .collect();
    let Some(last) = laid.last().copied() else {
        return;
    };
    let allowance = allowance(ctx.screen_rect().width(), bar_width(ctx, model));
    // **A rail opens at half the allowance whether or not the other stands**
    // — the baseline's rails are each a third of its chat, and a lone rail
    // at two fifths of the window is a second column, not a rail. Dragged,
    // a lone one may take the whole.
    let share = allowance / 2.0;
    let mut left = allowance;
    for rail in laid {
        // **The other rail keeps its floor** out of what this one may take,
        // and the second takes what the first left.
        let ceiling = if rail == last { left } else { left - FLOOR };
        let panel = egui::SidePanel::right(rail.name())
            .resizable(true)
            .default_width(share)
            .width_range(FLOOR..=ceiling.max(FLOOR))
            .frame(
                egui::Frame::none()
                    .fill(theme::GROUND)
                    .inner_margin(egui::Margin::symmetric(theme::space::S, theme::space::XS)),
            )
            .show(ctx, |ui| body(ui, model, rail));
        left -= panel.response.rect.width();
    }
}

/// **One rail's content**: the pane that stands on it, scrolled — the
/// overlay's own shape (§4.11's rule, kept), so a pane longer than the
/// window scrolls inside rather than laying controls past its edge.
fn body(ui: &mut egui::Ui, model: &mut Model, rail: Rail) {
    ui.data_mut(|d| d.insert_temp(egui::Id::new(clip(rail)), ui.max_rect()));
    egui::ScrollArea::vertical()
        .id_salt(rail.name())
        .auto_shrink(false)
        .show(ui, |ui| match rail {
            Rail::Branch => {
                let _ = crate::ui::trail::render(ui, model) || crate::ui::clear::render(ui, model);
            }
            Rail::Context => {
                // **The config pane's name line is its home's** (§4.48 item
                // 7): the overlay's breadcrumb there, the rail's name here.
                if !crate::ui::records::render(ui, model) {
                    overlay::name(ui, crate::ui::config::WORD);
                    crate::ui::config::render(ui, model);
                }
            }
        });
}

#[cfg(test)]
mod tests;
