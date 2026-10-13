//! **A selector's dropup** (DESIGN §4.42, item 11): the list a click on a
//! rule segment opens, standing over that segment and opening upward.
//!
//! It is the overlay's frame over a span of the pane rather than across it
//! (`crate::ui::overlay::float_at`), laid at the bottom of the central panel
//! where the rule begins — a child region and not an `egui::Area`, for the
//! overlay's own reason: a fresh area paints nothing on its first frame. It
//! lists what [`Model::choices`] holds at once, ↑↓ walk it, Enter or a click
//! picks, Esc or a click anywhere else closes it — and it stands down with the
//! rule under an overlay and under the completion menu.

use crate::ui::{Model, Pick, Selector, theme};

/// Where the open selector's segment stood this frame — a view fact, held
/// where view facts are.
const ANCHOR: &str = "the open selector's segment";
/// The pass on which a segment was clicked, so that click is not also the
/// click elsewhere that closes the list it opened.
const TOGGLED: &str = "the pass a selector was clicked on";
/// The box's own name.
pub const NAME: &str = "a selector's dropup";
/// What the list says under the rows while it has none.
pub const NOTHING: &str = "nothing heard yet";
/// The keys, said once over the rows.
pub const HINT: &str = "arrows walk · Enter picks · Esc closes";
/// What marks the row in force.
const CURRENT: &str = theme::glyph::CALL;

/// **Record where `selector`'s segment stood**, where its list is open.
pub fn anchor(ui: &egui::Ui, model: &Model, selector: Selector, rect: egui::Rect) {
    if model
        .dropup
        .as_ref()
        .is_some_and(|open| open.selector == selector)
    {
        ui.data_mut(|d| d.insert_temp(egui::Id::new(ANCHOR), rect));
    }
}

/// **A click on a segment**: open its list or close it, and say so to this
/// pass's click-elsewhere.
pub fn toggle(ui: &egui::Ui, model: &mut Model, selector: Selector) {
    model.drop_up(selector);
    let pass = ui.ctx().cumulative_pass_nr();
    ui.data_mut(|d| d.insert_temp(egui::Id::new(TOGGLED), pass));
}

/// **The ops a row's pick spends**: the aim reads the wall's conversations,
/// a provider opens its model list, and the rest are the tuning writes.
fn ops(pick: &Pick) -> &'static [&'static str] {
    use crate::verbs;
    match pick {
        Pick::Wall(_) => &[verbs::CONVERSATIONS.word],
        Pick::Provider(_) => &[verbs::MODELS.word],
        Pick::Model(_) => &[verbs::MODEL.word],
        Pick::Effort(_) => &[verbs::EFFORT],
        Pick::Priority(_) => &[verbs::PRIORITY],
    }
}

/// **Take the dropup's keys**, before the box can: ↑↓ walk, Enter picks, Esc
/// closes — each consumed, so the prompt neither sends nor moves on them.
pub fn keys(ctx: &egui::Context, model: &mut Model) {
    if model.dropup.is_none() || model.covered() {
        return;
    }
    let took = |key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
    let (up, down, enter, escape) = (
        took(egui::Key::ArrowUp),
        took(egui::Key::ArrowDown),
        took(egui::Key::Enter),
        took(egui::Key::Escape),
    );
    if up {
        model.walk_dropup(-1);
    }
    if down {
        model.walk_dropup(1);
    }
    if escape {
        model.close_dropup();
    }
    if enter && let Some(open) = &model.dropup {
        model.pick(open.cursor);
    }
}

/// **Paint the open list over its segment**, and take a click on a row or
/// anywhere else.
pub fn render(ui: &mut egui::Ui, model: &mut Model) {
    let Some(cursor) = model.dropup.as_ref().map(|open| open.cursor) else {
        return;
    };
    if model.covered() || crate::ui::slash::parsed(&model.draft).is_some() {
        model.close_dropup();
        return;
    }
    let choices = model.choices();
    let room = ui.max_rect();
    let air = 2.0 * theme::space::S;
    let words = |text: &str| super::width(ui, text) + air;
    // **As wide as its longest row, capped at the pane** (bl-8d6d): each row
    // measured as the row paints it, at the body size — measured at the
    // rule's small size it came out short, and a long model id was cut with
    // `…` at a desk width with room to spare — plus the bar a list taller
    // than the box scrolls with.
    let bar = ui.spacing().scroll.bar_width + ui.spacing().scroll.bar_inner_margin;
    let wide = choices
        .iter()
        .map(|choice| {
            theme::paint::row_width(ui, &format!("{CURRENT} {}", choice.text)) + air + bar
        })
        .fold(words(HINT), f32::max)
        .min(room.width() - 2.0 * theme::space::L);
    let anchor = ui.data(|d| d.get_temp::<egui::Rect>(egui::Id::new(ANCHOR)));
    let left = anchor
        .map_or(room.left(), |seat| seat.left() - theme::space::S)
        .min(room.right() - theme::space::L - wide)
        .max(room.left() + theme::space::L);
    let line = theme::ROW + ui.spacing().item_spacing.y;
    let lines = std::iter::repeat_n(line, choices.len().max(1) + 1).sum::<f32>();
    let mut picked = None;
    let span = egui::Rangef::new(left, left + wide);
    let stood = crate::ui::overlay::float_at(ui, NAME, span, lines + air, |ui| {
        ui.label(egui::RichText::new(HINT).small().color(theme::INK_WEAK));
        for (at, choice) in choices.iter().enumerate() {
            let text = if choice.current {
                format!("{CURRENT} {}", choice.text)
            } else {
                choice.text.clone()
            };
            let seat = theme::paint::row(ui, &text, theme::INK, None, at == cursor, 0.0);
            crate::ui::act::tag(&seat, ops(&choice.pick));
            if seat.clicked() {
                picked = Some(at);
            }
        }
        if choices.is_empty() {
            theme::paint::empty(ui, NOTHING);
        }
    });
    if let Some(at) = picked {
        model.pick(at);
        return;
    }
    let toggled = ui.data(|d| d.get_temp::<u64>(egui::Id::new(TOGGLED)));
    let pass = ui.ctx().cumulative_pass_nr();
    let elsewhere = ui.input(|i| {
        i.pointer.any_click()
            && i.pointer
                .interact_pos()
                .is_some_and(|at| !stood.contains(at))
    });
    if elsewhere && toggled != Some(pass) {
        model.close_dropup();
    }
}
