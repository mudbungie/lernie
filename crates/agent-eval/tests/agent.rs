//! Coverage for the agent / bundler seams (ARCH §9.3, §9.2).
//!
//! `CommandAgent` and `CommandBundler` are exercised against shell stubs
//! standing in for the harness driver and `lernie bundle` — no live model
//! traffic, exactly as the runner's testability requires (§9.3).

use agent_eval::agent::{Agent, BundleTarget, Bundler, CommandAgent, CommandBundler, Dispatch};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Write an executable `sh` stub and return its path.
fn stub(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    let mut perm = std::fs::metadata(&path).unwrap().permissions();
    perm.set_mode(0o755);
    std::fs::set_permissions(&path, perm).unwrap();
    path
}

/// Invoke a stub agent whose body writes `report_body` to the report file,
/// and return the parsed outcome target.
fn dispatch_with(report_body: &str) -> Option<BundleTarget> {
    let d = tempfile::tempdir().unwrap();
    let body = match report_body {
        "<none>" => "exit 0".to_string(),
        "<empty>" => ": > \"$LERNIE_EVAL_REPORT\"".to_string(),
        other => format!("printf '{other}' > \"$LERNIE_EVAL_REPORT\""),
    };
    let prog = stub(d.path(), "agent.sh", &body);
    let home = d.path().join("home");
    let work = d.path().join("work");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&work).unwrap();
    let agent = CommandAgent::new(prog);
    agent
        .dispatch(&Dispatch {
            prompt: "do the thing",
            workdir: &work,
            lernie_home: &home,
            experiment: Path::new("/x/workflow.yaml"),
        })
        .unwrap()
        .target
}

#[test]
fn report_variants_parse() {
    // Good two-line report -> Some.
    let t = dispatch_with("ws\\nid\\n").unwrap();
    assert_eq!(t.workspace, PathBuf::from("ws"));
    assert_eq!(t.agent_id, "id");
    // No report file at all -> None (read error arm).
    assert_eq!(dispatch_with("<none>"), None);
    // Empty file -> None (missing workspace line).
    assert_eq!(dispatch_with("<empty>"), None);
    // One line only -> None (missing agent line).
    assert_eq!(dispatch_with("ws\\n"), None);
    // Empty workspace field -> None (first operand of the guard).
    assert_eq!(dispatch_with("\\nid\\n"), None);
    // Empty agent field -> None (second operand of the guard).
    assert_eq!(dispatch_with("ws\\n\\n"), None);
}

#[test]
fn agent_exit_code_is_ignored() {
    // A stub that exits non-zero but writes a valid report: the exit code
    // is not the pass signal (§9.1), so the target still parses.
    let d = tempfile::tempdir().unwrap();
    let prog = stub(
        d.path(),
        "agent.sh",
        "printf 'ws\\nid\\n' > \"$LERNIE_EVAL_REPORT\"; exit 3",
    );
    let home = d.path().join("home");
    let work = d.path().join("work");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&work).unwrap();
    let agent = CommandAgent::new(prog);
    let out = agent
        .dispatch(&Dispatch {
            prompt: "p",
            workdir: &work,
            lernie_home: &home,
            experiment: Path::new("/x"),
        })
        .unwrap();
    assert!(out.target.is_some());
}

#[test]
fn agent_spawn_failure_is_an_error() {
    let agent = CommandAgent::new("agent-eval-no-such-binary-xyz");
    let d = tempfile::tempdir().unwrap();
    let err = agent
        .dispatch(&Dispatch {
            prompt: "p",
            workdir: d.path(),
            lernie_home: d.path(),
            experiment: Path::new("/x"),
        })
        .unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn bundler_success_and_failure() {
    let d = tempfile::tempdir().unwrap();
    let dest = d.path().join("out");
    #[rustfmt::skip]
    let target = BundleTarget { workspace: d.path().join("ws"), agent_id: "a1".to_string() };

    // A stub that records its argv and exits 0.
    let ok = stub(
        d.path(),
        "ok.sh",
        "printf '%s\\n' \"$@\" > \"$(dirname \"$0\")/argv\"; exit 0",
    );
    CommandBundler::new(&ok)
        .bundle(&target, &dest)
        .expect("bundle ok");
    let argv = std::fs::read_to_string(d.path().join("argv")).unwrap();
    // lernie bundle <workspace> <agent> <dest>
    assert!(argv.contains("bundle"));
    assert!(argv.contains("a1"));

    // A stub that fails -> Err.
    let bad = stub(d.path(), "bad.sh", "exit 1");
    let err = CommandBundler::new(&bad)
        .bundle(&target, &dest)
        .unwrap_err();
    assert!(err.to_string().contains("lernie bundle exited"));
}
