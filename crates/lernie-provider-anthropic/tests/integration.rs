//! End-to-end subprocess test for `lernie-provider-anthropic`.
//!
//! Covers the wiring that the inline unit tests in `src/adapter.rs`
//! cannot: argv parsing, stdin/stdout as real pipes, env-var injection,
//! exit codes. One happy-path and one error-path case is enough — the
//! adapter's branching logic has full coverage in the library tests.

use httpmock::Method::POST;
use httpmock::MockServer;
use serde_json::Value;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;

fn adapter_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lernie-provider-anthropic")
}

fn spawn_and_capture(mut cmd: Command, stdin_bytes: Option<&[u8]>) -> (bool, Vec<u8>) {
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().expect("spawn adapter");
    if let Some(bytes) = stdin_bytes {
        child.stdin.as_mut().unwrap().write_all(bytes).unwrap();
    }
    drop(child.stdin.take());
    let out = child.wait_with_output().expect("wait adapter");
    (out.status.success(), out.stdout)
}

#[test]
fn describe_subcommand_prints_contract_json() {
    let mut cmd = Command::new(adapter_bin());
    cmd.arg("describe");
    let (ok, stdout) = spawn_and_capture(cmd, None);
    assert!(ok);
    let v: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(v["name"], "anthropic");
    assert_eq!(v["schema_version"], 2);
    let auth_env: Vec<&str> = v["auth_env"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    assert!(auth_env.contains(&"ANTHROPIC_API_KEY"));
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

    let mut cmd = Command::new(adapter_bin());
    cmd.arg("complete")
        .env("ANTHROPIC_API_KEY", "integration-key")
        .env("LERNIE_PROVIDER_ANTHROPIC_ENDPOINT", server.base_url());
    let (ok, stdout) = spawn_and_capture(cmd, Some(request.to_string().as_bytes()));
    assert!(ok);
    let v: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(v["id"], "msg_int");
    assert_eq!(v["content"][0]["text"], "pong");
}

#[test]
fn complete_request_flag_reads_from_file_matching_stdin_output() {
    // Additive --request <path> input per ARCH §4.4: semantics identical
    // to stdin. Prove that by running the same request both ways against
    // one mock server and asserting identical stdout JSON.
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST)
            .path("/v1/messages")
            .header("x-api-key", "file-input-key");
        then.status(200).body(
            r#"{"id":"msg_file","model":"claude-sonnet-4-7","stop_reason":"end_turn",
               "content":[{"type":"text","text":"from-file"}],
               "usage":{"input_tokens":3,"output_tokens":2}}"#,
        );
    });

    let request = serde_json::json!({
        "model": "claude-sonnet-4-7",
        "max_tokens": 8,
        "messages": [{"role": "user", "content": "replay me"}],
    })
    .to_string();

    let mut request_file = NamedTempFile::new().expect("tempfile");
    request_file.write_all(request.as_bytes()).unwrap();
    request_file.flush().unwrap();

    let via_file = Command::new(adapter_bin())
        .arg("complete")
        .arg("--request")
        .arg(request_file.path())
        .env("ANTHROPIC_API_KEY", "file-input-key")
        .env("LERNIE_PROVIDER_ANTHROPIC_ENDPOINT", server.base_url())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn adapter (--request)");
    assert!(via_file.status.success());

    let mut via_stdin = Command::new(adapter_bin())
        .arg("complete")
        .env("ANTHROPIC_API_KEY", "file-input-key")
        .env("LERNIE_PROVIDER_ANTHROPIC_ENDPOINT", server.base_url())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn adapter (stdin)");
    via_stdin
        .stdin
        .as_mut()
        .unwrap()
        .write_all(request.as_bytes())
        .unwrap();
    drop(via_stdin.stdin.take());
    let stdin_out = via_stdin.wait_with_output().expect("wait adapter (stdin)");
    assert!(stdin_out.status.success());

    let v_file: serde_json::Value = serde_json::from_slice(&via_file.stdout).unwrap();
    let v_stdin: serde_json::Value = serde_json::from_slice(&stdin_out.stdout).unwrap();
    assert_eq!(v_file, v_stdin);
    assert_eq!(v_file["id"], "msg_file");
    assert_eq!(v_file["content"][0]["text"], "from-file");
}

#[test]
fn complete_request_flag_with_missing_file_exits_nonzero() {
    // File-open failure is an adapter-side fault, not a provider error:
    // the harness never wrote request.json the adapter was told to read.
    // Per ARCH §4.4, surface that as a non-zero exit (adapter crash),
    // not as an in-band error object.
    let out = Command::new(adapter_bin())
        .arg("complete")
        .arg("--request")
        .arg("/nonexistent/path/for/bl-becc-test.json")
        .env("ANTHROPIC_API_KEY", "k")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn adapter");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--request"), "stderr: {stderr}");
}

#[test]
fn complete_without_api_key_emits_in_band_fatal_error_and_exits_zero() {
    let mut cmd = Command::new(adapter_bin());
    cmd.arg("complete").env_remove("ANTHROPIC_API_KEY");
    let (ok, stdout) = spawn_and_capture(cmd, None);
    // In-band errors are exit 0 per ARCH §4.4 — non-zero would tell the
    // harness the adapter itself crashed.
    assert!(ok);
    let v: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(v["type"], "error");
    assert_eq!(v["kind"], "fatal");
    assert!(v["message"].as_str().unwrap().contains("ANTHROPIC_API_KEY"));
}

const STREAM_BODY: &str = concat!(
    "event: message_start\n",
    "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_e2e\",\"model\":\"claude-sonnet-4-7\",\"stop_reason\":null,\"content\":[],\"usage\":{\"input_tokens\":2,\"output_tokens\":0}}}\n\n",
    "event: content_block_start\n",
    "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"streamed\"}}\n\n",
    "event: content_block_stop\n",
    "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
    "event: message_delta\n",
    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":3}}\n\n",
    "event: message_stop\n",
    "data: {\"type\":\"message_stop\"}\n\n",
);

#[test]
fn complete_with_stream_true_emits_normalized_jsonl_end_to_end() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "text/event-stream")
            .body(STREAM_BODY);
    });

    let request = serde_json::json!({
        "model": "claude-sonnet-4-7",
        "max_tokens": 8,
        "messages": [{"role": "user", "content": "hi"}],
        "stream": true,
    });

    let mut cmd = Command::new(adapter_bin());
    cmd.arg("complete")
        .env("ANTHROPIC_API_KEY", "k")
        .env("LERNIE_PROVIDER_ANTHROPIC_ENDPOINT", server.base_url());
    let (ok, stdout) = spawn_and_capture(cmd, Some(request.to_string().as_bytes()));
    assert!(ok, "stream complete should exit 0");

    let lines: Vec<Value> = std::str::from_utf8(&stdout)
        .unwrap()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let types: Vec<&str> = lines.iter().map(|e| e["type"].as_str().unwrap()).collect();
    assert_eq!(
        types,
        vec![
            "message_start",
            "content_block_start",
            "text_delta",
            "content_block_stop",
            "message_stop",
        ]
    );
    let stop = lines.last().unwrap();
    assert_eq!(stop["api_calls"], 1);
    assert_eq!(stop["usage"]["input_tokens"], 2);
    assert_eq!(stop["usage"]["output_tokens"], 3);
    assert_eq!(stop["stop_reason"], "end_turn");
}

#[test]
fn sigterm_during_streaming_flushes_terminal_event_and_exits_within_5s() {
    // Server stalls for 8s; SIGTERM after 200ms. §4.4: adapter drops
    // in-flight HTTP, emits a terminal event, exits within 5s.
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "text/event-stream")
            .delay(Duration::from_secs(8))
            .body(STREAM_BODY);
    });
    let request = serde_json::json!({
        "model": "claude-sonnet-4-7", "max_tokens": 8,
        "messages": [{"role": "user", "content": "hi"}], "stream": true,
    });
    let mut child = Command::new(adapter_bin())
        .arg("complete")
        .env("ANTHROPIC_API_KEY", "k")
        .env("LERNIE_PROVIDER_ANTHROPIC_ENDPOINT", server.base_url())
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
    std::thread::sleep(Duration::from_millis(200));
    let started = Instant::now();
    // SAFETY: kill(2) on a child pid we own with SIGTERM is well-defined POSIX.
    let rc = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
    assert_eq!(rc, 0, "kill(SIGTERM) failed");
    let out = child.wait_with_output().expect("wait adapter");
    let elapsed = started.elapsed();
    #[rustfmt::skip]
    let _ = assert!(elapsed < Duration::from_secs(5), "adapter took {elapsed:?} (>5s)");
    assert!(out.status.success(), "adapter exited non-zero on SIGTERM");
    let line = std::str::from_utf8(&out.stdout)
        .unwrap()
        .lines()
        .find(|l| !l.is_empty())
        .expect("expected a terminal event line");
    let v: Value = serde_json::from_str(line).unwrap();
    assert_eq!(v["type"], "error");
    assert_eq!(v["kind"], "retryable");
}
