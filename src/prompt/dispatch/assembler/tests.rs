//! Cover the §4.4 stream-assembly state machine: text + tool_use
//! folding, terminal `message_stop` / `error`, half-stream detection,
//! and tool-input JSON parsing.

use super::*;
use serde_json::json;

fn ev(value: serde_json::Value) -> StreamEvent {
    serde_json::from_value(value).unwrap()
}

#[test]
fn folds_text_deltas_into_a_single_text_block() {
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"message_start",
        "message":{"id":"m","model":"x","usage":{"input_tokens":3,"output_tokens":0}}})));
    a.feed(ev(
        json!({"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}),
    ));
    a.feed(ev(json!({"type":"text_delta","index":0,"text":"hi "})));
    a.feed(ev(json!({"type":"text_delta","index":0,"text":"there"})));
    a.feed(ev(json!({"type":"content_block_stop","index":0})));
    a.feed(ev(json!({"type":"message_stop","stop_reason":"end_turn",
        "usage":{"input_tokens":3,"output_tokens":4},"api_calls":1})));
    assert!(a.is_terminal());
    let c = a
        .into_completion()
        .unwrap_or_else(|_| panic!("expected stop"));
    assert_eq!(c.stop_reason, "end_turn");
    match &c.content[0] {
        ContentBlock::Text { text } => assert_eq!(text, "hi there"),
        other => panic!("expected text, got {other:?}"),
    }
}

#[test]
fn assembles_tool_use_block_from_partial_json_fragments() {
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"message_start",
        "message":{"id":"m","model":"x","usage":{"input_tokens":1,"output_tokens":0}}})));
    a.feed(ev(json!({"type":"content_block_start","index":0,
        "content_block":{"type":"tool_use","id":"toolu_01","name":"bash","input":{}}})));
    a.feed(ev(
        json!({"type":"tool_use_delta","index":0,"partial_json":"{\"cmd\":"}),
    ));
    a.feed(ev(
        json!({"type":"tool_use_delta","index":0,"partial_json":"\"ls\"}"}),
    ));
    a.feed(ev(json!({"type":"content_block_stop","index":0})));
    a.feed(ev(json!({"type":"message_stop","stop_reason":"tool_use",
        "usage":{"input_tokens":1,"output_tokens":2},"api_calls":1})));
    let c = a
        .into_completion()
        .unwrap_or_else(|_| panic!("expected stop"));
    match &c.content[0] {
        ContentBlock::ToolUse { id, name, input } => {
            assert_eq!(id, "toolu_01");
            assert_eq!(name, "bash");
            assert_eq!(input["cmd"], "ls");
        }
        other => panic!("expected tool_use, got {other:?}"),
    }
}

#[test]
fn empty_partial_json_yields_empty_object_input() {
    // Some upstream / stub sequences emit a content_block_start for a
    // tool_use with no ensuing input_json_delta. The assembler folds
    // this into an empty `input` object rather than a parse error.
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"content_block_start","index":0,
        "content_block":{"type":"tool_use","id":"t","name":"x","input":{}}})));
    a.feed(ev(json!({"type":"message_stop","stop_reason":"tool_use",
        "usage":{"input_tokens":1,"output_tokens":0},"api_calls":1})));
    let c = a
        .into_completion()
        .unwrap_or_else(|_| panic!("expected stop"));
    match &c.content[0] {
        ContentBlock::ToolUse { input, .. } => assert_eq!(input, &json!({})),
        other => panic!("expected tool_use, got {other:?}"),
    }
}

#[test]
fn surfaces_invalid_tool_input_json() {
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"content_block_start","index":0,
        "content_block":{"type":"tool_use","id":"t","name":"x","input":{}}})));
    a.feed(ev(
        json!({"type":"tool_use_delta","index":0,"partial_json":"{ not json"}),
    ));
    a.feed(ev(json!({"type":"message_stop","stop_reason":"tool_use",
        "usage":{"input_tokens":1,"output_tokens":0},"api_calls":1})));
    match a.into_completion() {
        Err(AssemblyError::ToolInputJson(_)) => {}
        Ok(_) => panic!("expected ToolInputJson"),
        Err(_) => panic!("expected ToolInputJson"),
    }
}

#[test]
fn unknown_block_types_pass_through_as_unknown() {
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"content_block_start","index":0,
        "content_block":{"type":"thinking","content":"…"}})));
    a.feed(ev(json!({"type":"message_stop","stop_reason":"end_turn",
        "usage":{"input_tokens":1,"output_tokens":1},"api_calls":1})));
    let c = a
        .into_completion()
        .unwrap_or_else(|_| panic!("expected stop"));
    assert_eq!(c.content.len(), 1);
    assert!(matches!(c.content[0], ContentBlock::Unknown));
}

#[test]
fn message_stop_inherits_stop_reason_from_message_start() {
    // §4.4: stop_reason may arrive on message_start, message_delta, or
    // the terminal message_stop. If only message_start carries it, the
    // assembler must still surface the value.
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"message_start",
        "message":{"id":"m","model":"x","stop_reason":"end_turn",
            "usage":{"input_tokens":2,"output_tokens":0}}})));
    a.feed(ev(json!({"type":"message_stop",
        "usage":{"input_tokens":2,"output_tokens":3},"api_calls":1})));
    let c = a
        .into_completion()
        .unwrap_or_else(|_| panic!("expected stop"));
    assert_eq!(c.stop_reason, "end_turn");
}

#[test]
fn missing_stop_reason_is_assembly_error() {
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"message_stop",
        "usage":{"input_tokens":1,"output_tokens":1},"api_calls":1})));
    match a.into_completion() {
        Err(AssemblyError::MissingStopReason) => {}
        _ => panic!("expected MissingStopReason"),
    }
}

#[test]
fn half_stream_detected_when_no_terminator_arrives() {
    let a = Assembler::new();
    match a.into_completion() {
        Err(AssemblyError::HalfStream) => {}
        _ => panic!("expected HalfStream"),
    }
}

#[test]
fn in_band_error_event_terminates_with_adapter_error() {
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"error","kind":"fatal",
        "http_status":401,"message":"boom"})));
    assert!(a.is_terminal());
    match a.into_completion() {
        Err(AssemblyError::Adapter {
            kind,
            message,
            http_status,
        }) => {
            assert_eq!(kind, "fatal");
            assert_eq!(message, "boom");
            assert_eq!(http_status, Some(401));
        }
        _ => panic!("expected Adapter"),
    }
}

#[test]
fn events_after_terminator_are_ignored() {
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"message_stop","stop_reason":"end_turn",
        "usage":{"input_tokens":1,"output_tokens":1},"api_calls":1})));
    // Late delta — the assembler must not lose terminal status nor
    // panic on the spurious event.
    a.feed(ev(json!({"type":"text_delta","index":0,"text":"too late"})));
    let c = a
        .into_completion()
        .unwrap_or_else(|_| panic!("expected stop"));
    assert!(c.content.is_empty());
}

#[test]
fn tool_use_delta_without_partial_json_is_ignored() {
    // v0.2 reserved this: a stray ToolUseDelta with no payload must
    // not corrupt the partial_json buffer.
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"content_block_start","index":0,
        "content_block":{"type":"tool_use","id":"t","name":"x","input":{}}})));
    a.feed(ev(json!({"type":"tool_use_delta","index":0})));
    a.feed(ev(
        json!({"type":"tool_use_delta","index":0,"partial_json":"{\"a\":1}"}),
    ));
    a.feed(ev(json!({"type":"message_stop","stop_reason":"tool_use",
        "usage":{"input_tokens":1,"output_tokens":1},"api_calls":1})));
    let c = a
        .into_completion()
        .unwrap_or_else(|_| panic!("expected stop"));
    match &c.content[0] {
        ContentBlock::ToolUse { input, .. } => assert_eq!(input["a"], 1),
        other => panic!("expected tool_use, got {other:?}"),
    }
}

#[test]
fn deltas_to_unknown_block_indices_are_silently_dropped() {
    // A stray delta arriving before its content_block_start (or for a
    // mistyped block) must not panic the assembler.
    let mut a = Assembler::new();
    a.feed(ev(json!({"type":"text_delta","index":3,"text":"x"})));
    a.feed(ev(
        json!({"type":"tool_use_delta","index":3,"partial_json":"y"}),
    ));
    a.feed(ev(json!({"type":"message_stop","stop_reason":"end_turn",
        "usage":{"input_tokens":1,"output_tokens":1},"api_calls":1})));
    let c = a
        .into_completion()
        .unwrap_or_else(|_| panic!("expected stop"));
    assert!(c.content.is_empty());
}
