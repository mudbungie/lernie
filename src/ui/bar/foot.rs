//! **The bar's foot is the workspace** (DESIGN §4.44 item 1): item 11's
//! workspace selector moved whole off the rule — the aimed workspace's name with
//! `⏶` beside it, `workspace ⏶` with nothing aimed, elided with `…` at the
//! width the bar has, so a collapsed bar dragged wider shows more of it
//! (§4.45 item 2; bl-773e) — and the dropup a click opens, standing upward
//! from it.
//!
//! The run is `crate::ui::status::selectors::workspace`'s and the list is
//! `crate::ui::status::dropup`'s: the same words, mark, rows, keys and picks
//! the rule's segment had. What is the foot's own is only where the list
//! stands — on a layer above the bar's rows, inside the bar's edges, its
//! bottom on the foot's top — and that the bar stays open while it stands
//! (`super::render`): a list opened from the bar is the bar in use, and the
//! column beside it keeps every word it had.

use crate::ui::status::{self, dropup, selectors};
use crate::ui::{Model, Selector, theme};

/// The list's layer, named once.
const LAYER: &str = "the workspace's dropup at the bar's foot";

/// **Paint the foot**, toggle its list on a click, and answer where it
/// stood. The words are cut by the rule's own cut, to the width the bar has
/// less the mark that stands inside the door with them.
pub(super) fn render(ui: &mut egui::Ui, model: &mut Model) -> egui::Rect {
    let run = selectors::workspace(model);
    let mark = run
        .mark
        .map_or(0.0, |(glyph, _)| status::width(ui, &format!(" {glyph}")));
    let words = status::elide(ui, &run.text, ui.available_width() - mark);
    let (clicked, rect) = ui.horizontal(|ui| status::paint(ui, &run, &words)).inner;
    dropup::anchor(ui, model, Selector::Workspace, rect);
    if clicked {
        dropup::toggle(ui, model, Selector::Workspace);
    }
    rect
}

/// **The workspace's list, where it is open**, standing on the foot's top
/// inside the bar `ui` is — the dropup keeps its margin from the edges of
/// the room it is given, so the room is the bar widened by that margin less
/// a hair either side.
pub(super) fn list(ui: &egui::Ui, model: &mut Model, foot: egui::Rect) {
    if !dropup::footed(model) {
        return;
    }
    let bar = ui.max_rect();
    let edge = theme::space::L - theme::space::XS;
    let room = egui::Rect::from_min_max(
        egui::pos2(bar.left() - edge, ui.ctx().screen_rect().top()),
        egui::pos2(bar.right() + edge, foot.top()),
    );
    let id = egui::Id::new(LAYER);
    let mut ui = egui::Ui::new(
        ui.ctx().clone(),
        id,
        egui::UiBuilder::new()
            .layer_id(egui::LayerId::new(egui::Order::Foreground, id))
            .max_rect(room),
    );
    dropup::stand(&mut ui, model);
}
