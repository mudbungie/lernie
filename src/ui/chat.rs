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
//!
//! The one thing that is **not** a row is a half of a turn with nothing in it
//! — see [`half`], which is that rule's one home. An empty half is not
//! something the operator was not shown; it is something that was never said.

use crate::reply::stream::Stream;
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript};

/// **The word this pane wears**, and the name of the column it is (bl-dfda).
/// It is painted by `crate::ui::shell` — above the pane in the broad shape, on
/// the navigation bar in the narrow one — because a column's name has one home
/// and which one it is depends on the shape.
pub const HEADING: &str = "conversation";

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
        } => blocks.iter().filter_map(|b| block(model_id, b)).collect(),
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
///
/// The two halves of a turn go through [`half`], so the committed path obeys
/// the rule the live one states. The other two blocks do not: a tool call with
/// an empty input still happened, and an unreadable block is deliberately blank
/// — dropping either would be the opposite defect, a transcript the operator
/// reads as shorter than it was.
fn block(model_id: &str, block: &Block) -> Option<Row> {
    match block {
        Block::Text(text) => half(model_id, "", text),
        Block::Thinking(text) => half(model_id, "thinking", text),
        Block::ToolUse { id, name, input } => Some(Row {
            who: format!("{model_id} → {name} {id}"),
            said: input.clone(),
        }),
        Block::Unknown(word) => Some(Row {
            who: format!("{model_id} ({word}, which this seat cannot read)"),
            said: String::new(),
        }),
    }
}

/// The live fold's rows.
fn streaming(stream: &Stream) -> Vec<Row> {
    live_rows(
        stream.thinking.as_deref().unwrap_or_default(),
        stream.text.as_deref().unwrap_or_default(),
    )
}

/// **The one rule for a half of a turn, and its one home.** Reasoning and
/// answer are each a row of its own and each omitted when it is empty: a model
/// that has only thought so far, or one that answered without reasoning. An
/// empty half is simply no row — never a blank one, which would claim something
/// was said.
///
/// It is stated once because a turn reaches this pane by two routes — the live
/// fold at write cadence, the committed entry at ask cadence — and a second
/// copy of the filter is a rule one of the two will stop obeying. It did: the
/// committed path painted a `(thinking)` header over nothing for every empty
/// thinking block the engine emitted, which reads as *the model thought
/// something and this seat lost it* (bl-beb7).
fn half(speaker: &str, mark: &str, said: &str) -> Option<Row> {
    (!said.is_empty()).then(|| Row {
        who: if mark.is_empty() {
            speaker.to_owned()
        } else {
            format!("{speaker} ({mark})")
        },
        said: said.to_owned(),
    })
}

/// The two halves of a turn in flight, by [`half`]'s rule.
fn live_rows(thinking: &str, text: &str) -> Vec<Row> {
    [("thinking", thinking), ("", text)]
        .into_iter()
        .filter_map(|(mark, said)| half(LIVE, mark, said))
        .collect()
}

/// Paint the pane. **The heading is the shell's** — see [`HEADING`].
pub fn render(ui: &mut egui::Ui, model: &crate::ui::Model) {
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
