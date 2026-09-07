//! **What a wall holds**: the roster, the conversation list, the decision
//! queue and the live tail — four of the five reads a day of work is made of
//! (bl-6ae7). The fifth is the conversation itself, and it is next door
//! ([`super::chat`]) because a transcript is a different shape: a listing of
//! prose rather than a listing of rows.

use crate::reply::queue::{Held, QueueRow};
use crate::reply::roster::Workspaces;
use crate::reply::stream::window::Window;
use crate::reply::stream::{Delta, Stream};

use super::parts::{age, brief, clause, line, line_over, listing, narrated, tally, things, when};

/// The conversation index and its fold, which is a listing with a rule of its
/// own (bl-96cd) — split from here at the 300-line cap on the seam this
/// module's doc already draws between its four reads.
mod convs;

pub(super) use convs::conversations;

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
///
/// **The tool window rides under the prose** (REMOTE §5.5, PROTOCOL 15), and
/// it is the half an operator watching an agent administer their machines is
/// actually watching: prose alone said "thinking" and two sentences while
/// eight commands ran on two boxes. Each entry is a whole line — what ran,
/// where, and how it ended — because a terminal cannot go back and amend the
/// line it printed a minute ago; the pairing an interface would do by
/// redrawing a row is done by [`Stream::appended`] before this is called.
pub(super) fn follow(stream: &Stream) -> String {
    let landed = line_over(
        &line(vec![
            clause("(thinking)", stream.thinking.as_deref()),
            stream.text.clone(),
        ]),
        // **The window rides the gutter** (bl-293d): what a tool is doing is
        // the seat narrating the run, not the model's answer, and on this lane
        // the two arrive interleaved a line at a time.
        Some(narrated(
            &stream
                .tools
                .iter()
                .map(running)
                .collect::<Vec<String>>()
                .join("\n"),
        )),
    );
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

/// **The line a watch ends on when the conversation is HELD** (bl-3a1f; REMOTE
/// §5.5, PROTOCOL 18) — which is a rest, and is not the rest above.
///
/// The state read alone says `quiescent`, and [`at_rest`]'s sentence is then
/// false in the only two words that matter: something more *will* arrive, and
/// neither remedy it names will make it. A park is the conversation waiting on
/// the **operator**, who is the person reading this line, so the line names the
/// call, the control's own reason for holding it, and the one gesture that
/// lifts it. On a foot lane every call to a non-shell tool is held, which makes
/// this most of the endings rather than a corner.
///
/// The remedy is spelled with the address the watch was aimed at, because a
/// sentence an operator has to complete from memory is a sentence they will
/// complete wrongly.
pub(crate) fn held_at(agent: &str, workspace: &str, held: &Held, secs: i64) -> String {
    narrated(&format!(
        "{agent} is holding {} ({}) for your answer after {} — {}; release or decline it \
         with `lernie answer {workspace} {agent} pass|refuse` (a third word widens the \
         answer to every call like it)",
        held.tool,
        held.tool_use,
        age(secs),
        held.reason
    ))
}

/// **The line a watch says while it waits for a driver to take mail** (bl-87ab,
/// bl-3ecd) — which conversation, the rest the engine reported, and how much is
/// waiting. It is said once per watch, not once per look: the hold is the
/// answer and a quarter-second drumbeat of it would be worse than the silence.
///
/// It names the state the engine gave rather than hiding it, because that
/// reading is true and it is the one an operator will see again in
/// `conversations` a second later. What it adds is the fact that makes the
/// at-rest sentence false here: mail nobody has taken yet.
pub(crate) fn waiting_on_mail(agent: &str, state: &str, deposits: usize) -> String {
    narrated(&format!(
        "{agent} is at rest ({state}) with {} not yet taken — holding until a driver takes \
         it; Ctrl-C to stop watching",
        // The caller only says this line when something IS waiting — a rest
        // with an empty inbox ends the watch instead — so the total form of
        // `things` is the arm that never renders rather than a second sentence.
        things(u64::try_from(deposits).unwrap_or(u64::MAX), "deposit").unwrap_or_default()
    ))
}

/// **One tool call, whole.** What ran and on which machine — REMOTE §5.1
/// presents a routed tool as `<client>_<tool>`, so the name is the box — the
/// input the engine already bounded, and how it ended.
///
/// **The status is the presence of an exit code** and never a third reading:
/// absent is a call in flight. And a call whose opening half this reader never
/// saw is named by its id rather than dropped — the id is what the frame
/// carried, and a line that said `exit 0` about nothing at all would be worse
/// than one that says which invocation.
fn running(call: &Window) -> String {
    line(vec![
        Some(status(call)),
        Some(call.tool.clone().unwrap_or_else(|| call.tool_use.clone())),
        call.input.as_deref().map(brief),
        call.held.as_deref().map(brief),
    ])
}

/// **The word a call is in, and the order is the order the transitions
/// happen in** (REMOTE §5.5, PROTOCOL 18). A capture that landed is the last
/// word about a call, so an exit code outranks the park it was released from;
/// a park outranks *running*, because a held call is not running and saying it
/// was is the whole defect this closes — on a foot lane every call to a
/// non-shell tool is held, so it is most of the conversation.
fn status(call: &Window) -> String {
    if let Some(code) = call.exit_code {
        return format!("exit {code}");
    }
    if call.held.is_some() {
        return HELD.to_owned();
    }
    RUNNING.to_owned()
}

/// What a call with no exit code yet is said to be doing.
const RUNNING: &str = "running";

/// What a call the capability boundary parked is said to be doing — the agent
/// row's own word (`parked`, above), so one park reads the same on both reads.
const HELD: &str = "held";

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
