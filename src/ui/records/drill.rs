//! **One step's records, under the row that addresses them** (bl-3257).
//!
//! The steps ledger paints a `seq` per row; this is what that `seq` opens. The
//! control is on the row rather than at the top of the half, because the
//! question is *about that step* — and the answer paints under the row it was
//! asked from, keyed on the `seq` the reply echoes back
//! (`crate::ui::model::deep`), so nothing here remembers what was asked.
//!
//! # Bytes, not trees
//!
//! Every record is painted as the bytes the engine read, monospaced. That is
//! the whole promise of the drill-in: a parsed tree loses key order, spacing
//! and number spelling, so a rendering of one could not answer *what does the
//! file say*. The engine's own note above an unparseable record is repeated
//! rather than reworded, for the reason every sentence in this pane is.
//!
//! # A record that is not there is a sentence, and it is not an empty one
//!
//! `absent` means no bytes at all — missing, unreadable, or empty — and the
//! three are one fact for a file nobody promised to write. It paints as that
//! sentence. A response stream that landed nothing is a different claim again:
//! there are no events, rather than an event with nothing in it.

use crate::reply::convs::Tone;
use crate::reply::step::{Doc, Step, ToolCall};
use crate::ui::{Model, theme};

/// The word on the control that opens a step's records.
pub const OPEN: &str = "records…";
/// The word that puts them away.
pub const CLOSE: &str = "hide";
/// Said in place of a record with no bytes.
pub const NO_BYTES: &str = "no bytes — missing, unreadable, or empty";
/// Said for a response stream that landed nothing.
pub const NO_EVENTS: &str = "its response stream landed no event";
/// Said for a step that called no tool.
pub const NO_TOOLS: &str = "it called no tool";
/// The label a failed tool call wears.
pub const ERRORED: &str = "ended in error";

/// **The control on one steps row.** It rides in the row's own headline line
/// (`super`) rather than on a line of its own, because this pane covers the
/// window and its content has to fit it.
pub fn control(ui: &mut egui::Ui, model: &mut Model, seq: &str) {
    let open = model.drilled_into(seq).is_some();
    let word = if open { CLOSE } else { OPEN };
    let control = ui.button(word);
    crate::ui::act::tag(&control, &[crate::verbs::STEP.word]);
    if control.clicked() {
        if open {
            model.records.drilled = None;
        } else {
            model.ask_step(seq);
        }
    }
}

/// The records themselves, under the row that asked for them — and nothing at
/// all under every other row, which is what the `seq` on the answer decides.
pub fn records(ui: &mut egui::Ui, model: &Model, seq: &str) {
    if let Some(answer) = model.drilled_into(seq) {
        drilled(ui, &answer);
    }
}

/// The records themselves: the four files, the stream, the tool calls and the
/// captured logs.
fn drilled(ui: &mut egui::Ui, records: &Step) {
    for (head, doc) in [
        ("meta", &records.meta),
        ("request", &records.request),
        ("staging", &records.staging),
    ] {
        record(ui, head, doc);
    }
    ui.colored_label(theme::tone_ink(&Tone::Weak), "response");
    if records.response.is_empty() {
        ui.label(NO_EVENTS);
    }
    for (index, event) in records.response.iter().enumerate() {
        record(ui, &format!("event {index}"), event);
    }
    if records.tools.is_empty() {
        ui.colored_label(theme::tone_ink(&Tone::Weak), NO_TOOLS);
    }
    for call in &records.tools {
        tool(ui, call);
    }
    for (head, log) in [("stderr", &records.stderr), ("driver", &records.driver)] {
        if let Some(preview) = log {
            ui.colored_label(theme::tone_ink(&Tone::Weak), head.to_owned());
            ui.label(egui::RichText::new(super::previewed(preview)).monospace());
        }
    }
}

/// One tool call: what it was, how it ended, and both of its sides.
fn tool(ui: &mut egui::Ui, call: &ToolCall) {
    ui.colored_label(theme::tone_ink(&Tone::Weak), headline(call));
    record(ui, "input", &call.input);
    record(ui, "output", &call.output);
}

/// One record under its own name.
fn record(ui: &mut egui::Ui, head: &str, doc: &Doc) {
    ui.colored_label(theme::tone_ink(&Tone::Weak), head.to_owned());
    match doc {
        Doc::Absent => {
            ui.label(NO_BYTES);
        }
        other => {
            if let Some(said) = framing(other) {
                ui.colored_label(theme::NOTICE, said);
            }
            ui.label(egui::RichText::new(bytes(other)).monospace());
        }
    }
}

/// **The line a tool call wears**: which call it was, and whether it ended in
/// error — the engine's own fold of the exit code, never re-derived.
pub fn headline(call: &ToolCall) -> String {
    if call.is_error {
        return format!("{} — {ERRORED}", call.tool_id);
    }
    call.tool_id.clone()
}

/// The engine's own framing above a record it could not parse, or the seat's
/// sentence for a class it has no reading of. `None` for a record that parsed,
/// which needs no sentence at all.
pub fn framing(doc: &Doc) -> Option<String> {
    match doc {
        Doc::Unparsed { note, .. } => Some(note.clone()),
        Doc::Unknown(word) => Some(format!("a {word:?} record, which this seat cannot show")),
        Doc::Json { .. } | Doc::Absent => None,
    }
}

/// The bytes a record paints as. An absent record has none and is said in
/// words instead; a class this build has no reading of carries none either,
/// and its sentence above is the whole of what can be shown.
pub fn bytes(doc: &Doc) -> String {
    match doc {
        Doc::Json { raw } | Doc::Unparsed { raw, .. } => raw.clone(),
        Doc::Absent | Doc::Unknown(_) => String::new(),
    }
}

#[cfg(test)]
mod tests;
