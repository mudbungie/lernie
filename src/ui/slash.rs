//! **The slash verbs**: the way into every pane and every act this window
//! affords, typed into the one box (DESIGN §4.42, item 5; `docs/STYLE.md` §2,
//! *navigation is typed*).
//!
//! A `/` at the head of the draft opens the **completion menu** — one row per
//! verb this seat offers right now, its word and what it does — and the word
//! typed after the slash is the act. The three strips this replaces (the
//! window's acts above the roster, the aimed wall's band under its row, and
//! the composer's row of offers with its `…` strip) were fifteen controls on
//! three surfaces; every one of them is a row here, reached from the box the
//! operator is already in.
//!
//! # A row's word is the word its control carried
//!
//! The strips named each pane by its noun with an ellipsis — `records…`,
//! `trail…`, `config…` — and each act by its verb. The rows keep exactly
//! those words, minus the ellipsis: the noun opens the pane, the verb fires
//! the act. Nothing is renamed, so a hand that reached for `records…` types
//! `/records`, and the `act:` tags move with the words unchanged — the
//! parity ledger (`crate::snapshot::parity`) reads the menu where it read the
//! strips.
//!
//! # Only what is offered is listed
//!
//! A row is on the menu where its control would have been on the glass: the
//! window's own verbs always; the aimed wall's once a wall is aimed at, the
//! pin and the unpin by the row's own rank (DESIGN §4.25); the selected
//! conversation's once one is selected and addressable, `interrupt` and
//! `stop` while the engine says the driver runs and `nudge` while it rests
//! (§4.38's *the row offers what the engine offers*, with the row deleted);
//! and `plan` in the start's own mode. An absent reading is not a refusal:
//! until the engine has answered about the conversation, every act is
//! listed.
//!
//! # The words a verb takes ride the same line
//!
//! `/flag it is rewriting an unrelated crate` raises the flag with that
//! reason; `/delete <name>` arms the cascade by the name typed back, which is
//! the arming §4.20 asks for with no second box; `/interrupt say this
//! instead` is the cut with its content; `/find needle` opens the pane and
//! spends the needle. A verb whose parameter is missing fires nothing, as the
//! disabled control it replaces fired nothing.

use crate::ui::{Model, theme};

/// What firing a word does.
mod fire;
/// The rows this seat offers right now, and the table they are read from.
mod rows;

pub use fire::{fire, wanting};
pub use rows::rows;

/// The glyph that opens the menu, and the one character a slash line starts
/// with.
pub const LEAD: char = '/';

/// **The start's own parameter** (`crate::ui::composer::start`): the next
/// start is a plan, said until it is said again. Not a wire verb — it states
/// a field of the `prompt` the start is already tagged for.
pub const PLAN: &str = "plan";
/// The verb that asks every engine for its roster again.
pub const REFRESH: &str = "refresh";
/// The verb that unmakes the selected conversation — `delete-agent` on the
/// wire, the word the composer's strip carried on the glass.
pub const DELETE: &str = "delete";

/// **One row of the menu**: the word typed, what it does, and the ops its
/// firing spends — which are its `act:` tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    pub word: &'static str,
    pub summary: &'static str,
    pub ops: &'static [&'static str],
}

impl Row {
    const fn new(word: &'static str, summary: &'static str, ops: &'static [&'static str]) -> Row {
        Row { word, summary, ops }
    }

    /// The row as one run of text: the word, then its summary.
    pub fn line(&self) -> String {
        format!("/{}  {}", self.word, self.summary)
    }
}

/// **What the operator typed, if it is a slash line**: the word and the rest.
pub fn parsed(draft: &str) -> Option<(&str, &str)> {
    let line = draft.strip_prefix(LEAD)?;
    let (word, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    Some((word.trim(), rest.trim()))
}

/// **The rows the typed word narrows to** — every row whose word starts with
/// it, all of them on a bare `/`.
pub fn matching(model: &Model, draft: &str) -> Vec<Row> {
    let Some((word, _)) = parsed(draft) else {
        return Vec::new();
    };
    rows(model)
        .into_iter()
        .filter(|row| row.word.starts_with(word))
        .collect()
}

/// Where the menu keeps which row is chosen — the toolkit's memory, as a
/// folded tool result's is: a view state no wire gesture corresponds to.
const CHOSEN_ID: &str = "the menu's chosen row";
/// **Where it keeps why the last word did not fire**, beside the draft it was
/// said about — the same kind of view state, and it stands only while that
/// draft does, so the next keystroke takes it down.
const WHY_ID: &str = "why the menu's word did not fire";

/// **Paint the menu over the box and take what it was given**: a click on a
/// row, ↑↓ moving the choice, Tab completing the word, Enter firing it.
///
/// Called by the shell after the composer's panel, so the box it floats over
/// has already been laid, and only while the draft is a slash line.
pub fn menu(ui: &mut egui::Ui, model: &mut Model) {
    let draft = model.draft.clone();
    let Some((_, rest)) = parsed(&draft) else {
        return;
    };
    let rows = matching(model, &draft);
    let id = egui::Id::new(CHOSEN_ID);
    let mut at = ui.data(|d| d.get_temp::<usize>(id)).unwrap_or(0);
    let (up, down, tab, enter, escape) = ui.input(|i| {
        (
            i.key_pressed(egui::Key::ArrowUp),
            i.key_pressed(egui::Key::ArrowDown),
            i.key_pressed(egui::Key::Tab),
            i.key_pressed(egui::Key::Enter) && !i.modifiers.shift,
            i.key_pressed(egui::Key::Escape),
        )
    });
    if up {
        at = at.saturating_sub(1);
    }
    if down {
        at = at.saturating_add(1);
    }
    at = at.min(rows.len().saturating_sub(1));
    ui.data_mut(|d| d.insert_temp(id, at));
    let mut chose = None;
    // **A window of rows centred on the choice, never a scroll** — the
    // baseline's own shape, and the one the matrix admits: every control
    // laid is on the glass whole, so a menu longer than the pane lays the
    // rows around the chosen one and no others. The box is as tall as the
    // rows it lays, a line for the hint, and the frame's own air.
    let line = theme::ROW + ui.spacing().item_spacing.y;
    let why_id = egui::Id::new(WHY_ID);
    let why = ui
        .data(|d| d.get_temp::<(String, String)>(why_id))
        .filter(|(said_of, _)| *said_of == draft)
        .map(|(_, why)| why);
    let small = ui.text_style_height(&egui::TextStyle::Small) + ui.spacing().item_spacing.y;
    let hint = small + why.as_ref().map_or(0.0, |_| line);
    let room = crate::ui::overlay::ceiling(ui) - 2.0 * theme::space::S - hint;
    // Counted rather than divided: one row always, then one more for every
    // line that still fits — so no float is ever cast to a count.
    let (mut fit, mut tall) = (1, line);
    while fit < rows.len() && tall + line <= room {
        tall += line;
        fit += 1;
    }
    let start = at
        .saturating_sub(fit / 2)
        .min(rows.len().saturating_sub(fit));
    let shown = rows
        .get(start..(start + fit).min(rows.len()))
        .unwrap_or(&[]);
    let height = tall + hint + 2.0 * theme::space::S;
    crate::ui::overlay::float(ui, "the completion menu", height, |ui| {
        ui.label(egui::RichText::new(HINT).small().color(theme::INK_WEAK));
        if let Some(why) = &why {
            ui.colored_label(theme::NOTICE, why);
        }
        for (i, row) in shown.iter().enumerate() {
            let seat = theme::paint::row(ui, &row.line(), theme::INK, None, start + i == at, 0.0);
            crate::ui::act::tag(&seat, row.ops);
            if seat.clicked() {
                chose = Some(*row);
            }
        }
        if rows.is_empty() {
            theme::paint::empty(ui, NOTHING);
        }
    });
    if escape {
        model.draft.clear();
        return;
    }
    if tab && let Some(row) = rows.get(at) {
        model.draft = format!("{LEAD}{} ", row.word);
        return;
    }
    if enter {
        chose = rows.get(at).copied();
    }
    if let Some(row) = chose {
        if fire(model, row.word, rest) {
            model.draft.clear();
        } else if let Some(why) = wanting(model, row.word, rest) {
            ui.data_mut(|d| d.insert_temp(why_id, (draft, why)));
            ui.ctx().request_repaint();
        }
    }
}

/// What the menu says when nothing starts with what was typed.
pub const NOTHING: &str = "nothing here answers to that — Esc clears it";
/// The keys, said once under the rows.
pub const HINT: &str = "arrows move · Tab completes · Enter fires · Esc clears";

#[cfg(test)]
mod tests;
