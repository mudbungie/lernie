//! **What a wall holds**: the roster, the conversation list, the decision
//! queue and the live tail — four of the five reads a day of work is made of
//! (bl-6ae7). The fifth is the conversation itself, and it is next door
//! ([`super::chat`]) because a transcript is a different shape: a listing of
//! prose rather than a listing of rows.

use crate::reply::convs::ConvRow;
use crate::reply::queue::{Held, QueueRow};
use crate::reply::roster::Workspaces;
use crate::reply::stream::{Delta, Stream};

use super::parts::{age, brief, clause, line, line_over, listing, quoted, tally, things, when};

/// **The roster.** One row a wall, and the derivation's own currency under it.
///
/// The empty arm is a fact plus the one act that follows it: a world
/// bootstraps its first workspace when a conversation is started in it (yog
/// DESIGN §3.1, the fixed name `home`), so an empty roster is a knowable state
/// with exactly one sensible next act and this is where it is said (bl-b00f).
pub(super) fn workspaces(roster: &Workspaces) -> String {
    let rows = roster
        .rows
        .iter()
        .map(|row| {
            line(vec![
                Some(row.workspace.clone()),
                Some(row.kind.label()),
                things(row.agents, "conversation"),
                tally(row.attention, "waiting"),
                when(row.running, "running"),
                clause("pinned", row.pinned.map(|n| n.to_string()).as_deref()),
            ])
        })
        .collect();
    let head = line(vec![
        Some("workspaces".to_owned()),
        clause("(stale:", roster.stale.as_deref()).map(|said| format!("{said})")),
        clause("(growing:", roster.growth.as_deref()).map(|said| format!("{said})")),
    ]);
    listing(
        &head,
        rows,
        "none yet — `lernie start home \"<goal>\"` founds the first",
    )
}

/// **One wall's conversations.** The row a person scans, and the preview under
/// it: what was last said is what tells one row from another.
pub(super) fn conversations(rows: &[ConvRow]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            let head = line(vec![
                Some(row.display.clone()),
                Some(row.state.label()),
                when(row.uncertain, "(uncertain)"),
                Some(age(row.age_secs)),
                things(row.members, "member"),
                tally(row.depth, "deep"),
                tally(row.attention, "waiting"),
                clause("failed:", row.failure.as_deref().map(brief).as_deref()),
            ]);
            line_over(&head, quoted(&row.preview))
        })
        .collect();
    listing(
        "conversations",
        painted,
        "none yet — `lernie start <workspace> \"<goal>\"` begins one",
    )
}

/// **The decision queue**: everything waiting on the operator, anywhere.
///
/// The wall is on every row because the queue's subject is every wall at once
/// — it is the one read whose rows come from more than one of them.
pub(super) fn attention(rows: &[QueueRow]) -> String {
    let painted = rows
        .iter()
        .map(|row| {
            let head = line(vec![
                Some(format!("{}/{}", row.workspace, row.display)),
                Some(row.state.label()),
                when(row.uncertain, "(uncertain)"),
                Some(age(row.age_secs)),
                tally(row.pending, "queued"),
                when(!row.signals.is_empty(), &row.signals.join(", ")),
                clause("flagged:", row.flag.as_ref().map(|f| f.reason.as_str())),
                clause("failed:", row.failure.as_deref().map(brief).as_deref()),
            ]);
            line_over(&head, row.held.as_ref().map(parked))
        })
        .collect();
    listing("waiting on you", painted, "nothing is waiting on you")
}

/// A parked invocation, as the sentence that says what it is waiting for.
pub(super) fn parked(held: &Held) -> String {
    format!("held: {} ({}) — {}", held.tool, held.tool_use, held.reason)
}

/// **The live tail**, folded. The thinking and the text as they stand, and the
/// delta only as a note about which of them last moved — a frame is an APPEND
/// and the fold is the lane's (REMOTE §5.5), so what is printed is the whole
/// of what has landed and never the fragment that just arrived.
pub(super) fn follow(stream: &Stream) -> String {
    let body = tail(
        stream.thinking.as_deref().unwrap_or_default(),
        stream.text.as_deref().unwrap_or_default(),
    );
    line_over(
        &line(vec![
            Some("live".to_owned()),
            clause("(", stream.last_delta.as_ref().map(Delta::label).as_deref())
                .map(|said| format!("{said})")),
        ]),
        Some(body),
    )
}

/// A turn in progress: what it is thinking, and what it has said.
pub(super) fn tail(thinking: &str, text: &str) -> String {
    line(vec![
        when(
            !thinking.is_empty(),
            &line_over("thinking", Some(thinking.to_owned())),
        ),
        when(!text.is_empty(), text),
    ])
}
