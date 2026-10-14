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

use super::{
    BRAND, HAIRLINE, INK, INK_FAINT, INK_WEAK, RADIUS, RAISED, ROW, RULE, space, type_scale,
};

/// The two fields that take text: the one-line box and the composer.
mod field;

/// **One list row**: full width, [`ROW`] tall, its words at the left plus
/// `indent` plus `M`, elided at the width it has, in `ink`. Bare ground at
/// rest, [`super::hover`] under the pointer, `RAISED` when it is the chosen one.
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
    let said = [mark.0, words]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let (rect, seat) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
    seat.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            chosen,
            &said,
        )
    });
    if !ui.is_rect_visible(rect) {
        return seat;
    }
    let fill = if chosen {
        RAISED
    } else if seat.hovered() {
        super::hover()
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, RADIUS, fill);
    if let Some(accent) = chosen.then_some(BRAND).or(state) {
        rule_at(ui, rect.left_top(), rect.height(), accent);
    }
    // **The row that holds the keyboard wears the brand ring** (bl-2d6b): a
    // row is in the Tab order and Space fires it, and the one stroke the
    // language leaves on the glass is the ring on whatever holds the keyboard
    // — so a Tab that landed here is visible where it landed.
    if seat.has_focus() {
        ui.painter()
            .rect_stroke(rect, RADIUS, Stroke::new(1.0, BRAND));
    }
    let left = rect.min.x + RULE + indent + space::M;
    let lead = said.len().saturating_sub(words.len());
    let galley = elided(ui, &said, lead, (mark.1, ink), rect.max.x - left - space::S);
    let at = Pos2::new(left, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(at, galley, ink);
    seat
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
    width: f32,
) -> std::sync::Arc<egui::Galley> {
    let font = egui::TextStyle::Body.resolve(ui.style());
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

/// **An empty state**: one sentence in weak ink, wrapped at the width it
/// has, that says what to do next (bl-f251, item 5). Weak because it is not
/// content — it is the absence of content, said once — and one sentence
/// because the next act is the whole of what an empty pane owes a reader.
pub fn empty(ui: &mut egui::Ui, sentence: &str) -> Response {
    ui.add(egui::Label::new(egui::RichText::new(sentence).color(INK_WEAK)).wrap())
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

/// **A run that is a door** (`docs/STYLE.md` §5, *the status rule*): `words`
/// at `SMALL` in `ink`, laid exactly as wide as a label would lay them, so a
/// line that measured its runs as labels still holds. Bare ground at rest,
/// [`super::hover`] under the pointer with the pointing hand — the row's own
/// idiom at the size of a word, and never a box: a control on the rule is a
/// fact you can open, not a button standing among the facts.
pub fn link(ui: &mut egui::Ui, words: &str, ink: Color32) -> Response {
    door(ui, egui::TextStyle::Small, &[(words, ink)], words)
}

/// **A link with its mark inside it** (DESIGN §4.44 item 5): `<words>
/// <mark>`, the mark in its own ink, ONE seat — the selector's `⏶` is where
/// a hand reaches to open it, so it opens it, tints with it and is ringed
/// with it, and the run is one stop in the Tab order. It is named by its
/// words alone: the mark is the affordance, not the name.
pub fn link_marked(
    ui: &mut egui::Ui,
    words: &str,
    ink: Color32,
    (mark, marked): (&str, Color32),
) -> Response {
    let mark = format!(" {mark}");
    door(
        ui,
        egui::TextStyle::Small,
        &[(words, ink), (&mark, marked)],
        words,
    )
}

/// **A tab** (DESIGN §4.44 item 2): a [`link`] at the body size.
pub fn tab(ui: &mut egui::Ui, words: &str, ink: Color32) -> Response {
    door(ui, egui::TextStyle::Body, &[(words, ink)], words)
}

/// **The one door**: `parts` laid as one run at `style`, each in its own ink,
/// one click seat over all of them named `name` — bare at rest, the hover
/// wash under the pointer, the brand ring while it holds the keyboard.
fn door(
    ui: &mut egui::Ui,
    style: egui::TextStyle,
    parts: &[(&str, Color32)],
    name: &str,
) -> Response {
    let font = style.resolve(ui.style());
    let mut job = egui::text::LayoutJob::default();
    for (text, ink) in parts {
        job.append(text, 0.0, egui::TextFormat::simple(font.clone(), *ink));
    }
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    let (rect, seat) = ui.allocate_exact_size(galley.size(), Sense::click());
    seat.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Link, ui.is_enabled(), name));
    if seat.hovered() {
        ui.painter()
            .rect_filled(rect.expand(space::XS), RADIUS, super::hover());
    }
    // **A link that holds the keyboard wears the brand ring** (bl-8e3e;
    // bl-2d6b's rule for a row): a Tab lands on the rule's doors, and Enter or
    // Space opens one, so where it landed is visible.
    if seat.has_focus() {
        ui.painter()
            .rect_stroke(rect.expand(space::XS), RADIUS, Stroke::new(1.0, BRAND));
    }
    ui.painter().galley(rect.min, galley, INK);
    seat.on_hover_cursor(egui::CursorIcon::PointingHand)
}

pub use field::{field, prompt};

/// **A paragraph of prose**: `words` wrapped at the width it has, in `ink`,
/// laid at [`type_scale::LEADING`] rather than the font's own line height
/// (bl-f251). The transcript's body is read line after line, and a body
/// laid as tight as a list row reads as one — every other run on the glass
/// keeps the font's height, which is what makes this shape a shape.
pub fn prose(ui: &mut egui::Ui, words: &str, ink: Color32) -> Response {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let format = egui::TextFormat {
        font_id: font,
        color: ink,
        line_height: Some(type_scale::LEADING),
        ..egui::TextFormat::default()
    };
    let mut job = egui::text::LayoutJob::default();
    job.append(words, 0.0, format);
    job.wrap.max_width = ui.available_width();
    ui.add(egui::Label::new(job).wrap())
}

#[cfg(test)]
mod tests;
