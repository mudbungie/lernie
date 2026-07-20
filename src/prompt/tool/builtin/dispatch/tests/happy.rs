//! Happy-path assertions for [`super::super::run`]: input parsing,
//! arg forwarding to the spawner, handle JSON shape, and the
//! production [`super::super::ProcessEnv`] smoke check.

use super::super::*;
use super::fixtures::{StubSpawner, env, fake_repo, input_for};
use std::io::Cursor;

#[test]
fn happy_path_writes_handle_json_and_forwards_args() {
    let (_h, repo) = fake_repo("worker");
    let mut stdin = Cursor::new(input_for("worker", "do the thing"));
    let mut stdout = Vec::new();
    let env = env(&repo, "p1-conv");
    let spawner = StubSpawner::ok("p1-conv-ct9-feedface");

    run(&mut stdin, &mut stdout, &env, &spawner).unwrap();

    let payload: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(payload["status"], "in_progress");
    assert_eq!(payload["handle"], "p1-conv-ct9-feedface");

    let calls = spawner.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "worker");
    assert_eq!(calls[0].1, repo);
    assert_eq!(calls[0].2, "p1-conv");
    assert_eq!(calls[0].3, "do the thing");
}

#[test]
fn handle_is_trimmed_of_trailing_whitespace() {
    // `lernie dispatch worker` prints with a trailing newline; the
    // handle on the wire must not carry it.
    let (_h, repo) = fake_repo("worker");
    let mut stdin = Cursor::new(input_for("worker", "g"));
    let mut stdout = Vec::new();
    let env = env(&repo, "p1");
    let spawner = StubSpawner::ok("p1-sub  ");

    run(&mut stdin, &mut stdout, &env, &spawner).unwrap();
    let payload: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(payload["handle"], "p1-sub");
}

#[test]
fn process_env_reads_live_var() {
    // Production [`ProcessEnv`] just defers to std::env. Pick a var
    // that is always set on Linux test runs (PATH).
    let p = ProcessEnv;
    assert!(p.get("PATH").is_some());
    assert!(p.get("DEFINITELY_NOT_SET_LERNIE_TEST_VAR_xxxxx").is_none());
}

#[test]
fn subprocess_spawner_with_exe_returns_captured_output() {
    // Pin the exe to `true` so the subprocess exits 0 with empty
    // stdio without touching a real lernie binary; the wrapper's
    // job is to capture and surface, regardless of what the child
    // produced.
    let s = SubprocessSpawner::with_exe(PathBuf::from("true"));
    let out = s
        .dispatch("worker", Path::new("/tmp"), "p1", "g")
        .expect("true exits cleanly");
    assert_eq!(out.exit, 0);
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
}

#[test]
fn subprocess_spawner_with_exe_returns_nonzero_for_failing_binary() {
    // `false` exits 1 unconditionally; the wrapper preserves the
    // exit code and empty stdio without inventing an io error.
    let s = SubprocessSpawner::with_exe(PathBuf::from("false"));
    let out = s
        .dispatch("worker", Path::new("/tmp"), "p1", "g")
        .expect("false runs");
    assert_eq!(out.exit, 1);
}

#[test]
fn subprocess_spawner_with_exe_surfaces_spawn_error_for_missing_binary() {
    // No binary at the given path — Command::output returns io error.
    let s = SubprocessSpawner::with_exe(PathBuf::from("/no/such/lernie-binary"));
    let err = s
        .dispatch("worker", Path::new("/tmp"), "p1", "g")
        .unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}
