//! **What a wall holds**: the roster, the conversation list, the decision
//! queue and the live tail — four of the five reads a day of work is made of
//! (bl-6ae7). The fifth is the conversation itself, and it is next door
//! ([`super::chat`]) because a transcript is a different shape: a listing of
//! prose rather than a listing of rows.

use crate::reply::convs::ConvRow;
use crate::reply::queue::{Held, QueueRow};
use crate::reply::roster::Workspaces;
use crate::reply::stream::{Delta, Stream};

use super::parts::{
    age, brief, clause, line, line_over, listing, narrated, quoted, tally, things, when,
};

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

/// **One frame of the live tail: what LANDED, and nothing else.**
///
/// A frame is an APPEND (REMOTE §5.5) and a terminal is already a fold, so the
/// rendering is the appended text — printed in order, one frame a line, it
/// reads as the turn being written. Folding here and re-printing the whole
/// accumulation each frame would be the same answer at quadratic cost.
pub(super) fn follow(stream: &Stream) -> String {
    let landed = line(vec![
        clause("(thinking)", stream.thinking.as_deref()),
        stream.text.clone(),
    ]);
    if !landed.is_empty() {
        return landed;
    }
    // **A frame that carried no content is still the tail moving** (bl-f076).
    // A reasoning-heavy stretch sends `{"delta": "thinking"}` and nothing
    // else, over and over; printing the frame's own shape there said the same
    // eight characters forever and nothing about progress, and printing
    // nothing at all is the silence this word was fixed for. What is left to
    // say is which kind of work the engine says is going on, once per frame,
    // which is a heartbeat.
    //
    // **And it rides the gutter, because it is the seat talking about the
    // run** (bl-293d). On this lane the model's prose and the seat's own
    // narration arrive interleaved, a line at a time, and a column is what
    // stops an eye having to read one to find out which it was. The prose
    // above keeps the left margin; everything the seat says about the work
    // goes behind the mark, which is where the tool window's activity rows
    // land when the frame carries one (bl-183b).
    narrated(&line(vec![
        Some("…".to_owned()),
        stream.last_delta.as_ref().map(Delta::label),
    ]))
}

/// **The line a watch ends on** — which conversation, what state it came to
/// rest in, how long the watch held it, and what makes it move again, because
/// a watch that simply stopped is the silence the word was fixed for
/// (bl-3dca).
///
/// It is this seat's narration rather than any frame's rendering, so it rides
/// the same gutter the heartbeat does — and it carries the elapsed, because
/// *what did that turn take* is otherwise a number nobody has (bl-293d). The
/// count of tool calls belongs on it too and is not here: the frame carries no
/// call to count until bl-183b lands the tool window.
pub(crate) fn at_rest(agent: &str, state: &str, secs: i64) -> String {
    narrated(&format!(
        "{agent} is at rest ({state}) after {} — nothing more will arrive \
         until it is nudged or messaged",
        age(secs)
    ))
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
