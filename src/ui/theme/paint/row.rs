//! **The list row** (`docs/STYLE.md` §5): one line, or a block of them — a
//! head, an aside at its right end and the lines hung beneath it — under ONE
//! seat, so the tint under the pointer and the chosen row's tint fill exactly
//! what a click takes (DESIGN §4.44 item 5).

use egui::{Color32, Pos2, Response, Sense, Stroke, vec2};

use super::super::{BRAND, INK_WEAK, RADIUS, RAISED, ROW, RULE, space};

/// **One list row**: full width, [`ROW`] tall, its words at the left plus
/// `indent` plus `M`, elided at the width it has, in `ink`. Bare ground at
/// rest, [`super::super::hover`] under the pointer, `RAISED` when it is the
/// chosen one.
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
    marked(ui, ("", ink), words, ink, state, chosen, indent)
}

/// **The width a row needs to say `words` uncut** (bl-8d6d): the geometry
/// [`marked`] lays them in — the rule, the air either side, the words at
/// the body size — so a list sized by it never elides its longest row.
pub fn row_width(ui: &egui::Ui, words: &str) -> f32 {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let said = ui.fonts(|fonts| {
        fonts
            .layout_no_wrap(words.to_owned(), font, Color32::PLACEHOLDER)
            .size()
            .x
    });
    RULE + space::M + said + space::S
}

/// **A row led by a mark in its own ink** — a conversation's state glyph in
/// the switcher (DESIGN §4.42 item 7). The mark and the words are still ONE
/// run, `<mark> <words>`, on the glass and in the accessibility tree alike, so
/// a probe aims at what the operator reads; only the mark's ink differs.
pub fn marked(
    ui: &mut egui::Ui,
    mark: (&str, Color32),
    words: &str,
    ink: Color32,
    state: Option<Color32>,
    chosen: bool,
    indent: f32,
) -> Response {
    Row {
        mark: (mark.0.to_owned(), mark.1),
        words: words.to_owned(),
        ink,
        state,
        chosen,
        indent,
        aside: None,
        beneath: Vec::new(),
    }
    .paint(ui)
}

/// **A row as a block**: [`marked`]'s head, with an aside in weak ink at its
/// right end and lines hung beneath, each in its own ink and size and cut with
/// `…` at the row's width. The whole block is one seat — tinted, ruled and
/// ringed edge to edge — because it is one thing a click takes.
pub struct Row {
    pub mark: (String, Color32),
    pub words: String,
    pub ink: Color32,
    pub state: Option<Color32>,
    pub chosen: bool,
    pub indent: f32,
    /// At the head's right end, `SMALL` in weak ink — a conversation's age.
    pub aside: Option<String>,
    /// The lines beneath the head, aligned with its words.
    pub beneath: Vec<(String, Color32, egui::TextStyle)>,
}

impl Row {
    /// **Paint the block**, answering its one seat.
    pub fn paint(&self, ui: &mut egui::Ui) -> Response {
        let said = [self.mark.0.as_str(), self.words.as_str()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let height = ROW
            + self
                .beneath
                .iter()
                .map(|(_, _, s)| tall(ui, s))
                .sum::<f32>();
        let (rect, seat) =
            ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
        seat.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::SelectableLabel,
                ui.is_enabled(),
                self.chosen,
                &said,
            )
        });
        if !ui.is_rect_visible(rect) {
            return seat;
        }
        let fill = if self.chosen {
            RAISED
        } else if seat.hovered() {
            super::super::hover()
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, RADIUS, fill);
        if let Some(accent) = self.chosen.then_some(BRAND).or(self.state) {
            super::rule_at(ui, rect.left_top(), rect.height(), accent);
        }
        // **The row that holds the keyboard wears the brand ring** (bl-2d6b):
        // a row is in the Tab order and Space fires it, and the one stroke the
        // language leaves on the glass is the ring on whatever holds the
        // keyboard — so a Tab that landed here is visible where it landed.
        if seat.has_focus() {
            ui.painter()
                .rect_stroke(rect, RADIUS, Stroke::new(1.0, BRAND));
        }
        let left = rect.min.x + RULE + self.indent + space::M;
        let mut right = rect.max.x - space::S;
        let middle = rect.min.y + ROW / 2.0;
        if let Some(aside) = &self.aside {
            let small = egui::TextStyle::Small;
            let galley = elided(ui, aside, 0, (INK_WEAK, INK_WEAK), &small, f32::INFINITY);
            right -= galley.size().x;
            let at = Pos2::new(right, middle - galley.size().y / 2.0);
            ui.painter().galley(at, galley, INK_WEAK);
            right -= space::S;
        }
        let lead = said.len().saturating_sub(self.words.len());
        let body = egui::TextStyle::Body;
        let inks = (self.mark.1, self.ink);
        let galley = elided(ui, &said, lead, inks, &body, right - left);
        let at = Pos2::new(left, middle - galley.size().y / 2.0);
        ui.painter().galley(at, galley, self.ink);
        let mut top = rect.min.y + ROW;
        for (line, ink, style) in &self.beneath {
            let galley = elided(
                ui,
                line,
                0,
                (*ink, *ink),
                style,
                rect.max.x - space::S - left,
            );
            ui.painter().galley(Pos2::new(left, top), galley, *ink);
            top += tall(ui, style);
        }
        seat
    }
}

/// How much height one line beneath a head takes at `style`.
fn tall(ui: &egui::Ui, style: &egui::TextStyle) -> f32 {
    let font = style.resolve(ui.style());
    ui.fonts(|fonts| fonts.row_height(&font)) + space::XXS
}

/// **One line of words that stop at a width**, with `…` where they were cut —
/// so the cut is on the glass and a probe reads what an operator reads.
///
/// The first `lead` bytes are the mark and wear the first ink; the rest wear
/// the second.
fn elided(
    ui: &egui::Ui,
    said: &str,
    lead: usize,
    (marked, ink): (Color32, Color32),
    style: &egui::TextStyle,
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let font = style.resolve(ui.style());
    let mut job = egui::text::LayoutJob::simple_singleline(said.to_owned(), font, ink);
    if let Some(first) = job.sections.first_mut() {
        first.byte_range = lead..said.len();
        let mut mark = first.clone();
        mark.byte_range = 0..lead;
        mark.format.color = marked;
        job.sections.insert(0, mark);
    }
    job.wrap = egui::text::TextWrapping::truncate_at_width(width.max(0.0));
    ui.fonts(|fonts| fonts.layout_job(job))
}
