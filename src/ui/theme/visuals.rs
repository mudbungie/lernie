//! **The one adapter into egui**: the tokens installed as a `Style` (bl-73d2).
//!
//! Nothing here decides a colour; every byte is [`super`]'s. What this file
//! decides is which egui slot each token fills, and that is stated once so a
//! pane never reaches for a `Visuals` field of its own. The shape it installs
//! is `docs/STYLE.md` §2 and §5 read as egui: the ground is the panel fill,
//! the ladder is the widget fills at rest / under the pointer / pressed, the
//! hairline is the one stroke a boundary may be, and the brand is the ring on
//! the field that holds the caret — the one stroke left on the glass.

use egui::{Color32, FontFamily, FontId, Rounding, Stroke, TextStyle};

use super::{
    BRAND, GROUND, HAIRLINE, INK, RADIUS, RAISED, SURFACE, State, accent, space, tint, type_scale,
};

/// **Install the language on this context**, once per frame — the same call
/// the native boot and every offscreen frame make, so what a test paints is
/// what the operator sees.
pub fn install(ctx: &egui::Context) {
    let mut style = egui::Style {
        text_styles: text_styles(),
        ..egui::Style::default()
    };
    // **The gaps are the scale's** (STYLE §4): the default item gap is `S`
    // across and — a desktop delta — `XXS` down, because three columns of
    // single-line rows at the phone's gap is a window one third empty and a
    // covering pane at it overruns the narrowest shape the layout promises.
    // A control is padded `S` across and `XXS` down for the same reason, and
    // stands no taller than its word asks; `theme::ROW` is a list row's
    // height, spent by the row helper and not by every control.
    let spacing = &mut style.spacing;
    spacing.item_spacing = egui::vec2(space::S, space::XXS);
    spacing.button_padding = egui::vec2(space::S, space::XXS);
    spacing.interact_size = egui::vec2(space::XL + space::L, space::L);
    spacing.indent = space::L;
    spacing.window_margin = egui::Margin::same(space::M);
    spacing.menu_margin = egui::Margin::same(space::S);
    style.visuals = visuals();
    ctx.set_style(style);
}

/// **Four sizes and no fifth** (STYLE §4): a button's words are body size,
/// because a control is text and not a smaller kind of text.
fn text_styles() -> std::collections::BTreeMap<TextStyle, FontId> {
    [
        (
            TextStyle::Small,
            FontId::new(type_scale::SMALL, FontFamily::Proportional),
        ),
        (
            TextStyle::Body,
            FontId::new(type_scale::BODY, FontFamily::Proportional),
        ),
        (
            TextStyle::Button,
            FontId::new(type_scale::BODY, FontFamily::Proportional),
        ),
        (
            TextStyle::Heading,
            FontId::new(type_scale::HEADING, FontFamily::Proportional),
        ),
        (
            TextStyle::Monospace,
            FontId::new(type_scale::MONO, FontFamily::Monospace),
        ),
    ]
    .into()
}

/// **The window's visuals**: dark, with no light face, and every slot filled
/// from the tokens.
pub fn visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.override_text_color = None;
    v.panel_fill = GROUND;
    v.window_fill = SURFACE;
    v.window_stroke = Stroke::NONE;
    v.window_rounding = Rounding::same(RADIUS);
    v.menu_rounding = Rounding::same(RADIUS);
    v.faint_bg_color = SURFACE;
    v.extreme_bg_color = SURFACE;
    v.code_bg_color = RAISED;
    v.hyperlink_color = BRAND;
    v.warn_fg_color = accent(State::Annotation);
    v.error_fg_color = accent(State::Error);
    v.text_cursor.stroke = Stroke::new(2.0, BRAND);
    // **The selection is a brand tint under the words and the brand ring
    // around the field that holds the caret** — the one stroke left.
    v.selection.bg_fill = tint(State::Working);
    v.selection.stroke = Stroke::new(1.0, BRAND);
    v.widgets.noninteractive = widget(GROUND, Stroke::new(1.0, HAIRLINE), INK);
    v.widgets.inactive = widget(SURFACE, Stroke::NONE, INK);
    v.widgets.hovered = widget(RAISED, Stroke::NONE, INK);
    v.widgets.active = widget(RAISED, Stroke::NONE, INK);
    v.widgets.open = widget(RAISED, Stroke::NONE, INK);
    v.button_frame = true;
    v.striped = false;
    v
}

/// One rung of the widget ladder: a fill, the one boundary it may have, and
/// the ink its words are in. **No expansion** — a control does not grow under
/// the pointer, it tints.
fn widget(fill: Color32, boundary: Stroke, ink: Color32) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: boundary,
        rounding: Rounding::same(RADIUS),
        fg_stroke: Stroke::new(1.0, ink),
        expansion: 0.0,
    }
}
