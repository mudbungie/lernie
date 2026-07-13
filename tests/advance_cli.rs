//! End-to-end test for `lernie advance` (ARCH §6): the reprompt chain
//! and the exec baton, over real subprocesses and real `bz`. Flow: `lernie prompt` answers "ping" and quiesces. `lernie message`
//! deposits a reprompt and — the §2.11 probe finding the lease free —
//! detach-spawns `lernie advance`, which delivers the deposit and steps.
//! The model call returns `tool_use`, so the hop runs the bash tool and
//! **exec's its successor** with the lock fd riding `LERNIE_LOCK_FD`
//! (§6 exec baton); the successor adopts the lease, finds the tail
//! user-side, and steps to the final response — whose exit protocol
//! launches one last driver that finds nothing due (§2.11 pin 1).
//!
//! The scripted TCP server (the `prompt_retry.rs` pattern) serves one
//! response per connection: ping → tool_use → final. The compactor is
//! the v0.3 stub (no model call), so it opens no connection.

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::TempDir;

fn lernie_bin() -> &'static str {
    env!("CARGO_BIN_EXE_lernie")
}

const HAPPY_SSE: &str = concat!(
    "event: message_start\n",
    "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_a\",\"model\":\"claude-sonnet-4-7\",\"stop_reason\":null,\"content\":[],\"usage\":{\"input_tokens\":2,\"output_tokens\":0}}}\n\n",
    "event: content_block_start\n",
    "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"pong\"}}\n\n",
    "event: content_block_stop\n",
    "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
    "event: message_delta\n",
    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":1}}\n\n",
    "event: message_stop\n",
    "data: {\"type\":\"message_stop\"}\n\n",
);

/// A `tool_use` completion: run `echo BATON-OK` through the bash tool.
const TOOL_USE_SSE: &str = concat!(
    "event: message_start\n",
    "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_t\",\"model\":\"claude-sonnet-4-7\",\"stop_reason\":null,\"content\":[],\"usage\":{\"input_tokens\":2,\"output_tokens\":0}}}\n\n",
    "event: content_block_start\n",
    "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_01\",\"name\":\"bash\",\"input\":{}}}\n\n",
    "event: content_block_delta\n",
    "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"command\\\":\\\"echo BATON-OK\\\"}\"}}\n\n",
    "event: content_block_stop\n",
    "data: {\"type\":\"content_block_stop\",\"index\":0}\n\n",
    "event: message_delta\n",
    "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":5}}\n\n",
    "event: message_stop\n",
    "data: {\"type\":\"message_stop\"}\n\n",
);

/// Serve one scripted SSE body per incoming connection.
fn spawn_seq_server(bodies: Vec<&'static str>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for body in bodies {
            let (mut stream, _) = listener.accept().expect("accept");
            drain_http_request(&mut stream);
            let resp = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\n\
                 content-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(resp.as_bytes()).expect("write response");
            stream.flush().expect("flush");
        }
    });
    format!("http://127.0.0.1:{port}")
}

fn drain_http_request(stream: &mut TcpStream) {
    let mut tmp = [0u8; 8192];
    let _ = stream.read(&mut tmp);
}

fn write_global_models(harness: &Path) {
    fs::write(
        harness.join("models.yaml"),
        "\
models:
  claude-sonnet-4-7:
    provider: test
    model_id: claude-sonnet-4-7
    capabilities: [tool_use_native]
    context_window: 200000
  claude-haiku-4-5:
    provider: test
    model_id: claude-haiku-4-5
    capabilities: [tool_use_native]
    context_window: 200000
",
    )
    .unwrap();
}

fn write_brazen_config(dir: &Path, endpoint: &str) -> std::path::PathBuf {
    let toml = format!(
        "timeout_connect = 5\ntimeout_response = 10\ntimeout_idle = 10\n\
         [[provider]]\nname = \"test\"\nbase_url = \"{endpoint}\"\n\
         protocol = \"anthropic_messages\"\nauth = \"none\"\n\
         body_defaults = {{ max_tokens = 64 }}\n"
    );
    let path = dir.join("brazen.toml");
    fs::write(&path, toml).unwrap();
    path
}

const ROLES_YAML: &str = "\
roles:
  worker:
    provider: test
    model: claude-sonnet-4-7
    tools: [bash]
  compactor:
    provider: test
    model: claude-haiku-4-5
";

fn scaffold(dest: &Path, harness: &Path) {
    let out = Command::new(lernie_bin())
        .arg("new")
        .arg(dest)
        .env("LERNIE_HOME", harness)
        .output()
        .expect("spawn lernie new");
    assert!(
        out.status.success(),
        "lernie new: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Config-commit amendment (§2.2): point roles at the fixture row,
    // over the shipped authoring core rather than hand-rolled worktrees.
    lernie::template::authoring::author(
        dest,
        &dest.join(".no-pools"),
        "default",
        lernie::template::authoring::Origin::Advance,
        |dir| fs::write(dir.join("providers.yaml"), ROLES_YAML),
        &lernie::template::RealGit::new(),
    )
    .unwrap();
}

/// Poll for `path` to exist, up to `deadline` — the driver chain runs
/// detached, so the test observes disk, exactly like a frontend (§3.5).
fn wait_for(path: &Path, deadline: Duration) {
    let start = Instant::now();
    while !path.exists() {
        assert!(start.elapsed() < deadline, "timed out waiting for {path:?}");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn message_launches_a_detached_advance_chain_that_batons_through_tools() {
    let endpoint = spawn_seq_server(vec![HAPPY_SSE, TOOL_USE_SSE, HAPPY_SSE]);
    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_models(&harness);
    let brazen_config = write_brazen_config(holder.path(), &endpoint);
    let dest = holder.path().join("conv");
    scaffold(&dest, &harness);

    // Exchange 1: `lernie prompt` answers and quiesces. Its exit launch
    // spawns a real driver, which finds nothing due and exits silently
    // (§2.11 pin 1) — the recursion terminator, live.
    let out = Command::new(lernie_bin())
        .arg("prompt")
        .arg(&dest)
        .arg("ping")
        .env("LERNIE_HOME", &harness)
        .env("BRAZEN_CONFIG", &brazen_config)
        .output()
        .expect("spawn lernie prompt");
    assert!(
        out.status.success(),
        "lernie prompt: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let conv = String::from_utf8(out.stdout).unwrap().trim().to_string();

    // Exchange 2: the reprompt is a message (§2.4). The deposit probe
    // finds the lease free and detach-spawns `lernie advance`; the verb
    // returns immediately — delivery and stepping continue in the driver.
    let out = Command::new(lernie_bin())
        .arg("message")
        .arg(&dest)
        .arg(&conv)
        .arg("again")
        .env("LERNIE_HOME", &harness)
        .env("BRAZEN_CONFIG", &brazen_config)
        .output()
        .expect("spawn lernie message");
    assert!(
        out.status.success(),
        "lernie message: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // The chain: deliver (003-user) → step 2 (004 tool_use) → bash tool
    // (005-tool) → exec successor with the lease riding LERNIE_LOCK_FD →
    // step 3 (006 final response).
    let messages = dest.join("agents").join(&conv).join("messages");
    let deadline = Duration::from_secs(120);
    wait_for(&messages.join("003-user.md"), deadline);
    wait_for(&messages.join("004-claude-sonnet-4-7.json"), deadline);
    wait_for(&messages.join("005-tool.json"), deadline);
    wait_for(&messages.join("006-claude-sonnet-4-7.json"), deadline);

    let tool_entry = fs::read_to_string(messages.join("005-tool.json")).unwrap();
    assert!(tool_entry.contains("BATON-OK"), "got {tool_entry:?}");

    // Both hops recorded their steps at the derived sequence; the
    // successor's response closed with a terminal `end` (§4.4).
    let step3 = dest.join(format!("steps/{conv}/003/response.json"));
    wait_for(&step3, deadline);
    let deadline_at = Instant::now() + deadline;
    loop {
        let lines: Vec<serde_json::Value> = fs::read(&step3)
            .unwrap()
            .split(|b| *b == b'\n')
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::from_slice(l).expect("valid JSON line"))
            .collect();
        if lines.last().map(|e| e["type"] == "end").unwrap_or(false) {
            break;
        }
        assert!(Instant::now() < deadline_at, "step 3 never completed");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn advance_verb_surfaces_an_unusable_workspace_loudly() {
    let out = Command::new(lernie_bin())
        .args(["advance", "/no/such/workspace", "20260101-a1"])
        .output()
        .expect("spawn lernie advance");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("lernie advance"));
}

#[test]
fn advance_on_a_quiescent_empty_agent_is_a_silent_noop() {
    // A real workspace, an agent id with no branch and no mail: the
    // driver acquires, finds nothing due, exits silently (§2.11 pin 1).
    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_models(&harness);
    let dest = holder.path().join("ws");
    scaffold(&dest, &harness);
    let out = Command::new(lernie_bin())
        .args(["advance"])
        .arg(&dest)
        .arg("20260101-a1")
        .env("LERNIE_HOME", &harness)
        .output()
        .expect("spawn lernie advance");
    assert!(
        out.status.success(),
        "lernie advance: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.is_empty(), "a no-op driver is silent");
}
