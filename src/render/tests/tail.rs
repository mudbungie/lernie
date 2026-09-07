//! **The live tail, rendered** — the one read whose frames are printed as they
//! land, and therefore the one whose rendering has a memory.
//!
//! Split from [`super::shapes`] because it is a different question. That file
//! asks what a frame looks like; this one asks what a SEQUENCE of frames looks
//! like on a surface that cannot go back and amend a line it has printed —
//! which is the whole reason [`crate::render::tail`] takes a fold at all
//! (REMOTE §5.5: the closing half of a tool window restates neither the name
//! nor the input).

use serde_json::{Value, json};

use crate::render::{Form, rendered, tail};
use crate::reply::stream::Stream;

/// Render a frame and assert every needle is in it.
#[track_caller]
fn says(frame: &Value, needles: &[&str]) {
    let said = rendered(frame);
    for needle in needles {
        assert!(said.contains(needle), "{frame}\n-> {said}");
    }
}

/// **A follow frame renders what LANDED** (bl-f076), and a frame that landed
/// nothing is still the tail moving: the delta is a heartbeat, where the
/// frame's own shape said the same eight characters forever.
#[test]
fn a_follow_frame_is_the_text_that_landed_or_a_heartbeat() {
    let frame = |stream| json!({"ok": true, "kind": "follow", "tools": [], "stream": stream});
    assert_eq!(
        rendered(&frame(json!({"delta": "text", "text": "391"}))),
        "391"
    );
    assert_eq!(
        rendered(&frame(json!({"delta": "thinking", "thinking": "counting"}))),
        "(thinking) counting"
    );
    // **The heartbeat is the seat talking, so it rides the gutter** (bl-293d):
    // the model's prose above keeps the left margin, and the column is what
    // stops an eye reading a line to find out which of the two wrote it.
    assert_eq!(
        rendered(&frame(json!({"delta": "thinking"}))),
        "┊ …  thinking"
    );
    assert_eq!(rendered(&frame(json!({}))), "┊ …");
}

/// **The tool window is the half an operator watching their machines is
/// watching** (REMOTE §5.5, PROTOCOL 15). Every entry is a whole line — how it
/// ended, what ran and where (§5.1's `<client>_<tool>` carries the box), and
/// the input the engine already bounded — because a terminal cannot go back
/// and amend a line it printed a minute ago.
#[test]
fn a_follow_frame_says_what_is_running_where_and_how_it_ended() {
    says(
        &json!({"ok": true, "kind": "follow", "stream": {"delta": "text", "text": "on it"},
                "tools": [
                    {"tool_use": "toolu_01", "tool": "box2_Bash",
                     "input": "{\"command\":\"hostname && uptime\"}"}]}),
        &["on it", "┊ running", "box2_Bash", "hostname && uptime"],
    );
    says(
        &json!({"ok": true, "kind": "follow", "stream": {},
                "tools": [{"tool_use": "toolu_01", "tool": "box2_Bash",
                           "input": "{}", "exit_code": 2}]}),
        &["┊ exit 2", "box2_Bash"],
    );
    // A window with nothing in it is not a heartbeat suppressor: the frame
    // still landed nothing, and the delta is what says the tail is moving.
    assert_eq!(
        rendered(&json!({"ok": true, "kind": "follow", "tools": [],
                         "stream": {"delta": "thinking"}})),
        "┊ …  thinking"
    );
}

/// **A closing entry restates neither name nor input** (REMOTE §5.5), so a
/// frame read alone can only name the call by its id — and that is what it
/// says, rather than `exit 0` about nothing at all. [`crate::render::tail`] is
/// the surface that has the fold to do better, and its own test is next door.
#[test]
fn a_closing_entry_read_alone_is_named_by_its_invocation() {
    says(
        &json!({"ok": true, "kind": "follow", "stream": {},
                "tools": [{"tool_use": "toolu_01", "exit_code": 0}]}),
        &["exit 0", "toolu_01"],
    );
}

/// **The fold is what names the closing half** — the defect a per-frame
/// rendering cannot fix on its own, and the reason the command line's follower
/// holds one. Two frames of one read: the call opens in the first and lands in
/// the second, and the second says `box2_Bash` because the fold absorbed the
/// first.
#[test]
fn a_held_read_names_the_call_that_landed_out_of_the_fold() {
    let mut fold = Stream::default();
    let opened = tail(
        &json!({"ok": true, "kind": "follow", "stream": {"delta": "text", "text": "running it"},
                "tools": [{"tool_use": "toolu_01", "tool": "box2_Bash",
                           "input": "{\"command\":\"uptime\"}"}]}),
        &mut fold,
        Form::Rendered,
    );
    assert!(opened.contains("running"), "{opened}");
    assert!(opened.contains("box2_Bash"), "{opened}");
    let landed = tail(
        &json!({"ok": true, "kind": "follow", "stream": {},
                "tools": [{"tool_use": "toolu_01", "exit_code": 0}]}),
        &mut fold,
        Form::Rendered,
    );
    assert!(landed.contains("exit 0"), "{landed}");
    assert!(
        landed.contains("box2_Bash"),
        "the closing entry restates no name, so the fold must: {landed}"
    );
    assert!(
        !landed.contains("running it"),
        "the prose is the APPEND and never the accumulation: {landed}"
    );
}

/// **What a frame is not** goes through the same door: a refusal mid-stream and
/// bytes this build cannot read are said in this seat's own words, and `--json`
/// prints the frame exactly as it crossed whatever it turned out to be.
#[test]
fn a_held_read_says_a_refusal_and_prints_the_frame_where_asked() {
    let mut fold = Stream::default();
    let refusal = json!({"ok": false, "error": "unknown conversation"});
    assert!(
        tail(&refusal, &mut fold, Form::Rendered).contains("unknown conversation"),
        "a refusal mid-stream is the engine answering"
    );
    let unreadable = json!({"ok": true, "kind": "follow"});
    assert!(
        tail(&unreadable, &mut fold, Form::Rendered).contains("cannot read"),
        "and a frame with no fold in it is this seat's own statement"
    );
    assert_eq!(
        tail(&refusal, &mut fold, Form::Json),
        refusal.to_string(),
        "the machine form is the frame, byte for byte"
    );
}

/// **A parked call arrives on the lane rather than only at the end of it**
/// (REMOTE §5.5, PROTOCOL 18; yog bl-58bb). The park is a transition litany
/// lands no file for — a held invocation is stopped before the executor is
/// entered — so the window said nothing at the one moment the operator reading
/// this line was the thing it was waiting for. On a foot lane every call to a
/// non-shell tool is held, which makes it most of the conversation.
#[test]
fn a_follow_frame_says_a_call_the_boundary_parked_and_why() {
    says(
        &json!({"ok": true, "kind": "follow", "stream": {},
                "tools": [{"tool_use": "toolu_02", "tool": "box2_service_status",
                           "held": "box2_service_status {} classified opaque"}]}),
        &["┊ held", "box2_service_status", "classified opaque"],
    );
}

/// **The park is not terminal, and the fold is what makes that legible.** When
/// the operator answers it the call runs, and its opening and closing arrive
/// under the same `tool_use` — so a follower keyed on that id reads one call
/// going held → posted → complete, and the last word about it is the exit code
/// rather than the park it was released from.
#[test]
fn an_answered_park_reads_as_one_call_going_held_then_posted_then_complete() {
    let mut fold = Stream::default();
    let parked = tail(
        &json!({"ok": true, "kind": "follow", "stream": {},
                "tools": [{"tool_use": "toolu_02", "tool": "box2_service_status",
                           "held": "classified opaque"}]}),
        &mut fold,
        Form::Rendered,
    );
    assert!(parked.contains("held"), "{parked}");
    let ran = tail(
        &json!({"ok": true, "kind": "follow", "stream": {},
                "tools": [{"tool_use": "toolu_02", "exit_code": 0}]}),
        &mut fold,
        Form::Rendered,
    );
    assert!(ran.contains("exit 0"), "{ran}");
    assert!(
        ran.contains("box2_service_status"),
        "the closing entry restates no name, so the fold must: {ran}"
    );
}
