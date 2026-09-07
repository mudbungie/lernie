//! **The chat pane as data** — every entry of a transcript, and the live fold,
//! as the rows the pane paints.
//!
//! Split from [`super`] at the design-time budget on the seam that module's
//! own doc draws: this is the **projection**, a pure function of the
//! transcript and the fold, and [`super`] is the pane that paints it. The two
//! change for different reasons — an entry kind the engine grows changes this,
//! a control the pane grows changes that — and the split is what keeps a
//! reply-vocabulary change from touching any paint at all.
//!
//! **Nothing is dropped, including what nothing could parse.** An entry the
//! engine could not read is surfaced as its raw bytes, and an entry of a kind
//! this build does not know is surfaced as its own word beside them — the
//! reply vocabulary's rung 3, on the glass. A transcript that quietly skipped
//! an entry would be a conversation the operator reads as shorter than it was,
//! which is the one failure a transcript must not have.
//!
//! The one thing that is **not** a row is a half of a turn with nothing in it
//! — see [`half`], which is that rule's one home. An empty half is not
//! something the operator was not shown; it is something that was never said.

use super::{Fold, fold};

/// The turn that has not settled yet.
mod live;

use crate::reply::stream::Stream;
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript};
use live::{half, live_rows, streaming};

/// One painted row: who is speaking, and what they said.
///
/// **[`Row::fold`] is what makes a machine's answer machinery** (bl-90d0). It
/// is `None` for everything a person or a model said, whatever its length — a
/// long prose answer is the thing the pane is opened for — and `Some` only for
/// a tool result big enough to have something to hide. So the pane folds by
/// KIND and never by size, which is the distinction that matters: a
/// seven-hundred-line `bl --help` is not a long answer, it is bookkeeping in
/// front of one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub who: String,
    pub said: String,
    /// What this row hides while it is folded, or `None` where it hides
    /// nothing — see [`fold`].
    pub fold: Option<Fold>,
}

impl Row {
    /// **A row that hides nothing** — everything a person, a model or this
    /// seat itself put on the glass. One constructor rather than a `fold: None`
    /// at eight sites, because a row that hides something is the exception and
    /// naming the exception is what keeps it visible.
    fn plain(who: String, said: String) -> Self {
        Self {
            who,
            said,
            fold: None,
        }
    }
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
            sender_name,
            epitaph,
            body,
        } => {
            let who = crate::reply::transcript::said_by(sender, sender_name.as_deref());
            vec![Row::plain(
                match epitaph {
                    Some(word) => format!("{who} ({word})"),
                    None => who,
                },
                body.clone(),
            )]
        }
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
            // **The one row in this pane that hides anything**, and only when
            // there is something to hide (`fold`).
            fold: fold::of(content),
        }],
        EntryKind::Streaming { thinking, text } => live_rows(thinking, text),
        EntryKind::Compacted {
            first,
            last,
            summary,
        } => vec![Row::plain(
            format!("compacted {first}–{last}"),
            summary.clone(),
        )],
        // The two unreadables, held apart: the engine could not read this one,
        // and this build does not know that one. Only the second is fixed by an
        // upgrade, so they must not read alike.
        EntryKind::Raw => vec![Row::plain(
            format!("{} (unparsed)", entry.name),
            entry.raw.clone(),
        )],
        EntryKind::Unknown(word) => vec![Row::plain(
            format!("{} ({word}, which this seat cannot read)", entry.name),
            entry.raw.clone(),
        )],
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
        Block::ToolUse { id, name, input } => Some(Row::plain(
            format!("{model_id} → {name} {id}"),
            input.clone(),
        )),
        Block::Unknown(word) => Some(Row::plain(
            format!("{model_id} ({word}, which this seat cannot read)"),
            String::new(),
        )),
    }
}
