//! Kill-mid-stream stopped tests (ARCH §2.9, §3.5). The on-disk
//! signature: latest step's `response.json` has bytes but no
//! `message_stop`/`error` terminal line, AND no process holds the
//! fd open — the kernel closed it on harness exit. Detected by the
//! [`crate::prompt::stop::PgidFinder`] /proc-fd scan that backs
//! `lernie stop`'s pid discovery (single source of truth — same
//! observation the §3.5 in_flight classification reads).
//!
//! Tests inject [`super::fixtures::StubPgidFinder::no_writer`] so
//! the loop sees the no-writer signal without a real process. The
//! production wiring (`ProcFsFinder` against `/proc`) is exercised
//! transitively via the `stop::discover` test suite — these tests
//! are scoped to the await-side classification.

use super::super::*;
use super::fixtures::{LiveRepo, NoopSleeper, StubPgidFinder, env, input_for};
use std::io::Cursor;

const PARTIAL_STREAM: &str = r#"{"type":"message_start"}
{"type":"content_block_delta","delta":{"text":"partial"}}
"#;

fn fixture_with_unmerged_sub() -> LiveRepo {
    let live = LiveRepo::new();
    live.run_git(&["checkout", "-b", "p1"]);
    live.run_git(&["commit", "--allow-empty", "-m", "p1 base"]);
    live.branch_and_commit("p1", "p1-sub", "marker.txt");
    live.run_git(&["checkout", "p1"]);
    live
}

fn run_against(
    live: &LiveRepo,
    finder: &dyn crate::prompt::stop::PgidFinder,
) -> serde_json::Value {
    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        finder,
        &NoopSleeper::new(),
    )
    .unwrap();
    serde_json::from_slice(&stdout).unwrap()
}

#[test]
fn killed_mid_stream_resolves_to_stopped_when_writer_absent() {
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, PARTIAL_STREAM);
    let finder = StubPgidFinder::no_writer();
    let payload = run_against(&live, &finder);
    assert_eq!(payload["status"], "stopped");
    assert!(payload.get("summary").is_none());
    let calls = finder.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0].ends_with("steps/p1-sub/001/response.json"),
        "{:?}",
        calls[0]
    );
}

#[test]
fn killed_with_no_completed_line_resolves_to_stopped() {
    // Process died after `File::create` but before `\n` landed —
    // the file holds bytes with no completed line. With no writer,
    // that's still a kill.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "{\"type\":\"message_start\"}");
    assert_eq!(
        run_against(&live, &StubPgidFinder::no_writer())["status"],
        "stopped"
    );
}

#[test]
fn killed_with_malformed_jsonl_resolves_to_stopped() {
    // A writer crashing while emitting garbage bytes is still a
    // kill — the §4.4 stream is corrupt and the fd is closed.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "garbage line\n");
    assert_eq!(
        run_against(&live, &StubPgidFinder::no_writer())["status"],
        "stopped"
    );
}

#[test]
fn killed_with_only_blank_lines_resolves_to_stopped() {
    // `\n\n\n` — every split line is empty; classifier reports
    // `NonTerminal`, and with no writer the kill arm fires.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, "\n\n\n");
    assert_eq!(
        run_against(&live, &StubPgidFinder::no_writer())["status"],
        "stopped"
    );
}

#[test]
fn writer_probe_io_error_surfaces_as_typed_error() {
    // /proc scan fails (e.g. EPERM on a hardened host). Surfaces as
    // the documented Error::Git arm so the executor concats stderr
    // into the model's tool_result.
    let live = fixture_with_unmerged_sub();
    live.write_response("p1-sub", 1, PARTIAL_STREAM);
    let mut stdin = Cursor::new(input_for("p1-sub"));
    let mut stdout = Vec::new();
    let env_stub = env(live.repo(), "p1");
    let err = run(
        &mut stdin,
        &mut stdout,
        &env_stub,
        &live.git,
        &StubPgidFinder::raises(std::io::ErrorKind::PermissionDenied),
        &NoopSleeper::new(),
    )
    .unwrap_err();
    assert!(
        matches!(
            err,
            Error::Git {
                op: "scan /proc for response.json writer",
                ..
            }
        ),
        "{err}"
    );
}
