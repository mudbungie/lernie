//! **The list row** (`docs/STYLE.md` §5): one line, or a block of them — a
//! head, an aside at its right end, the lines hung beneath it and a tail at
//! the last one's right end — under ONE seat, so the tint under the pointer
//! and the chosen row's tint fill exactly what a click takes (DESIGN §4.44
//! item 5).
//!
//! **A single line stands [`ROW`] tall; a block stands as tall as its lines**
//! (bl-2997): the head at the body's own height and each line beneath at its
//! own, with `XXS` of air above and below the block and none between the
//! lines — the font's own height is what a row keeps (STYLE §4), and a head
//! centred in `ROW` over a line beneath carried a third of the block as air.

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, vec2};

use super::super::{BRAND, INK_FAINT, INK_WEAK, RADIUS, RAISED, ROW, RULE, space};

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
    help: &str,
) -> Response {
    Row {
        mark: (String::new(), ink),
        words: words.to_owned(),
        ink,
        state,
        chosen,
        indent,
        aside: None,
        beneath: Vec::new(),
        tail: None,
        door: None,
        help: help.to_owned(),
    }
    .paint(ui)
}

/// **Where the head's words centre, under the block's top**: half [`ROW`]
/// for a single line, and `XXS` plus half the body's own height for a block
/// — the one answer, so a connector aimed at a row's words meets them.
pub fn middle(ui: &egui::Ui, blocked: bool) -> f32 {
    let (top, band) = head(ui, blocked);
    top + band / 2.0
}

/// **The head's band**: how far under the block's top it begins, and how
/// tall it stands. The air above it is the air under the block, so the two
/// sum the block's padding.
fn head(ui: &egui::Ui, blocked: bool) -> (f32, f32) {
    if blocked {
        (space::XXS, tall(ui, &egui::TextStyle::Body))
    } else {
        (0.0, ROW)
    }
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

/// **A row as a block**: [`marked`]'s head, with an aside in weak ink at its
/// right end, lines hung beneath, each in its own ink and size and cut with
/// `…` at the width it has, and a tail in weak ink at the last line's right
/// end. The whole block is one seat — tinted, ruled and ringed edge to edge —
/// because it is one thing a click takes.
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
    /// **At the last line beneath's right end**, `SMALL` in weak ink — the
    /// aside's twin on the bottom line, the bar row's addressable name
    /// (DESIGN §4.44 item 1; bl-2997). The line it shares is cut at the
    /// width the tail leaves it; with nothing beneath there is no line to
    /// stand on and it is not painted.
    pub tail: Option<String>,
    /// **A door at the head's right end, painted only while the pointer is
    /// over the row** (DESIGN §4.44 item 1; bl-e837): its glyph, in faint ink
    /// under a seat of its own, so its click is never the row's. The aside
    /// steps left of it. Its glyph, and its own help sentence.
    pub door: Option<(&'static str, &'static str)>,
    /// **What a click on the row does**, one sentence (DESIGN §4.46 item 2).
    pub help: String,
}

impl Row {
    /// **Paint the block**, answering its one seat.
    pub fn paint(&self, ui: &mut egui::Ui) -> Response {
        self.paint_doored(ui).0
    }

    /// **Paint the block and, while the pointer is over it, its door**,
    /// answering the row's seat and the door's where one was painted. The
    /// door is laid after the row, so the hit test hands it the click. The
    /// row asks whether the pointer is WITHIN it and not whether it is
    /// hovered, because a pointer on the door is on the row — egui hands
    /// `hovered` to the door alone — and a row whose wash went out the
    /// moment its door was reached would say the hand had left it.
    pub fn paint_doored(&self, ui: &mut egui::Ui) -> (Response, Option<Response>) {
        let said = [self.mark.0.as_str(), self.words.as_str()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        let blocked = !self.beneath.is_empty();
        let (pad, band) = head(ui, blocked);
        let height = pad
            + band
            + self
                .beneath
                .iter()
                .map(|(_, _, s)| tall(ui, s))
                .sum::<f32>()
            + pad;
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
            return (super::told(seat, &self.help), None);
        }
        let fill = if self.chosen {
            RAISED
        } else if seat.contains_pointer() {
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
        let middle = rect.min.y + pad + band / 2.0;
        let door = self
            .door
            .filter(|_| seat.contains_pointer())
            .map(|(glyph, help)| {
                let strip = Rect::from_min_max(
                    Pos2::new(left, rect.min.y + pad),
                    Pos2::new(right, rect.min.y + pad + band),
                );
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt(seat.id)
                        .max_rect(strip)
                        .layout(egui::Layout::right_to_left(egui::Align::Center)),
                );
                let door = super::tab(&mut child, glyph, INK_FAINT, help);
                right = door.rect.min.x - space::S;
                door
            });
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
        self.hang(ui, rect, left, rect.min.y + pad + band);
        // **The door's sentence is the one the pointer is on** (bl-a6b3): egui
        // keeps a seat's tooltip open while the pointer is anywhere inside
        // that seat, and the door stands inside the row — so a row that kept
        // telling would speak over its own door. While the door is hovered
        // the row attaches nothing and the door's tooltip opens on the
        // next frame; the seat is still recorded, since the row is still a
        // control.
        let seat = if door.as_ref().is_some_and(Response::hovered) {
            crate::ui::act::bridge(&seat);
            seat
        } else {
            super::told(seat, &self.help)
        };
        (seat, door)
    }

    /// **Hang the lines beneath the head** from `top`, each cut at the width
    /// it has, and the tail at the last one's right end.
    fn hang(&self, ui: &egui::Ui, rect: Rect, left: f32, top: f32) {
        let mut top = top;
        let last = self.beneath.len().saturating_sub(1);
        for (i, (line, ink, style)) in self.beneath.iter().enumerate() {
            let mut end = rect.max.x - space::S;
            if let Some(tail) = self.tail.as_ref().filter(|_| i == last) {
                let small = egui::TextStyle::Small;
                let galley = elided(ui, tail, 0, (INK_WEAK, INK_WEAK), &small, f32::INFINITY);
                end -= galley.size().x;
                let at = Pos2::new(end, top + (tall(ui, style) - galley.size().y) / 2.0);
                ui.painter().galley(at, galley, INK_WEAK);
                end -= space::S;
            }
            let galley = elided(ui, line, 0, (*ink, *ink), style, end - left);
            ui.painter().galley(Pos2::new(left, top), galley, *ink);
            top += tall(ui, style);
        }
    }
}

/// How much height one line of a block takes at `style`: the font's own.
fn tall(ui: &egui::Ui, style: &egui::TextStyle) -> f32 {
    let font = style.resolve(ui.style());
    ui.fonts(|fonts| fonts.row_height(&font))
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
