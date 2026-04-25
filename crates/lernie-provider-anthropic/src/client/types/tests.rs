//! Wire-shape tests for [`super`].
//!
//! Lives next to [`super`] (rather than in the parent `client::tests`)
//! so the file stays under the repo's 300-line cap and so the type-only
//! tests are colocated with the type-only module — the HTTP-driven
//! tests in `client::tests` need a `MockServer`, these don't.

use super::*;

#[test]
fn response_text_joins_text_blocks_in_order() {
    let text = Response {
        id: "msg_1".into(),
        model: "claude-sonnet-4-7".into(),
        stop_reason: "end_turn".into(),
        content: vec![
            ContentBlock::Text {
                text: "hello ".into(),
            },
            ContentBlock::Unknown,
            ContentBlock::Text {
                text: "world".into(),
            },
        ],
        usage: Usage {
            input_tokens: 1,
            output_tokens: 2,
        },
    }
    .text();
    assert_eq!(text, "hello world");
}

#[test]
fn request_serializes_with_expected_fields() {
    let v: serde_json::Value = serde_json::to_value(&Request {
        model: "claude-sonnet-4-7".into(),
        max_tokens: 16,
        system: Some("be terse".into()),
        messages: vec![Message {
            role: Role::User,
            content: "hi".into(),
        }],
        tools: None,
    })
    .unwrap();
    assert_eq!(v["model"], "claude-sonnet-4-7");
    assert_eq!(v["max_tokens"], 16);
    assert_eq!(v["system"], "be terse");
    assert_eq!(v["messages"][0]["role"], "user");
    assert_eq!(v["messages"][0]["content"], "hi");
    assert!(v.get("tools").is_none(), "tools omitted when None");
}

#[test]
fn request_omits_system_when_absent() {
    let v: serde_json::Value = serde_json::to_value(&Request {
        model: "m".into(),
        max_tokens: 1,
        system: None,
        messages: vec![],
        tools: None,
    })
    .unwrap();
    assert!(v.get("system").is_none());
}

#[test]
fn request_serializes_tools_array_when_present() {
    let v: serde_json::Value = serde_json::to_value(&Request {
        model: "claude-sonnet-4-7".into(),
        max_tokens: 16,
        system: None,
        messages: vec![],
        tools: Some(vec![ToolDecl {
            name: "bash".into(),
            description: "Run a shell command.".into(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "cmd": { "type": "string" } },
                "required": ["cmd"]
            }),
        }]),
    })
    .unwrap();
    assert_eq!(v["tools"][0]["name"], "bash");
    assert_eq!(v["tools"][0]["description"], "Run a shell command.");
    assert_eq!(v["tools"][0]["input_schema"]["type"], "object");
    assert_eq!(v["tools"][0]["input_schema"]["required"][0], "cmd");
}

#[test]
fn message_content_round_trips_string_and_block_array() {
    // String — the v0.1 shape, still legal.
    let m: Message = serde_json::from_str(r#"{"role":"user","content":"hello"}"#).unwrap();
    assert!(matches!(m.content, MessageContent::Text(ref s) if s == "hello"));
    let bytes = serde_json::to_vec(&m).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(raw["content"], "hello", "string survives the round-trip");

    // Block array — required for v0.3 user messages carrying tool_result.
    let m: Message = serde_json::from_str(
        r#"{"role":"user","content":[
            {"type":"tool_result","tool_use_id":"t1","content":"42"},
            {"type":"text","text":"see above"}
        ]}"#,
    )
    .unwrap();
    let MessageContent::Blocks(blocks) = &m.content else {
        panic!("expected Blocks");
    };
    assert_eq!(blocks.len(), 2);
    assert!(
        matches!(blocks[0], ContentBlock::ToolResult { ref tool_use_id, .. } if tool_use_id == "t1")
    );
    assert!(matches!(blocks[1], ContentBlock::Text { ref text } if text == "see above"));
}

#[test]
fn tool_use_block_round_trips_with_arbitrary_input() {
    let block: ContentBlock = serde_json::from_str(
        r#"{"type":"tool_use","id":"toolu_01","name":"read_file",
            "input":{"path":"/etc/hostname","limit":10}}"#,
    )
    .unwrap();
    let bytes = serde_json::to_vec(&block).unwrap();
    let back: ContentBlock = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(block, back);
    match back {
        ContentBlock::ToolUse { id, name, input } => {
            assert_eq!(id, "toolu_01");
            assert_eq!(name, "read_file");
            assert_eq!(input["path"], "/etc/hostname");
            assert_eq!(input["limit"], 10);
        }
        other => panic!("expected ToolUse, got {other:?}"),
    }
}

#[test]
fn tool_result_block_skips_default_is_error_on_the_wire() {
    let ok = ContentBlock::ToolResult {
        tool_use_id: "t1".into(),
        content: "ok".into(),
        is_error: false,
    };
    let raw: serde_json::Value = serde_json::to_value(&ok).unwrap();
    assert!(
        raw.get("is_error").is_none(),
        "is_error: false omitted on the wire"
    );

    let bad = ContentBlock::ToolResult {
        tool_use_id: "t2".into(),
        content: "boom".into(),
        is_error: true,
    };
    let raw: serde_json::Value = serde_json::to_value(&bad).unwrap();
    assert_eq!(raw["is_error"], true);
}

#[test]
fn unknown_content_block_type_deserializes_to_unknown() {
    let json = r#"{"type":"something_new","foo":"bar"}"#;
    let block: ContentBlock = serde_json::from_str(json).unwrap();
    assert_eq!(block, ContentBlock::Unknown);
}
