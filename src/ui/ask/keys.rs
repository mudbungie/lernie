//! **The ask's keys and the field's words while it asks** (DESIGN §4.42,
//! item 8; bl-1252) — split from [`super`] on the seam its own doc draws:
//! that module paints the box and answers it, and this is what the prompt
//! line lends it while the caret stays in the box.

use super::{HEADING, choices, confirm, down, moved, pick, row};
use crate::ui::Model;

/// The digits, in the order they number the rows.
const DIGITS: [egui::Key; 9] = [
    egui::Key::Num1,
    egui::Key::Num2,
    egui::Key::Num3,
    egui::Key::Num4,
    egui::Key::Num5,
    egui::Key::Num6,
    egui::Key::Num7,
    egui::Key::Num8,
    egui::Key::Num9,
];

/// **The ask's keys** (DESIGN §4.42, item 8; bl-1252): the prompt keeps the
/// caret while the conversation asks, so a key is the ask's only while the
/// draft is empty — a digit picks the row it numbers, ↑↓ move the choice and
/// Enter answers with it — and each is taken out of the frame's input, so the
/// box never reads it as well. With words in the draft every key is the
/// box's: Enter sends them, or fires the slash line, and typed words never go
/// nowhere. A control a Tab landed on holds the keyboard and is egui's to
/// fire, so nothing here runs over it.
pub fn keys(ctx: &egui::Context, model: &mut Model) {
    let field = egui::Id::new(crate::ui::keys::BOX_ID);
    let here = ctx
        .memory(egui::Memory::focused)
        .is_none_or(|id| id == field);
    if !here || !model.draft.is_empty() || row(model).is_none() || down(model).is_some() {
        return;
    }
    let took = |key| ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
    for (key, step) in [(egui::Key::ArrowUp, -1), (egui::Key::ArrowDown, 1)] {
        if took(key) {
            moved(ctx, model, step);
        }
    }
    if took(egui::Key::Enter) {
        confirm(ctx, model);
    }
    for (n, key) in (1..).zip(DIGITS) {
        if took(key) {
            let typed = n.to_string();
            ctx.input_mut(|i| {
                i.events
                    .retain(|event| !matches!(event, egui::Event::Text(t) if *t == typed));
            });
            pick(model, n);
        }
    }
}

/// **What the field says while the conversation asks** (bl-1252): that it
/// asks, the digits that answer where the queue's row has landed, and that
/// words typed are sent — because the caret is in the box, and the box is
/// where both kinds of answer go.
pub fn hint(model: &Model) -> String {
    if let Some(why) = down(model) {
        return why;
    }
    let digits = match row(model).map(|asked| choices(&asked.1).len()) {
        Some(1) => "1 answers, or ".to_owned(),
        Some(n) => format!("1–{n} answers, or "),
        None => String::new(),
    };
    format!("{HEADING} — {digits}{}", crate::ui::composer::HINT)
}
