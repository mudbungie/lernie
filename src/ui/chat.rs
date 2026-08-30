//! **The chat pane**: one conversation, entry by entry.
//!
//! Every entry becomes rows of `(who said it, what was said)`, and the
//! projection is a **pure function** of the transcript and the live fold — so
//! what the pane shows is a value a test reads back, and the paint below it is
//! thin enough to have nothing of its own to get wrong.
//!
//! **Nothing is dropped, including what nothing could parse.** An entry the
//! engine could not read is surfaced as its raw bytes, and an entry of a kind
//! this build does not know is surfaced as its own word beside them — the reply
//! vocabulary's rung 3, on the glass. A transcript that quietly skipped an
//! entry would be a conversation the operator reads as shorter than it was,
//! which is the one failure a transcript must not have.

use crate::reply::stream::Stream;
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript};

/// What the pane says with no conversation selected.
pub const NO_CONVERSATION: &str = "pick a conversation";
/// The name the live tail wears — no file backs it.
pub const LIVE: &str = "«live»";

/// One painted row: who is speaking, and what they said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub who: String,
    pub said: String,
}

/// **The whole pane as data.**
///
/// The live fold **replaces** any streaming entry the committed read already
/// folded on, rather than appending beside it: the tail reaches a seat by two
/// routes at two cadences — the pull read folds one on at ask cadence, the
/// follow lane delivers a newer one at write cadence — and *the newest fold
/// wins* is the only reconciliation either needs. Appending would paint the
/// answer twice.
pub fn rows(transcript: &Transcript, live: Option<&Stream>) -> Vec<Row> {
    let mut out: Vec<Row> = transcript
        .entries
        .iter()
        .filter(|entry| live.is_none() || !matches!(entry.kind, EntryKind::Streaming { .. }))
        .flat_map(entry_rows)
        .collect();
    out.extend(live.into_iter().flat_map(streaming));
    out
}

/// One entry's rows.
fn entry_rows(entry: &Entry) -> Vec<Row> {
    match &entry.kind {
        EntryKind::Delivered {
            sender,
            epitaph,
            body,
        } => vec![Row {
            who: match epitaph {
                Some(word) => format!("{sender} ({word})"),
                None => sender.clone(),
            },
            said: body.clone(),
        }],
        EntryKind::Model {
            model_id, blocks, ..
        } => blocks.iter().map(|b| block(model_id, b)).collect(),
        EntryKind::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => vec![Row {
            who: format!(
                "{tool_use_id} {}",
                if *is_error { "failed" } else { "returned" }
            ),
            said: content.clone(),
        }],
        EntryKind::Streaming { thinking, text } => live_rows(thinking, text),
        EntryKind::Compacted {
            first,
            last,
            summary,
        } => vec![Row {
            who: format!("compacted {first}–{last}"),
            said: summary.clone(),
        }],
        // The two unreadables, held apart: the engine could not read this one,
        // and this build does not know that one. Only the second is fixed by an
        // upgrade, so they must not read alike.
        EntryKind::Raw => vec![Row {
            who: format!("{} (unparsed)", entry.name),
            said: entry.raw.clone(),
        }],
        EntryKind::Unknown(word) => vec![Row {
            who: format!("{} ({word}, which this seat cannot read)", entry.name),
            said: entry.raw.clone(),
        }],
    }
}

/// One content block. Reasoning is a **row**, not a spinner: a badge that never
/// grows cannot tell a model thinking hard from a driver that has hung.
fn block(model_id: &str, block: &Block) -> Row {
    match block {
        Block::Text(text) => Row {
            who: model_id.to_owned(),
            said: text.clone(),
        },
        Block::Thinking(text) => Row {
            who: format!("{model_id} (thinking)"),
            said: text.clone(),
        },
        Block::ToolUse { id, name, input } => Row {
            who: format!("{model_id} → {name} {id}"),
            said: input.clone(),
        },
        Block::Unknown(word) => Row {
            who: format!("{model_id} ({word}, which this seat cannot read)"),
            said: String::new(),
        },
    }
}

/// The live fold's rows.
fn streaming(stream: &Stream) -> Vec<Row> {
    live_rows(
        stream.thinking.as_deref().unwrap_or_default(),
        stream.text.as_deref().unwrap_or_default(),
    )
}

/// The two halves of a turn in flight, each a row of its own and each omitted
/// when it is empty: a model that has only thought so far, or one that answered
/// without reasoning. An empty half is simply no row — never a blank one, which
/// would claim something was said.
fn live_rows(thinking: &str, text: &str) -> Vec<Row> {
    [("thinking", thinking), ("", text)]
        .into_iter()
        .filter(|(_, said)| !said.is_empty())
        .map(|(mark, said)| Row {
            who: if mark.is_empty() {
                LIVE.to_owned()
            } else {
                format!("{LIVE} ({mark})")
            },
            said: said.to_owned(),
        })
        .collect()
}

/// Paint the pane.
pub fn render(ui: &mut egui::Ui, model: &crate::ui::Model) {
    ui.heading("conversation");
    if model.conversation.is_none() {
        ui.label(NO_CONVERSATION);
        return;
    }
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for row in rows(&model.transcript, model.live.as_ref()) {
                ui.separator();
                ui.strong(row.who);
                ui.label(row.said);
            }
        });
}

#[cfg(test)]
mod tests;
