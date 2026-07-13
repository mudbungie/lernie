//! Integration test: cascade. `lernie prompt` against a stalling
//! httpmock + `lernie stop` → the group SIGTERM kills `bz` (no handler),
//! and the executor catches its own copy, deposits its `stopped` result
//! on the way out, and exits cleanly (ARCH §2.9 step 3 — "Return is not a
//! verb"). response.json is left closed without a terminal `end` (the
//! stop signature, an independent write untouched by the deposit) and the
//! branch is left unmerged.
//!
//! Idempotence + error-path tests live in `tests/stop_idempotence.rs`.

mod stop_common;

use httpmock::Method::POST;
use httpmock::MockServer;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;
use stop_common::{
    HAPPY_SSE, git_command, lernie_bin, poll_for_conv_branch_with_diag, poll_for_path,
    scaffold_repo, spawn_prompt, write_brazen_config, write_global_models,
};
use tempfile::TempDir;

#[test]
fn stop_cascades_sigterm_and_leaves_response_without_terminal_end() {
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        // Long delay — `bz` blocks on the HTTP response while the
        // executor holds its inbox-directory lock fd (§2.11). `lernie
        // stop` discovers the pid by that lock fd (§2.9) and cuts the
        // cord; the open (empty) response.json is left without a
        // terminal `end` as the on-disk stop signature.
        then.status(200)
            .header("content-type", "text/event-stream")
            .delay(Duration::from_secs(30))
            .body(HAPPY_SSE);
    });

    let holder = TempDir::new().unwrap();
    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_models(&harness);
    let brazen_config = write_brazen_config(holder.path(), &server.base_url());
    let dest = holder.path().join("conv");
    scaffold_repo(&dest, &harness);

    let mut prompt_child = spawn_prompt(&dest, &harness, &brazen_config, "ping");

    let primary = dest.join("root");
    let branch =
        poll_for_conv_branch_with_diag(&primary, Duration::from_secs(15), &mut prompt_child);
    let step_dir = dest.join("steps").join(&branch).join("001");
    poll_for_path(&step_dir.join("response.json"), Duration::from_secs(15));

    let stop_out = Command::new(lernie_bin())
        .arg("stop")
        .arg(&dest)
        .arg(&branch)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .expect("spawn lernie stop");
    assert!(
        stop_out.status.success(),
        "lernie stop: {}",
        String::from_utf8_lossy(&stop_out.stderr)
    );

    // §2.9 step 3: the executor catches SIGTERM, deposits its result on
    // the way out, and exits cleanly — it does not die on the spot. (This
    // is a root conversation, so the deposit is a structural no-op; the
    // clean exit is the observable.) `bz` still died from its own copy of
    // the group SIGTERM, leaving the missing-`end` signature below.
    let prompt_status = prompt_child.wait().expect("reap lernie prompt");
    assert!(
        prompt_status.success(),
        "lernie prompt must exit cleanly after depositing on stop, got {prompt_status:?}"
    );

    // §2.9 on-disk signature: latest response.json closed and either
    // empty or whose last JSONL line is not the terminal `end`.
    let resp_path = step_dir.join("response.json");
    let resp = fs::read(&resp_path).unwrap();
    let lines: Vec<&[u8]> = resp
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .collect();
    if let Some(last) = lines.last() {
        let v: serde_json::Value = serde_json::from_slice(last).expect("trailing line is JSON");
        assert_ne!(
            v["type"].as_str(),
            Some("end"),
            "stopped response.json must not end with a terminal `end`; last: {v}"
        );
    }

    let merged_check = git_command(&primary, &["merge-base", "--is-ancestor", &branch, "main"])
        .status()
        .expect("spawn git merge-base");
    assert!(
        !merged_check.success(),
        "branch should be unmerged after stop"
    );
}

/// An Anthropic SSE that resolves to a single `bash` tool call running
/// `command` — the tool-execution window this test needs. bz normalizes
/// the `tool_use` content block (`content_block_start` + one
/// `input_json_delta`) into the canonical stream lernie records.
fn tool_use_sse(command: &str) -> String {
    let input = serde_json::json!({ "command": command }).to_string();
    let events = [
        (
            "message_start",
            serde_json::json!({"type":"message_start","message":{"id":"msg_tool","model":"claude-sonnet-4-7","stop_reason":null,"content":[],"usage":{"input_tokens":2,"output_tokens":0}}}),
        ),
        (
            "content_block_start",
            serde_json::json!({"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"toolu_1","name":"bash","input":{}}}),
        ),
        (
            "content_block_delta",
            serde_json::json!({"type":"content_block_delta","index":0,"delta":{"type":"input_json_delta","partial_json":input}}),
        ),
        (
            "content_block_stop",
            serde_json::json!({"type":"content_block_stop","index":0}),
        ),
        (
            "message_delta",
            serde_json::json!({"type":"message_delta","delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":5}}),
        ),
        ("message_stop", serde_json::json!({"type":"message_stop"})),
    ];
    events
        .iter()
        .map(|(name, data)| format!("event: {name}\ndata: {data}\n\n"))
        .collect()
}

/// §2.9 / §2.11: `lernie stop` must land during a *tool-execution
/// window* — the model call for step 1 has closed its `response.json`
/// (terminal `end`) and the executor is running a long tool, so the
/// old `response.json`-fd discovery would find no writer. Discovery via
/// the executor's inbox-directory lock fd (held for the whole loop)
/// still finds the pid, so the stop reaches the harness and its tool.
///
/// The stop landing here follows the *same* terminal sequence as the
/// model-call window (§2.9 step 3): the tool subprocess dies with the
/// group SIGTERM (its `KilledBySignal` read as the stop, not a fault),
/// the `stopped` result is deposited, and the executor exits **cleanly**
/// (exit 0) — not the non-zero crash shape a propagated `KilledBySignal`
/// used to produce. (This is a root, so the deposit is a structural
/// no-op; the clean exit is the observable, as in the model-call test.)
#[test]
fn stop_lands_during_tool_execution_via_inbox_lock_fd() {
    let holder = TempDir::new().unwrap();
    // Marker the slow tool touches once it is actually executing — a
    // deterministic "we are in the tool window" signal (no sleep-race).
    let marker = holder.path().join("tool_running");
    let command = format!("touch {} && sleep 30", marker.display());

    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(POST).path("/v1/messages");
        then.status(200)
            .header("content-type", "text/event-stream")
            .body(tool_use_sse(&command));
    });

    let harness = holder.path().join("harness");
    fs::create_dir_all(&harness).unwrap();
    write_global_models(&harness);
    // Seed the data-root tool pool so `lernie new` snapshots the bash
    // schema into `descriptions/tools/`, making the tool composable.
    let pool = harness.join("tools");
    fs::create_dir_all(&pool).unwrap();
    fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/schemas/tools/bash.json"),
        pool.join("bash.json"),
    )
    .unwrap();
    let brazen_config = write_brazen_config(holder.path(), &server.base_url());
    let dest = holder.path().join("conv");
    scaffold_repo(&dest, &harness);
    // Give the worker the bash tool (stop_common's scaffold declares none).
    fs::write(
        dest.join("providers.yaml"),
        "roles:\n  worker:\n    provider: test\n    model: claude-sonnet-4-7\n    tools: [bash]\n  compactor:\n    provider: test\n    model: claude-haiku-4-5\n",
    )
    .unwrap();

    let mut prompt_child = spawn_prompt(&dest, &harness, &brazen_config, "run a slow tool");

    let primary = dest.join("root");
    let branch =
        poll_for_conv_branch_with_diag(&primary, Duration::from_secs(15), &mut prompt_child);

    // Wait until the tool is actually running: the marker proves the
    // model call finished (step-1 response.json closed with `end`) and
    // the executor is inside `sleep 30`.
    poll_for_path(&marker, Duration::from_secs(20));

    // Discriminator: step-1 response.json ends with terminal `end`, so a
    // response.json-fd scan finds *no* open writer right now. Only the
    // inbox lock fd can reveal the live executor.
    let resp = fs::read(
        dest.join("steps")
            .join(&branch)
            .join("001")
            .join("response.json"),
    )
    .unwrap();
    let last = resp
        .split(|b| *b == b'\n')
        .rfind(|l| !l.is_empty())
        .expect("response.json has content");
    let v: serde_json::Value = serde_json::from_slice(last).expect("trailing line is JSON");
    assert_eq!(
        v["type"].as_str(),
        Some("end"),
        "step-1 response.json must be closed (fd not open) before the stop"
    );

    let stop_out = Command::new(lernie_bin())
        .arg("stop")
        .arg(&dest)
        .arg(&branch)
        .stderr(Stdio::piped())
        .output()
        .expect("spawn lernie stop");
    assert!(
        stop_out.status.success(),
        "lernie stop: {}",
        String::from_utf8_lossy(&stop_out.stderr)
    );

    // The stop reached the harness (via the lock fd) and its pgid: the
    // executor terminates promptly. Had discovery relied on the closed
    // response.json fd, no signal would have been sent and the harness
    // would still be sleeping.
    let status = wait_with_timeout(&mut prompt_child, Duration::from_secs(15))
        .expect("lernie prompt must terminate after the stop reaches it");
    // §2.9 step 3: the tool's group-SIGTERM death is classified as the
    // stop, so the executor deposits and exits *cleanly* — not the
    // non-zero exit a propagated `KilledBySignal` fault used to produce.
    assert!(
        status.success(),
        "stop during a tool window must exit cleanly (the stopped-deposit exit, §2.9 step 3), got {status:?}"
    );
}

/// Reap `child`, bounded by `deadline`. Returns its exit status, or
/// `None` (after killing it) if it outlived the deadline — a failed
/// stop leaves the harness sleeping, which this catches.
fn wait_with_timeout(
    child: &mut std::process::Child,
    deadline: Duration,
) -> Option<std::process::ExitStatus> {
    let until = std::time::Instant::now() + deadline;
    while std::time::Instant::now() < until {
        if let Some(status) = child.try_wait().expect("try_wait") {
            return Some(status);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    None
}
