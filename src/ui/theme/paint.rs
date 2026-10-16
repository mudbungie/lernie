//! **The anatomy, as paint** (`docs/STYLE.md` §5; bl-d1ae): the row, the
//! connector, the ruled block, the section and the field — every shape a pane
//! puts on the glass that is not a bare label, written once — and every
//! control, each painter taking the control's help sentence beside its word
//! (DESIGN §4.46 item 2; [`told`]).
//!
//! A pane that drew its own row would be a pane with its own opinion about
//! what a row is, and the window it replaces was that: a `selectable_label`
//! here, a `Button` there, a fill painted under a run somewhere else, each
//! with its own box. Nothing here is boxed. A row is raised by a tint and told
//! by a rule; a block is told by a rule; a section is told by a hairline.

use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, vec2};

use super::{BRAND, HAIRLINE, INK, INK_FAINT, INK_WEAK, RADIUS, RULE, space, type_scale};

/// The button, the toggle seat: the one home of egui's own controls.
mod control;
/// The two fields that take text: the one-line box and the composer.
mod field;
/// The slider, the settings pane's number knob (DESIGN §4.46 item 4).
mod knob;
/// The list row, one line or a block of them.
mod row;

pub use control::{button, seat, toggle};
pub use knob::slider;
pub use row::{Row, row, row_width};

/// **A control told** (DESIGN §4.46 item 2): `help` — one sentence that
/// defines what the control does — rides the seat as its tooltip, shown
/// after the pointer has rested on it for the style's delay, and the seat
/// is recorded for the instruments (`crate::ui::act::bridge`). Every painter
/// of a control ends here, which is what makes a control without a sentence
/// a control that does not compile.
pub fn told(response: Response, help: &str) -> Response {
    crate::ui::act::bridge(&response);
    // **The first control under a pane that just opened takes the keyboard**
    // (`crate::ui::overlay::FIRST`; bl-99ed): the flag is read down here,
    // in the one function every control passes through, so no pane has to
    // know which of its controls is first.
    let first = egui::Id::new(crate::ui::overlay::FIRST);
    if response.ctx.data_mut(|d| d.remove_temp::<bool>(first)) == Some(true) {
        response.request_focus();
    }
    response.on_hover_text(help)
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
/// `ink` standing beside it from its first line to its last. The covering
/// panes' anatomy — aligned and ruled; the transcript's is [`bubbled`].
pub fn ruled(ui: &mut egui::Ui, ink: Color32, body: impl FnOnce(&mut egui::Ui)) {
    let top = ui.cursor().min;
    ui.horizontal_top(|ui| {
        ui.add_space(RULE + space::S);
        ui.vertical(body);
    });
    let bottom = ui.min_rect().max.y;
    rule_at(ui, top, bottom - top.y, ink);
}

/// **A bubbled block** (DESIGN §4.45 item 7; `docs/STYLE.md` §5, *a block*):
/// whatever `body` paints, on the fill its `role` wears at [`RADIUS`], `S` of
/// air inside, spanning the column — so the gutter glyph stands INSIDE the
/// fill and the four roles read as one ramp down the pane rather than as
/// four widths of speech balloon. The stroke is the role's too, and only the
/// trail has one. **With no role the block stands on the ground** with the
/// same air — the `bubbles` knob off (DESIGN §4.46 item 4) — so the layout
/// is the same whichever way it is set.
pub fn bubbled(ui: &mut egui::Ui, role: Option<super::Bubble>, body: impl FnOnce(&mut egui::Ui)) {
    let (fill, stroke) = role.map_or((Color32::TRANSPARENT, Stroke::NONE), super::bubble);
    egui::Frame::none()
        .fill(fill)
        .stroke(stroke)
        .rounding(RADIUS)
        .inner_margin(space::S)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            body(ui);
        });
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
pub fn link(ui: &mut egui::Ui, words: &str, ink: Color32, help: &str) -> Response {
    door(
        ui,
        egui::TextStyle::Small,
        &[(words, ink)],
        words,
        0.0,
        help,
    )
}

/// **A door at the heading size with `S` of air inside its seat** (DESIGN
/// §4.44 item 1; bl-bc9d): the bar's head's pin and `+ new`. At the rule's
/// size they were a target the width of a word, which the operator found
/// too small; the air is INSIDE the seat, so what a hand reaches for is the
/// air and the words together, and the head stands a row taller than a
/// body line for it.
pub fn heading_link(ui: &mut egui::Ui, words: &str, ink: Color32, help: &str) -> Response {
    door(
        ui,
        egui::TextStyle::Heading,
        &[(words, ink)],
        words,
        space::S,
        help,
    )
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
    help: &str,
) -> Response {
    let mark = format!(" {mark}");
    door(
        ui,
        egui::TextStyle::Small,
        &[(words, ink), (&mark, marked)],
        words,
        0.0,
        help,
    )
}

/// **A tab** (DESIGN §4.44 item 2): a [`link`] at the body size.
pub fn tab(ui: &mut egui::Ui, words: &str, ink: Color32, help: &str) -> Response {
    door(ui, egui::TextStyle::Body, &[(words, ink)], words, 0.0, help)
}

/// **The one door**: `parts` laid as one run at `style`, each in its own ink,
/// one click seat over all of them named `name`, `pad` of air inside the
/// seat on every side — bare at rest, the hover wash under the pointer, the
/// brand ring while it holds the keyboard.
fn door(
    ui: &mut egui::Ui,
    style: egui::TextStyle,
    parts: &[(&str, Color32)],
    name: &str,
    pad: f32,
    help: &str,
) -> Response {
    let font = style.resolve(ui.style());
    let mut job = egui::text::LayoutJob::default();
    for (text, ink) in parts {
        job.append(text, 0.0, egui::TextFormat::simple(font.clone(), *ink));
    }
    let galley = ui.fonts(|fonts| fonts.layout_job(job));
    let (rect, seat) = ui.allocate_exact_size(galley.size() + vec2(pad, pad) * 2.0, Sense::click());
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
    ui.painter().galley(rect.min + vec2(pad, pad), galley, INK);
    told(seat.on_hover_cursor(egui::CursorIcon::PointingHand), help)
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
