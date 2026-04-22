//! End-to-end subprocess test for `lernie-provider-anthropic`.
//!
//! Covers the wiring that the inline unit tests in
//! `src/provider/anthropic_adapter.rs` cannot: argv parsing, stdin/stdout
//! as real pipes, env-var injection, exit codes. One happy-path and one
//! error-path case is enough — the adapter's branching logic has full
//! coverage in the library tests.

use httpmock::Method::POST;
use httpmock::MockServer;
use std::io::Write;
use std::process::{Command, Stdio};

fn adapter_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lernie-provider-anthropic")
}

#[test]
fn describe_subcommand_prints_contract_json() {
    let out = Command::new(adapter_bin())
        .arg("describe")
        .output()
        .expect("spawn adapter");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["name"], "anthropic");
    assert_eq!(v["schema_version"], 1);
    assert!(
        v["auth_env"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x == "ANTHROPIC_API_KEY")
    );
}

#[test]
fn complete_end_to_end_against_local_mock() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("x-api-key", "integration-key");
        then.status(200).body(
            r#"{"id":"msg_int","model":"claude-sonnet-4-7","stop_reason":"end_turn",
               "content":[{"type":"text","text":"pong"}],
               "usage":{"input_tokens":2,"output_tokens":1}}"#,
        );
    });

    let request = serde_json::json!({
        "model": "claude-sonnet-4-7",
        "max_tokens": 8,
        "messages": [{"role": "user", "content": "ping"}],
    });

    let mut child = Command::new(adapter_bin())
        .arg("complete")
        .arg("--endpoint")
        .arg(server.base_url())
        .env("ANTHROPIC_API_KEY", "integration-key")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn adapter");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(request.to_string().as_bytes())
        .unwrap();
    drop(child.stdin.take());

    let out = child.wait_with_output().expect("wait adapter");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["id"], "msg_int");
    assert_eq!(v["content"][0]["text"], "pong");
}

#[test]
fn complete_without_api_key_emits_in_band_fatal_error_and_exits_zero() {
    let mut child = Command::new(adapter_bin())
        .arg("complete")
        .env_remove("ANTHROPIC_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn adapter");
    drop(child.stdin.take());

    let out = child.wait_with_output().expect("wait adapter");
    // In-band errors are exit 0 per ARCH §4.4 — non-zero would tell the
    // harness the adapter itself crashed.
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["type"], "error");
    assert_eq!(v["kind"], "fatal");
    assert!(v["message"].as_str().unwrap().contains("ANTHROPIC_API_KEY"));
}
