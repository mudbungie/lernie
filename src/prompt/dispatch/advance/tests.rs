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
                },
            ],
        ),
    ];
    assert_eq!(warrant(&history), Warrant::Unpaired);
}
