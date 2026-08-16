//! Warrant-derivation unit tests (§6 hop step 3): the wire alternation
//! is the whole answer. Full hop flows are in `prompt::tests::advance`
//! (they need the shared adapter/tool stubs).

use super::{Warrant, warrant};
use brazen::{Content, Message, Role};
use serde_json::json;

fn msg(role: Role, content: Vec<Content>) -> Message {
    Message { role, content }
}

#[test]
fn empty_history_warrants_nothing() {
    assert_eq!(warrant(&[]), Warrant::NothingDue);
}

#[test]
fn user_side_tail_means_a_model_call_is_due() {
    // Delivered mail and committed tool results both compose user-side
    // (§2.3), so this one arm covers reprompt and post-tools alike.
    let history = vec![
        msg(Role::Assistant, vec![Content::Text("done?".into())]),
        msg(Role::User, vec![Content::Text("more".into())]),
    ];
    assert_eq!(warrant(&history), Warrant::ModelCallDue);
}

#[test]
fn assistant_tail_without_tool_use_warrants_nothing() {
    let history = vec![
        msg(Role::User, vec![Content::Text("hi".into())]),
        msg(Role::Assistant, vec![Content::Text("final".into())]),
    ];
    assert_eq!(warrant(&history), Warrant::NothingDue);
}

#[test]
fn assistant_tail_with_unmatched_tool_use_is_the_non_replayable_state() {
    let history = vec![
        msg(Role::User, vec![Content::Text("hi".into())]),
        msg(
            Role::Assistant,
            vec![
                Content::Text("running".into()),
                Content::ToolUse {
                    id: "t1".into(),
                    name: "bash".into(),
                    input: json!({"command": "true"}),
                    signature: None,
                },
            ],
        ),
    ];
    assert_eq!(warrant(&history), Warrant::Unpaired);
}

#[test]
fn delivered_mail_behind_an_unpaired_window_does_not_mask_the_decline() {
    // The bl-15f0 shape: `driver::deliver` ran before the derivation, so
    // the crash-orphaned window is buried user-side. The tail says
    // "model call due"; the pairing scan must overrule it — sent anyway,
    // the provider rejects the history forever.
    let history = vec![
        msg(
            Role::Assistant,
            vec![Content::ToolUse {
                id: "t1".into(),
                name: "bash".into(),
                input: json!({"command": "true"}),
                signature: None,
            }],
        ),
        msg(Role::User, vec![Content::Text("hello?".into())]),
    ];
    assert_eq!(warrant(&history), Warrant::Unpaired);
}

#[test]
fn a_paired_window_mid_history_is_no_unpaired_state() {
    // The ordinary post-tools shape: every `tool_use` matched by a
    // committed `tool_result`, wherever the window sits — the pairing
    // scan stays quiet and the tail speaks (§2.3).
    let history = vec![
        msg(
            Role::Assistant,
            vec![Content::ToolUse {
                id: "t1".into(),
                name: "bash".into(),
                input: json!({"command": "true"}),
                signature: None,
            }],
        ),
        msg(
            Role::Tool,
            vec![Content::ToolResult {
                tool_use_id: "t1".into(),
                content: vec![Content::Text("ok".into())],
                is_error: false,
            }],
        ),
        msg(Role::User, vec![Content::Text("more".into())]),
    ];
    assert_eq!(warrant(&history), Warrant::ModelCallDue);
}

/// The retarget report (§2.2): the hop consumes the mark on every
/// boundary, and only a decline is worth a line — the landing and the
/// no-op are silent, and an unmarked branch has nothing to say at all.
mod retarget_report {
    use super::super::report_retarget;
    use crate::prompt::retarget::Outcome;

    #[test]
    fn every_outcome_is_reportable_and_only_the_decline_speaks() {
        report_retarget("a", None);
        report_retarget("a", Some(Outcome::Landed));
        report_retarget("a", Some(Outcome::NoOp));
        report_retarget(
            "a",
            Some(Outcome::Conflicted(vec!["summary/001.md".into()])),
        );
    }
}
