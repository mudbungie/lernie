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
        // Long delay — `bz` blocks on the HTTP response, the harness has
        // opened response.json (empty) and holds its fd. `lernie stop`
        // cuts the cord.
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
