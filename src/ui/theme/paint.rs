//! **The anatomy, as paint** (`docs/STYLE.md` §5; bl-d1ae): the row, the
//! connector, the ruled block, the section and the field — every shape a pane
//! puts on the glass that is not a bare label or a control, written once.
//!
//! A pane that drew its own row would be a pane with its own opinion about
//! what a row is, and the window it replaces was that: a `selectable_label`
//! here, a `Button` there, a fill painted under a run somewhere else, each
//! with its own box. Nothing here is boxed. A row is raised by a tint and told
//! by a rule; a block is told by a rule; a section is told by a hairline.

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, vec2};

use super::{BRAND, HAIRLINE, INK_FAINT, INK_WEAK, RADIUS, RAISED, ROW, RULE, SURFACE, space};

/// **One list row**: full width, [`ROW`] tall, its words at the left plus
/// `indent` plus `M`, elided at the width it has, in `ink`. Bare ground at
/// rest, `SURFACE` under the pointer, `RAISED` when it is the chosen one.
///
/// **The rule at its left edge says its state**: the brand when it is chosen,
/// else `state`'s accent where it has one, else nothing. A chosen row that is
/// also asking keeps the brand — the selection is where the operator IS, and
/// the asking is said in the row's words.
///
/// The words are ONE run, so a test that aims a click by the painted glyphs
/// aims at exactly what a pane composed (`crate::paint_probe::locate`), and
/// the node the accessibility tree offers is labelled with the same words.
pub fn row(
    ui: &mut egui::Ui,
    words: &str,
    ink: Color32,
    state: Option<Color32>,
    chosen: bool,
    indent: f32,
) -> Response {
    let (rect, seat) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
    seat.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            chosen,
            words,
        )
    });
    if !ui.is_rect_visible(rect) {
        return seat;
    }
    let fill = if chosen {
        RAISED
    } else if seat.hovered() {
        SURFACE
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, RADIUS, fill);
    if let Some(accent) = chosen.then_some(BRAND).or(state) {
        rule_at(ui, rect.left_top(), rect.height(), accent);
    }
    let left = rect.min.x + RULE + indent + space::M;
    let galley = elided(ui, words, ink, rect.max.x - left - space::S);
    let at = Pos2::new(left, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(at, galley, ink);
    seat
}

/// **One line of words that stop at a width**, with `…` where they were cut —
/// so the cut is on the glass and a probe reads what an operator reads.
fn elided(ui: &egui::Ui, words: &str, ink: Color32, width: f32) -> std::sync::Arc<egui::Galley> {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let mut job = egui::text::LayoutJob::simple_singleline(words.to_owned(), font, ink);
    job.wrap = egui::text::TextWrapping::truncate_at_width(width.max(0.0));
    ui.fonts(|fonts| fonts.layout_job(job))
}

/// **A state's rule**: [`RULE`] wide, `height` tall, from `top`.
fn rule_at(ui: &egui::Ui, top: Pos2, height: f32, accent: Color32) {
    ui.painter()
        .rect_filled(Rect::from_min_size(top, vec2(RULE, height)), 0.0, accent);
}

/// **A threading rail**: one hairline down a row's height at `x`, in faint
/// ink — the phone's L-shaped connector idiom, the vertical half.
pub fn rail(ui: &egui::Ui, x: f32, top: f32, bottom: f32) {
    ui.painter()
        .vline(x, top..=bottom, Stroke::new(1.0, INK_FAINT));
}

/// **The elbow**: the rail down to the row's middle and across to its words.
pub fn elbow(ui: &egui::Ui, x: f32, top: f32, middle: f32, to: f32) {
    let stroke = Stroke::new(1.0, INK_FAINT);
    ui.painter().vline(x, top..=middle, stroke);
    ui.painter().hline(x..=to, middle, stroke);
}

/// **A ruled block**: whatever `body` paints, with a [`RULE`]-wide line in
/// `ink` standing beside it from its first line to its last. The transcript's
/// anatomy — aligned, ruled, never bubbled.
pub fn ruled(ui: &mut egui::Ui, ink: Color32, body: impl FnOnce(&mut egui::Ui)) {
    let top = ui.cursor().min;
    ui.horizontal_top(|ui| {
        ui.add_space(RULE + space::S);
        ui.vertical(body);
    });
    let bottom = ui.min_rect().max.y;
    rule_at(ui, top, bottom - top.y, ink);
}

/// **A section of a covering pane**: `L` of air, a hairline, and its word in
/// weak ink — never a framed group.
pub fn section(ui: &mut egui::Ui, word: &str) {
    ui.add_space(space::L);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    ui.painter().hline(
        rect.min.x..=rect.max.x,
        rect.center().y,
        Stroke::new(1.0, HAIRLINE),
    );
    ui.add_space(space::XS);
    ui.label(egui::RichText::new(word).color(INK_WEAK));
}

/// **A field that takes text**: `SURFACE` with no stroke at rest, the brand
/// ring while it holds the caret, and `glow`'s tint under it where the pane
/// has something to say about the moment — the composer while its
/// conversation is asking. The box wears `id`, which is what the keyboard's
/// gate compares against (`crate::ui::keys`).
pub fn field(
    ui: &mut egui::Ui,
    id: egui::Id,
    text: &mut String,
    hint: &str,
    glow: Option<Color32>,
) -> Response {
    let focused = ui.memory(|memory| memory.has_focus(id));
    let ring = if focused {
        Stroke::new(1.0, BRAND)
    } else {
        Stroke::NONE
    };
    egui::Frame::none()
        .fill(glow.unwrap_or(SURFACE))
        .stroke(ring)
        .rounding(RADIUS)
        .inner_margin(egui::Margin::symmetric(space::S, space::XS))
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::singleline(text)
                    .id(id)
                    .frame(false)
                    .desired_width(f32::INFINITY)
                    .hint_text(hint),
            )
        })
        .inner
}

#[cfg(test)]
mod tests;
