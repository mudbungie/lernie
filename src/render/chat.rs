//! **The conversation, as prose**: its transcript and its own whole row.
//!
//! Split from [`super::walls`] at the design-time budget on the seam the two
//! shapes already have. A wall's reads are listings of ROWS — a line each,
//! scanned. A conversation is a listing of what was SAID: arbitrary prose from
//! a model and a person, printed rather than summarised, because eliding it is
//! eliding the answer itself.

use crate::reply::agent::{Agent, Offer};
use crate::reply::transcript::{Block, Entry, EntryKind, Transcript};

use super::parts::{brief, clause, line, line_over, listing, when};
use super::walls::{parked, tail};

/// **The conversation itself.** One block an entry, its name over what it
/// says — and the `raw` bytes every entry carries beside its reading are not
/// printed, which is the whole of why a transcript was unreadable in a
/// terminal (bl-6ae7): it was every entry twice, on one line.
pub(super) fn transcript(transcript: &Transcript) -> String {
    let entries = transcript.entries.iter().map(entry).collect();
    listing("transcript", entries, "nothing has been said yet")
}

/// One entry: the file it is, what kind of entry it is, and what it says
/// under both. **One head and one body**, never a head over a head — an entry
/// is already indented under the transcript, and a third step in is a shape an
/// eye stops following.
fn entry(entry: &Entry) -> String {
    let (clause, body) = reading(&entry.kind);
    line_over(&line(vec![Some(entry.name.clone()), Some(clause)]), body)
}

/// **What one entry is, and what it says** — the clause that joins its name
/// line, and the prose under it.
fn reading(kind: &EntryKind) -> (String, Option<String>) {
    match kind {
        EntryKind::Delivered {
            sender,
            sender_name,
            epitaph,
            body,
        } => (
            line(vec![
                Some(format!(
                    "from {}",
                    crate::reply::transcript::said_by(sender, sender_name.as_deref())
                )),
                clause("epitaph:", epitaph.as_deref()),
            ]),
            Some(body.clone()),
        ),
        EntryKind::Model {
            model_id,
            blocks,
            usage,
        } => (
            line(vec![
                Some(model_id.clone()),
                when(!usage.is_empty(), &counts(usage)),
            ]),
            Some(blocks.iter().map(block).collect::<Vec<String>>().join("\n")),
        ),
        EntryKind::ToolResult {
            tool_use_id,
            content,
            is_error,
        } => (
            line(vec![
                Some(format!("result of {tool_use_id}")),
                when(*is_error, "ERROR"),
            ]),
            Some(content.clone()),
        ),
        EntryKind::Streaming { thinking, text } => {
            ("in flight".to_owned(), Some(tail(thinking, text)))
        }
        EntryKind::Compacted {
            first,
            last,
            summary,
        } => (
            format!("compacted {first}\u{2013}{last}"),
            Some(summary.clone()),
        ),
        EntryKind::Raw => ("raw".to_owned(), None),
        EntryKind::Unknown(word) => (
            format!("{word} (this seat has no reading of that entry)"),
            None,
        ),
    }
}

/// One block of a model's turn.
fn block(block: &Block) -> String {
    match block {
        Block::Text(text) => text.clone(),
        Block::Thinking(text) => line_over("thinking", Some(text.clone())),
        Block::ToolUse { id, name, input } => {
            line_over(&format!("{name} ({id})"), Some(input.clone()))
        }
        Block::Unknown(word) => format!("[{word}]"),
    }
}

/// The provider's own counters, as one clause.
fn counts(usage: &std::collections::BTreeMap<String, u64>) -> String {
    usage
        .iter()
        .map(|(name, count)| format!("{name} {count}"))
        .collect::<Vec<String>>()
        .join(" ")
}

/// **The conversation's own row, whole** — the deepest read of one, and the
/// only place the engine's own OFFERS are printed, which is what tells an
/// operator that a stop with children is on the table (bl-9fd1).
pub(super) fn agent(agent: &Agent) -> String {
    let facts = line(vec![
        Some(agent.display.clone()),
        Some(agent.state.label()),
        when(agent.display_only, "(display only)"),
        when(agent.refused, "refused"),
        when(agent.present, "present"),
        clause("tip", Some(&agent.tip)),
        clause("flight:", agent.flight.as_deref()),
        clause("failed:", agent.failure.as_deref().map(brief).as_deref()),
        when(!agent.marks.is_empty(), &agent.marks.join(", ")),
        clause("spend", Some(&agent.spend.tokens.total.to_string())),
        clause(
            "context",
            agent
                .context
                .as_ref()
                .map(|c| format!("{}% of {}", c.percent, c.model))
                .as_deref(),
        ),
    ]);
    let more = [
        agent.held.as_ref().map(parked),
        agent
            .strip
            .as_ref()
            .map(|strip| format!("{}: {}", strip.class, strip.facts)),
        when(
            !agent.seats.is_empty(),
            &agent
                .seats
                .iter()
                .map(|seat| format!("{} — {}", seat.name, seat.doing))
                .collect::<Vec<String>>()
                .join("\n"),
        ),
        when(!agent.offers.is_empty(), &offered(&agent.offers)),
    ];
    listing(&facts, more.into_iter().flatten().collect(), "")
}

/// The gates, as the words that reach them.
fn offered(offers: &[Offer]) -> String {
    format!(
        "offers: {}",
        offers
            .iter()
            .map(|offer| match offer {
                Offer::Nudge => "nudge",
                Offer::Stop => "stop",
                Offer::Children => "stop children",
            })
            .collect::<Vec<&str>>()
            .join(", ")
    )
}
