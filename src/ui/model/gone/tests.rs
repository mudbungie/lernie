//! A selection the engine no longer knows (bl-6dfd), driven through the
//! frame's own side of the link with a scripted engine reply.

use std::time::Duration;

use crate::reply::read;
use crate::state::Link;
use crate::test_support::window::{own, seated};
use crate::ui::{Model, Notice};
use serde_json::json;

/// The engine's refusal of a read about a conversation it does not know.
fn unknown() -> crate::reply::Read {
    read(&json!({"ok": false, "error": "unknown conversation \"gone\""}))
}

/// Settle one beat on which the selection's `op` read was answered `said`.
fn beat(model: &mut Model, op: &str, said: crate::reply::Read) {
    let link = Link::new(Duration::from_millis(1));
    let selected = model.conversation.clone().expect("a selection");
    link.about(&own().channel, &selected, op, said);
    link.settle(model);
}

/// **A refused `agent` read drops the selection back to the wall** and
/// posts the refusal once: the transcript goes, the aim stands, and with
/// nothing selected nothing asks again — so Esc puts the notice down for
/// good.
#[test]
fn a_refused_row_read_retires_the_selection() {
    let mut model = seated();
    let aim = model.aim.clone();
    beat(&mut model, crate::verbs::AGENT.word, unknown());
    assert_eq!(model.conversation, None, "the selection is retired");
    assert!(model.transcript.entries.is_empty(), "its transcript went");
    assert_eq!(model.aim, aim, "back to the wall it was on");
    assert!(
        matches!(&model.notice, Some(Notice::Refused(said)) if said.contains("unknown conversation")),
        "{:?}",
        model.notice
    );
    model.escape();
    assert_eq!(model.notice, None, "Esc put it down");
    assert_eq!(
        crate::state::Standing::of(&model).conversation,
        None,
        "nothing is asked about it again"
    );
}

/// **Only the row read decides it**: a refused transcript read, or an
/// answered row, leaves the selection where it is.
#[test]
fn other_reads_leave_the_selection_standing() {
    let mut model = seated();
    let selected = model.conversation.clone();
    beat(&mut model, crate::verbs::TRANSCRIPT.word, unknown());
    assert_eq!(model.conversation, selected);
    beat(
        &mut model,
        crate::verbs::AGENT.word,
        read(&json!({"ok": true, "kind": "unheard-of"})),
    );
    assert_eq!(model.conversation, selected);
}
