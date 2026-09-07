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
use crate::ui::theme::Speaker;

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
    /// **Whose voice this is, told by weight and never by hue** (STYLE §5,
    /// `theme::speaker`): the ink of the rule beside the block and of the
    /// header over it. It is decided HERE, where the entry is read, because
    /// the paint knows a row and only this knows what kind of entry made one.
    pub weight: Speaker,
    /// **Whether this row is a failure.** The one thing a transcript says that
    /// is a STATE rather than a speaker — so it is the one row whose rule is
    /// an accent (`State::Error`) instead of a weight, and colour still means
    /// state.
    pub failed: bool,
    /// What this row hides while it is folded, or `None` where it hides
    /// nothing — see [`fold`].
    pub fold: Option<Fold>,
}

impl Row {
    /// **A row that hides nothing and did not fail** — everything a person, a
    /// model or this seat itself put on the glass. One constructor rather than
    /// a `fold: None` at eight sites, because a row that hides something is
    /// the exception and naming the exception is what keeps it visible.
    pub(super) fn plain(who: String, said: String, weight: Speaker) -> Self {
        Self {
            who,
            said,
            weight,
            failed: false,
            fold: None,
        }
    }
}

/// **The senders that are the operator's own.** litany writes a deposit from
/// the seat with `user` (`corpus/answers/transcript.json`), and `op` is the
/// same claim written short — the form a hand-written transcript and this
/// crate's own fixtures carry. Reading either as a peer would paint the
/// operator's own words in a peer's weight, which is the one weight on the
/// glass a reader is certain about.
const OPERATOR: [&str; 2] = ["user", "op"];

/// **Who a delivered message is, by weight**: the operator's own deposit is
/// their act and wears the brand, a sender the engine buried has ended, and
/// anything else that speaks here is a peer.
fn delivered(sender: &str, epitaph: Option<&String>) -> Speaker {
    if OPERATOR.contains(&sender) {
        Speaker::Operator
    } else if epitaph.is_some() {
        Speaker::Ended
    } else {
        Speaker::Peer
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
            let weight = delivered(sender, epitaph.as_ref());
            vec![Row::plain(
                match epitaph {
                    Some(word) => format!("{who} ({word})"),
                    None => who,
                },
                body.clone(),
                weight,
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
            // A tool answers for the model, not as it: a peer's weight.
            weight: Speaker::Peer,
            failed: *is_error,
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
            Speaker::Peer,
        )],
        // The two unreadables, held apart: the engine could not read this one,
        // and this build does not know that one. Only the second is fixed by an
        // upgrade, so they must not read alike.
        EntryKind::Raw => vec![Row::plain(
            format!("{} (unparsed)", entry.name),
            entry.raw.clone(),
            Speaker::Peer,
        )],
        EntryKind::Unknown(word) => vec![Row::plain(
            format!("{} ({word}, which this seat cannot read)", entry.name),
            entry.raw.clone(),
            Speaker::Peer,
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
            Speaker::Model,
        )),
        Block::Unknown(word) => Some(Row::plain(
            format!("{model_id} ({word}, which this seat cannot read)"),
            String::new(),
            Speaker::Model,
        )),
    }
}
